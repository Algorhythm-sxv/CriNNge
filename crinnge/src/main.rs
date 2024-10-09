mod uci;

use std::{
    env,
    error::Error,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
    time::Instant,
};

use crinnge_lib::{
    board::Board,
    moves::MoveList,
    nnue::{Accumulator, NNUE},
    search::{
        info::{SearchInfo, UCI_QUIT},
        options::SearchOptions,
    },
    thread_data::ThreadData,
    timeman::{TimeData, TimeManager},
    tt::TT,
    types::*,
};
use uci::stdin_reader;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Box<dyn Error>> {
    let mut board = Board::new();
    let mut tt = TT::new(8);
    let mut search_options = SearchOptions::default();
    let mut threads_data = vec![ThreadData::new(&board, tt.slice()); search_options.threads];

    if env::args().nth(1) == Some("bench".to_string()) {
        let start_time = Instant::now();
        let time_manager = TimeManager::new(start_time).fixed_depth(Some(8));
        let node_counter = AtomicU64::new(0);
        let stop_signal = AtomicBool::new(false);
        let mut info = SearchInfo::new(&stop_signal, &node_counter)
            .time_manager(time_manager)
            .stdout(false);

        board.search(&mut info, &mut threads_data);

        let nodes = info.global_node_count();
        let elapsed = time_manager.elapsed().as_millis() as u64;
        println!(
            "{} Nodes {} NPS",
            info.global_node_count(),
            nodes * 1000 / elapsed
        );

        return Ok(());
    }

    let stdin_rx = Mutex::new(stdin_reader());
    'command: loop {
        let Ok(line) = stdin_rx.lock().unwrap().recv() else {
            UCI_QUIT.store(true, Ordering::Relaxed);
            break;
        };
        let command = uci::parse(&line);

        if let Err(e) = command {
            eprintln!("info string {:?}", e);
            continue;
        }

        match command.unwrap() {
            uci::UciCommand::Uci => uci::print_uci_message(),
            uci::UciCommand::UciNewGame => {
                board = Board::new();
                drop(threads_data);
                drop(tt);
                tt = TT::new(search_options.hash);
                threads_data = vec![ThreadData::new(&board, tt.slice()); search_options.threads];
            }
            uci::UciCommand::IsReady => println!("readyok"),
            uci::UciCommand::Position { start_fen, moves } => {
                let mut test_board = if let Some(fen) = start_fen {
                    Board::from_fen(&fen).unwrap()
                } else {
                    Board::new()
                };
                let mut prehistory = vec![test_board.hash()];

                for mv in moves.iter() {
                    let legals = test_board.legal_moves();
                    let legal = legals.iter().find(|m| &m.coords() == mv);
                    if let Some(mv) = legal {
                        prehistory.push(test_board.hash());
                        assert!(test_board.make_move_only(*mv));
                    } else {
                        eprintln!("info string Illegal move: {mv}");
                        continue 'command;
                    }
                }
                board = test_board;
                for t in threads_data.iter_mut() {
                    board.refresh_accumulator(&mut t.accumulators[0]);
                    t.search_history.clone_from(&prehistory);
                }
            }
            uci::UciCommand::Fen => {
                println!("info string {}", board.fen());
            }
            uci::UciCommand::SetOption => {
                // placeholder
            }
            uci::UciCommand::Go(options) => {
                if let Some(depth) = options.perft {
                    perft(&board, depth);
                    continue;
                }

                let (stm_time, stm_inc, ntm_time, ntm_inc) = if board.player() == White {
                    (
                        options.wtime.unwrap_or(0),
                        options.winc.unwrap_or(0),
                        options.btime.unwrap_or(0),
                        options.binc.unwrap_or(0),
                    )
                } else {
                    (
                        options.btime.unwrap_or(0),
                        options.binc.unwrap_or(0),
                        options.wtime.unwrap_or(0),
                        options.winc.unwrap_or(0),
                    )
                };
                let time_data = TimeData {
                    stm_time,
                    ntm_time,
                    stm_inc,
                    ntm_inc,
                    movestogo: options.movestogo,
                };
                let time_manager = TimeManager::new(Instant::now())
                    .time_limited(time_data, search_options.time_options())
                    .fixed_depth(options.depth)
                    .fixed_nodes(options.nodes)
                    .fixed_time_millis(options.movetime)
                    .infinite(options.infinite);

                let stop_signal = AtomicBool::new(false);
                let global_nodes = AtomicU64::new(0);
                let mut info = SearchInfo::new(&stop_signal, &global_nodes)
                    .time_manager(time_manager)
                    .stdin(Some(&stdin_rx));

                board.search(&mut info, &mut threads_data);
            }
            uci::UciCommand::Eval => {
                let mut acc = Accumulator::new();
                board.refresh_accumulator(&mut acc);
                let weval = NNUE.evaluate(&acc.white);
                let beval = NNUE.evaluate(&acc.black);
                println!("info string white eval: {weval}");
                println!("info string black eval: {beval}");
            }
            uci::UciCommand::Quit => {
                break;
            }
        }
    }
    Ok(())
}

fn perft(board: &Board, depth: usize) {
    if depth == 0 {
        println!("Total: 1\tNPS: 0")
    }

    let start = Instant::now();
    let mut count = 0usize;
    let [mut noisy, mut quiet] = [MoveList::new(); 2];

    board.generate_moves_into(&mut noisy, &mut quiet);

    for mv in noisy.iter_moves().chain(quiet.iter_moves()) {
        let mut next = *board;
        if next.make_move_only(*mv) {
            let subcount = _perft(&next, depth - 1);
            count += subcount;
            println!("{}: {}", mv.coords(), subcount)
        }
    }
    let end = Instant::now();
    let nps = count * 1000 / (end - start).as_millis().max(1) as usize;

    println!("\nTotal: {count}\tNPS: {nps}");
}

fn _perft(board: &Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut count = 0usize;
    let [mut noisy, mut quiet] = [MoveList::new(); 2];

    board.generate_moves_into(&mut noisy, &mut quiet);

    for mv in noisy.iter_moves().chain(quiet.iter_moves()) {
        let mut next = *board;
        if next.make_move_only(*mv) {
            count += _perft(&next, depth - 1)
        }
    }

    count
}
