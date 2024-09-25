use crinnge_lib::board::Board;

pub enum UciCommand {
    Position {
        start_fen: Option<String>,
        moves: Vec<String>,
    },
    Perft {
        depth: usize,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum UciError {
    EmptyCommand,
    IncompleteCommand,
    UnknownCommand,
    InvalidFen,
    InvalidPositionCommand,
    InvalidPerftCommand,
}

pub fn parse(command: String) -> Result<UciCommand, UciError> {
    let parts: Vec<_> = command.split(' ').collect();

    match *parts.get(0).ok_or(UciError::EmptyCommand)? {
        "position" => parse_position_command(&parts),
        "perft" => parse_perft_command(&parts),
        _ => Err(UciError::UnknownCommand),
    }
}

pub fn parse_position_command(parts: &[&str]) -> Result<UciCommand, UciError> {
    let start_fen = match *parts.get(1).ok_or(UciError::IncompleteCommand)? {
        "startpos" => None,
        "fen" => {
            let fen_parts = parts.get(2..=7).ok_or(UciError::IncompleteCommand)?;
            let fen = fen_parts.join(" ");
            if let Some(_) = Board::from_fen(&fen) {
                Some(fen)
            } else {
                Err(UciError::InvalidFen)?
            }
        }
        _ => Err(UciError::InvalidPositionCommand)?,
    };

    let moves = match parts.get(8) {
        Some(&"moves") => {
            let mut moves = Vec::new();
            if let Some(mvs) = parts.get(9..) {
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

fn parse_perft_command(parts: &[&str]) -> Result<UciCommand, UciError> {
    let depth = parts.get(1).ok_or(UciError::IncompleteCommand)?;
    let depth = depth
        .parse::<usize>()
        .map_err(|_| UciError::InvalidPerftCommand)?;

    Ok(UciCommand::Perft { depth })
}
