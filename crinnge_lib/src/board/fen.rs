use crinnge_bitboards::{BitBoard, Square};

use super::Board;
use crate::types::*;

impl Board {
    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut board = Board::empty();

        let parts: Vec<_> = fen.trim().split(' ').collect();

        let mut rank: u32 = 7;
        let mut file: u32 = 0;
        for chr in parts.first()?.chars() {
            let pieces = match chr.to_ascii_lowercase() {
                'p' => &mut board.pawns,
                'n' => &mut board.knights,
                'b' => &mut board.bishops,
                'r' => &mut board.rooks,
                'q' => &mut board.queens,
                'k' => &mut board.kings,
                n @ '1'..='8' => {
                    file += n.to_digit(10).unwrap();
                    continue;
                }
                '/' => {
                    rank = rank.checked_sub(1)?;
                    file = 0;
                    continue;
                }
                _ => return None,
            };

            if file > 7 {
                return None;
            }
            let color = chr.is_ascii_lowercase();
            pieces[color as usize] |= Square::from(rank * 8 + file).bitboard();
            file += 1;
        }
        board.occupied[White] = board.pawns[White]
            | board.knights[White]
            | board.bishops[White]
            | board.rooks[White]
            | board.queens[White]
            | board.kings[White];
        board.occupied[Black] = board.pawns[Black]
            | board.knights[Black]
            | board.bishops[Black]
            | board.rooks[Black]
            | board.queens[Black]
            | board.kings[Black];

        match *parts.get(1)? {
            "w" => board.player = White,
            "b" => board.player = Black,
            _ => return None,
        }

        for chr in parts.get(2)?.chars() {
            let file = match chr.to_ascii_lowercase() {
                '-' => break,
                'k' => 7,
                'q' => 0,
                n @ 'a'..='h' => n as u32 - 'a' as u32,
                _ => return None,
            };
            let color = if chr.is_ascii_lowercase() {
                Black
            } else {
                White
            };
            let castle = Square::from(7 * 8 * color as u32 + file).bitboard();
            let king_file = board.kings[color].first_square().file();
            let kingside = file > king_file as u32;
            board.castles[color][kingside as usize] = castle;
        }

        board.ep_mask = match parts.get(3)?.split_at(1) {
            (f @ ("a" | "b" | "c" | "d" | "e" | "f" | "g" | "h"), "3") => {
                file = f.chars().next().unwrap() as u32 - 'a' as u32;
                Square::from(3 * 8 + file).bitboard()
            }
            (f @ ("a" | "b" | "c" | "d" | "e" | "f" | "g" | "h"), "6") => {
                file = f.chars().next().unwrap() as u32 - 'a' as u32;
                Square::from(6 * 8 + file).bitboard()
            }
            ("-", _) => BitBoard::empty(),
            _ => return None,
        };

        board.halfmove_clock = if let Some(n) = parts.get(4) {
            n.parse::<u8>().ok()?
        } else {
            0
        };

        board.fullmove_count = if let Some(n) = parts.get(5) {
            n.parse::<u16>().ok()?
        } else {
            1
        };

        Some(board)
    }

    pub fn fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut space_counter = 0;
            for file in 0..8 {
                let sq = Square::from(rank * 8 + file);
                if let Some(piece) = self.piece_on(sq) {
                    if space_counter != 0 {
                        fen.push_str(&space_counter.to_string());
                        space_counter = 0;
                    }
                    let color = if (self.occupied[White] & sq.bitboard()).is_not_empty() {
                        White
                    } else {
                        Black
                    };
                    let mut letter = match piece {
                        Pawn => 'p',
                        Knight => 'n',
                        Bishop => 'b',
                        Rook => 'r',
                        Queen => 'q',
                        King => 'k',
                    };
                    if color == White {
                        letter = letter.to_ascii_uppercase();
                    }
                    fen.push(letter);
                } else {
                    space_counter += 1
                }
            }
            if space_counter != 0 {
                fen.push_str(&space_counter.to_string());
            }
            if rank != 0 {
                fen.push('/')
            }
        }

        fen.push_str(match self.player {
            White => " w",
            Black => " b",
        });

        if self.castles == [[BitBoard::empty(); 2]; 2] {
            fen.push_str(" -");
        } else {
            fen.push(' ');
            // TODO: FRC Castling
            if self.castles[White][1].is_not_empty() {
                fen.push('K');
            }
            if self.castles[White][0].is_not_empty() {
                fen.push('Q');
            }
            if self.castles[Black][1].is_not_empty() {
                fen.push('k');
            }
            if self.castles[Black][0].is_not_empty() {
                fen.push('q');
            }
        }

        if self.ep_mask.is_empty() {
            fen.push_str(" -");
        } else {
            fen.push(' ');
            fen.push_str(&self.ep_mask.first_square().coord());
        }

        fen.push_str(&format!(" {}", self.halfmove_clock));
        fen.push_str(&format!(" {}", self.fullmove_count));

        fen
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;

    #[test]
    fn test_fen_equality() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert!(fen == board.fen());
    }
}
