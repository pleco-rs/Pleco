//! Functions for parsing UCI input, including both time data & the position of the board to be searched.

use pleco::Board;

use time::uci_timer::{PreLimits, UCITimer};

fn is_keyword(arg: &str) -> bool {
    match arg {
        "searchmoves" | "ponder" | "wtime" | "btime" | "winc" | "binc" | "movestogo" | "depth"
        | "nodes" | "mate" | "movetime" | "infinite" => true,
        _ => false,
    }
}

// when "go" is passed into stdin, followed by several time control parameters
// "searchmoves" "move"+
// "ponder"
// "wtime" "[msec]"
// "btime" "[msec]"
// "winc" "[msec]"
// "binc" "[msec]"
// "movestogo" "[u32]"
// "depth" "[u16]"
// "nodes" "[u64]"
// "mate" "[moves]"
// movetime "msec"
// "infinite"
pub fn parse_time(args: &[&str]) -> PreLimits {
    let mut token_idx: usize = 0;
    let mut limit = PreLimits::blank();
    let mut timer = UCITimer::blank();
    while let Some(token) = args.get(token_idx) {
        match *token {
            "infinite" => {
                limit.infinite = true;
            }
            "ponder" => {
                limit.ponder = true;
            }
            "wtime" => {
                if let Some(wtime_s) = args.get(token_idx + 1) {
                    if let Ok(wtime) = wtime_s.parse::<i64>() {
                        timer.time_msec[0] = wtime;
                    }
                    token_idx += 1;
                }
            }
            "btime" => {
                if let Some(btime_s) = args.get(token_idx + 1) {
                    if let Ok(btime) = btime_s.parse::<i64>() {
                        timer.time_msec[1] = btime;
                    }
                    token_idx += 1;
                }
            }
            "winc" => {
                if let Some(winc_s) = args.get(token_idx + 1) {
                    if let Ok(winc) = winc_s.parse::<i64>() {
                        timer.inc_msec[0] = winc;
                    }
                    token_idx += 1;
                }
            }
            "binc" => {
                if let Some(binc_s) = args.get(token_idx + 1) {
                    if let Ok(binc) = binc_s.parse::<i64>() {
                        timer.inc_msec[1] = binc;
                    }
                    token_idx += 1;
                }
            }
            "movestogo" => {
                if let Some(movestogo_s) = args.get(token_idx + 1) {
                    if let Ok(movestogo) = movestogo_s.parse::<u32>() {
                        timer.moves_to_go = movestogo;
                    }
                    token_idx += 1;
                }
            }
            "depth" => {
                if let Some(depth_s) = args.get(token_idx + 1) {
                    if let Ok(depth) = depth_s.parse::<u16>() {
                        limit.depth = Some(depth);
                    }
                    token_idx += 1;
                }
            }
            "nodes" => {
                if let Some(nodes_s) = args.get(token_idx + 1) {
                    if let Ok(nodes) = nodes_s.parse::<u64>() {
                        limit.nodes = Some(nodes);
                    }
                    token_idx += 1;
                }
            }
            "mate" => {
                if let Some(mate_s) = args.get(token_idx + 1) {
                    if let Ok(mate) = mate_s.parse::<u16>() {
                        limit.mate = Some(mate);
                    }
                    token_idx += 1;
                }
            }
            "movetime" => {
                if let Some(movetime_s) = args.get(token_idx + 1) {
                    if let Ok(movetime) = movetime_s.parse::<u64>() {
                        limit.move_time = Some(movetime);
                    }
                    token_idx += 1;
                }
            }
            "searchmoves" => 'searchmoves: loop {
                if let Some(mov) = args.get(token_idx + 1) {
                    if !is_keyword(mov) {
                        limit.search_moves.push((*mov).to_string());
                        token_idx += 1;
                    } else {
                        break 'searchmoves;
                    }
                } else {
                    break 'searchmoves;
                }
            },
            _ => {}
        }
        token_idx += 1;
    }
    if !timer.is_blank() {
        limit.time = Some(timer);
    }
    limit
}

fn valid_move(board: &mut Board, mov: &str) -> bool {
    let all_moves = board
        .generate_moves()
        .iter()
        .map(|m| m.stringify())
        .collect::<Vec<String>>();

    if all_moves.contains(&mov.to_string()) {
        //        println!("Yes: {}", mov);
        return board.apply_uci_move(mov);
    }
    //    println!("Nope: :{}:", mov);
    //    for mov3 in all_moves.iter() {
    //        println!("mov: :{}:", &*mov3);
    //    }
    false
}

pub fn setboard_parse_board(args: &[&str]) -> Option<Board> {
    let fen_string: String = args
        .iter()
        .take_while(|p: &&&str| **p != "moves")
        .map(|p| (*p).to_string())
        .collect::<Vec<String>>()
        .join(" ");
    Board::from_fen(&fen_string).ok()
}

pub fn position_parse_board(args: &[&str]) -> Option<Board> {
    let start: &str = args[0];
    let mut board = if start == "startpos" {
        Some(Board::start_pos())
    } else if start == "fen" {
        let fen_string: String = args[1..]
            .iter()
            .take_while(|p: &&&str| **p != "moves")
            .map(|p| (*p).to_string())
            .collect::<Vec<String>>()
            .join(" ");
        Board::from_fen(&fen_string).ok()
    } else {
        None
    };

    let mut moves_start: Option<usize> = None;
    for (i, mov) in args.iter().enumerate() {
        if *mov == "moves" {
            moves_start = Some(i);
        }
    }

    if let Some(start) = moves_start {
        if let Some(ref mut op_board) = board {
            let mut index = start + 1;
            while index < args.len() {
                if !(valid_move(op_board, args[index])) {
                    break;
                }
                index += 1;
            }
        }
    }
    board
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleco::Player;

    // TODO: More testing

    #[test]
    fn board_parse() {
        let b_str = "position startpos moves e2e4 e7e5";
        let args: Vec<&str> = b_str.split_whitespace().collect();
        let board = position_parse_board(&args[1..]).unwrap();
        assert_eq!(board.moves_played(), 2);
        assert_eq!(board.turn(), Player::White);

        let b_str = "position startpos";
        let args: Vec<&str> = b_str.split_whitespace().collect();
        let board = position_parse_board(&args[1..]).unwrap();
        assert_eq!(board.moves_played(), 0);
    }

    #[test]
    fn time_parse() {
        let t_str = "go infinite searchmoves e2e4 d2d4";
        let args: Vec<&str> = t_str.split_whitespace().collect();
        let time = parse_time(&args[1..]);
        assert_eq!(time.search_moves.len(), 2);
    }

    #[test]
    fn tempboard() {
        // should be e1g1
        let old_str = "position startpos moves e2e4 d7d5 e4d5 d8d5 g1f3 d5e4 f1e2 c7c6 e1g1";
        // e8c8
        let args: Vec<&str> = old_str.split_whitespace().collect();
        position_parse_board(&args[1..]).unwrap();
    }
}
