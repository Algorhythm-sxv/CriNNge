mod uci;

use std::{error::Error, io::stdin, time::Instant};

use crinnge_lib::{
    board::Board,
    moves::MoveList,
    nnue::{Accumulator, NNUE}, thread_data::ThreadData,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut board = Board::new();

    for line in stdin().lines() {
        let command = uci::parse(line?);

        if let Err(e) = command {
            eprintln!("info string {:?}", e);
            continue;
        }

        match command.unwrap() {
            uci::UciCommand::Position { start_fen, moves } => {
                let mut test_board = if let Some(fen) = start_fen {
                    Board::from_fen(&fen).unwrap()
                } else {
                    Board::new()
                };
                let mut board_moves = MoveList::new();
                let mut t = ThreadData::new();
                test_board.refresh_accumulator(&mut t.accumulators[0]);
                for (i, mv) in moves.iter().enumerate() {
                    test_board.generate_moves_into(&mut board_moves);
                    if let Some(mv) = board_moves.iter_moves().iter().find(|m| m.coords() == *mv) {
                        if !test_board.make_move_nnue(*mv, &mut t, i) {
                            eprintln!("info string Illegal move: {}", mv.coords());
                        }
                    } else {
                        eprintln!("info string Illegal move: {mv}");
                    }
                }
                board = test_board;
            }
            uci::UciCommand::Fen => {
                println!("info string {}", board.fen());
            }
            uci::UciCommand::Perft { depth } => {
                perft(&board, depth);
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
    let mut moves = MoveList::new();

    board.generate_moves_into(&mut moves);

    for mv in moves.iter_moves() {
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
    let mut moves = MoveList::new();

    board.generate_moves_into(&mut moves);

    for mv in moves.iter_moves() {
        let mut next = *board;
        if next.make_move_only(*mv) {
            count += _perft(&next, depth - 1)
        }
    }

    count
}
