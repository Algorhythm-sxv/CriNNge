use std::{
    io::stdin,
    sync::{
        atomic::Ordering,
        mpsc::{self, Receiver},
    },
    thread,
};

use crinnge_lib::{
    board::Board,
    search::{info::UCI_QUIT, options::SearchOptions, INF, MAX_DEPTH},
};

use crate::VERSION;

pub struct GoCommand {
    pub perft: Option<usize>,
    pub infinite: bool,
    pub wtime: Option<i64>,
    pub btime: Option<i64>,
    pub winc: Option<i64>,
    pub binc: Option<i64>,
    pub movetime: Option<i64>,
    pub movestogo: Option<usize>,
    pub depth: Option<usize>,
    pub nodes: Option<u64>,
}
pub enum UciCommand {
    Uci,
    UciNewGame,
    IsReady,
    Position {
        start_fen: Option<String>,
        moves: Vec<String>,
    },
    Fen,
    Go(GoCommand),
    SetOption(String, String),
    Eval,
    Quit,
}

#[derive(Clone, Debug)]
pub enum UciError {
    EmptyCommand,
    IncompleteCommand,
    UnknownCommand,
    InvalidFen,
    InvalidPositionCommand,
    InvalidGoCommand,
    InvalidSetoptionCommand,
    UnknownOption(String),
}

pub fn parse(command: &str, options: &mut SearchOptions) -> Result<UciCommand, UciError> {
    let parts: Vec<_> = command.split(' ').collect();

    match parts
        .first()
        .ok_or(UciError::EmptyCommand)?
        .to_ascii_lowercase()
        .as_str()
    {
        "uci" => Ok(UciCommand::Uci),
        "ucinewgame" => Ok(UciCommand::UciNewGame),
        "isready" => Ok(UciCommand::IsReady),
        "position" => parse_position_command(&parts),
        "fen" => Ok(UciCommand::Fen),
        "go" => parse_go_command(&parts),
        "setoption" => parse_setoption_command(&parts, options),
        "eval" => Ok(UciCommand::Eval),
        "quit" => Ok(UciCommand::Quit),
        _ => Err(UciError::UnknownCommand),
    }
}

pub fn parse_position_command(parts: &[&str]) -> Result<UciCommand, UciError> {
    let start_fen = match *parts.get(1).ok_or(UciError::IncompleteCommand)? {
        "startpos" => None,
        "fen" => {
            let fen_parts = parts.get(2..=7).ok_or(UciError::IncompleteCommand)?;
            let fen = fen_parts.join(" ");
            if Board::from_fen(&fen).is_some() {
                Some(fen)
            } else {
                Err(UciError::InvalidFen)?
            }
        }
        _ => Err(UciError::InvalidPositionCommand)?,
    };

    let moves_start = if start_fen.is_some() { 8 } else { 2 };

    let moves = match parts.get(moves_start) {
        Some(&"moves") => {
            let mut moves = Vec::new();
            if let Some(mvs) = parts.get((moves_start + 1)..) {
                for mv in mvs {
                    moves.push(mv.to_string());
                }
            }
            moves
        }
        Some(_) => Err(UciError::InvalidPositionCommand)?,
        None => vec![],
    };

    Ok(UciCommand::Position { start_fen, moves })
}

fn parse_go_command(parts: &[&str]) -> Result<UciCommand, UciError> {
    let rest = &parts[1..];

    let perft = if let Some(i) = rest.iter().position(|&w| w == "perft") {
        let n = rest.get(i + 1).ok_or(UciError::IncompleteCommand)?;
        Some(n.parse::<usize>().map_err(|_| UciError::InvalidGoCommand)?)
    } else {
        None
    };

    let infinite = rest.contains(&"infinite");

    let wtime = if let Some(i) = rest.iter().position(|&w| w == "wtime") {
        let n = rest.get(i + 1).ok_or(UciError::IncompleteCommand)?;
        Some(n.parse::<i64>().map_err(|_| UciError::InvalidGoCommand)?)
    } else {
        None
    };
    let winc = if let Some(i) = rest.iter().position(|&w| w == "winc") {
        let n = rest.get(i + 1).ok_or(UciError::IncompleteCommand)?;
        Some(n.parse::<i64>().map_err(|_| UciError::InvalidGoCommand)?)
    } else {
        None
    };
    let btime = if let Some(i) = rest.iter().position(|&w| w == "btime") {
        let n = rest.get(i + 1).ok_or(UciError::IncompleteCommand)?;
        Some(n.parse::<i64>().map_err(|_| UciError::InvalidGoCommand)?)
    } else {
        None
    };
    let binc = if let Some(i) = rest.iter().position(|&w| w == "binc") {
        let n = rest.get(i + 1).ok_or(UciError::IncompleteCommand)?;
        Some(n.parse::<i64>().map_err(|_| UciError::InvalidGoCommand)?)
    } else {
        None
    };

    let movetime = if let Some(i) = rest.iter().position(|&w| w == "movetime") {
        let n = rest.get(i + 1).ok_or(UciError::IncompleteCommand)?;
        Some(n.parse::<i64>().map_err(|_| UciError::InvalidGoCommand)?)
    } else {
        None
    };

    let movestogo = if let Some(i) = rest.iter().position(|&w| w == "movestogo") {
        let n = rest.get(i + 1).ok_or(UciError::IncompleteCommand)?;
        Some(n.parse::<usize>().map_err(|_| UciError::InvalidGoCommand)?)
    } else {
        None
    };

    let depth = if let Some(i) = rest.iter().position(|&w| w == "depth") {
        let n = rest.get(i + 1).ok_or(UciError::IncompleteCommand)?;
        Some(n.parse::<usize>().map_err(|_| UciError::InvalidGoCommand)?)
    } else {
        None
    };

    let nodes = if let Some(i) = rest.iter().position(|&w| w == "nodes") {
        let n = rest.get(i + 1).ok_or(UciError::IncompleteCommand)?;
        Some(n.parse::<u64>().map_err(|_| UciError::InvalidGoCommand)?)
    } else {
        None
    };

    Ok(UciCommand::Go(GoCommand {
        perft,
        infinite,
        wtime,
        btime,
        winc,
        binc,
        movetime,
        movestogo,
        depth,
        nodes,
    }))
}

fn parse_setoption_command(
    parts: &[&str],
    options: &mut SearchOptions,
) -> Result<UciCommand, UciError> {
    if parts.get(1) != Some(&"name") {
        return Err(UciError::InvalidSetoptionCommand);
    }

    let Some(name) = parts.get(2) else {
        return Err(UciError::InvalidSetoptionCommand);
    };

    if parts.get(3) != Some(&"value") {
        return Err(UciError::InvalidSetoptionCommand);
    }

    let Some(value) = parts.get(4) else {
        return Err(UciError::InvalidSetoptionCommand);
    };

    match name.to_ascii_lowercase().as_str() {
        "threads" => {
            let Ok(n @ 1..=999) = value.parse::<usize>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.threads = n;
        }
        "hash" => {
            let Ok(n @ 0..=999999) = value.parse::<usize>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.hash = n;
        }
        "aspwindowinit" => {
            let Ok(n @ 1..=INF) = value.parse::<i32>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.asp_window_init = n;
        }
        "aspwindowscalepercent" => {
            let Ok(n @ 1..=INF) = value.parse::<i32>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.asp_window_scale_percent = n;
        }
        "hardtimepercent" => {
            let Ok(n @ 1..=100) = value.parse::<i64>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.hard_time_percent = n;
        }
        "softtimepercent" => {
            let Ok(n @ 1..=100) = value.parse::<i64>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.soft_time_percent = n;
        }
        "incpercent" => {
            let Ok(n @ 1..=100) = value.parse::<i64>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.inc_percent = n;
        }
        "nmpmindepth" => {
            let Ok(n @ 1..=MAX_DEPTH) = value.parse::<i32>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.nmp_min_depth = n;
        }
        "nmpreductionconst" => {
            let Ok(n @ 1..=MAX_DEPTH) = value.parse::<i32>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.nmp_r_const = n;
        }
        "nmpreductiondepthdivisor" => {
            let Ok(n @ 1..=MAX_DEPTH) = value.parse::<i32>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.nmp_r_depth_divisor = n;
        }
        "rfpmaxdepth" => {
            let Ok(n @ 1..=MAX_DEPTH) = value.parse::<i32>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.rfp_max_depth = n;
        }
        "rfpmargin" => {
            let Ok(n @ 1..=INF) = value.parse::<i32>() else {
                return Err(UciError::InvalidSetoptionCommand);
            };
            options.rfp_margin = n;
        }
        _ => return Err(UciError::UnknownOption(name.to_string())),
    }

    Ok(UciCommand::SetOption(
        name.to_ascii_lowercase().to_string(),
        value.to_string(),
    ))
}

pub fn print_uci_message() {
    println!("id name CriNNge {}", VERSION);
    println!("id author Algorhythm");
    print!("{}", SearchOptions::default());
    println!("uciok");
}

pub fn stdin_reader() -> Receiver<String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for line in stdin().lines() {
            let Ok(line) = line else {
                eprintln!("info string stdin read error");
                UCI_QUIT.store(true, Ordering::Relaxed);
                break;
            };
            let _ = tx.send(line.clone());
            if line.starts_with("quit") {
                UCI_QUIT.store(true, Ordering::Relaxed);
                break;
            }
        }
    });

    rx
}
