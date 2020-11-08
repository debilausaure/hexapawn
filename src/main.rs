#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    BlackPawn,
    WhitePawn,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    White,
    Black,
}

impl Side {
    pub fn switch (self) -> Self {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bound {
    Exact,
    MaybeBetter,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Score {
    score : isize,
    bound : Bound,
}

impl Score {
    pub fn new(score : isize, bound : Bound) -> Self {
        Self {
            score : score,
            bound : bound,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Board ([[Cell ; COLUMNS] ; ROWS]);

impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut zobrist_hash : u64 = 0;
        for (row_index, row) in self.0.iter().enumerate() {
            for (column_index, cell) in row.iter().enumerate() {
                match cell {
                    Cell::WhitePawn => zobrist_hash ^= ZOBRIST_CELL_SIGNATURES[0][row_index][column_index],
                    Cell::BlackPawn => zobrist_hash ^= ZOBRIST_CELL_SIGNATURES[1][row_index][column_index],
                    _ => (),
                }
            }
        }
        state.write_u64(zobrist_hash);
    }
}

pub fn better_than(a : isize, b : isize) -> bool {
    if b > 0 {
        if a > 0 && b >= a {
            return true;
        }
    } else {
        if a > 0 {
            return true;
        } else {
            if b >= a {
                return true;
            }
        }
    }
    return false;
}


pub fn best(a : isize, b : isize) -> isize {
    if b > 0 {
        if a > 0 && b > a {
            return a;
        }
    } else {
        if a > 0 {
            return a;
        } else {
            if b > a {
                return a;
            }
        }
    }
    return b;
}

impl Board {
    pub fn new_initial_board() -> Self {
        //declare unitialized array
        let mut board : [[MaybeUninit<Cell> ; COLUMNS ] ; ROWS] = unsafe {
            MaybeUninit::uninit().assume_init()
        };

        for (i, row) in board.iter_mut().enumerate() {
            for cell in row.iter_mut() {
                *cell = match i {
                    0 =>                  MaybeUninit::new(Cell::BlackPawn),
                    i if i == ROWS - 1 => MaybeUninit::new(Cell::WhitePawn),
                    _ =>                  MaybeUninit::new(Cell::Empty),
                };
            }
        }

        // change the type to match the initialized contents
        unsafe { mem::transmute::<_, Self>(board) }
    }

    pub fn display(&self, side : Side) {
        print!("┏");
        for _ in 0..(COLUMNS - 1) {
            print!("━━━┳");
        }
        println!("━━━┓");
        let rows = self.0.iter();
        if side == Side::White {
            for (j, row) in rows.enumerate() {
                print!("┃");
                for cell in row.iter().rev() {
                    match cell {
                        Cell::Empty     => print!("   ┃"),
                        Cell::BlackPawn => print!(" ◇ ┃"),
                        Cell::WhitePawn => print!(" ◆ ┃"),
                    };
                }
                println!();
                if j < (ROWS-1) {
                    print!("┣");
                    for _ in 0..(COLUMNS-1) {
                        print!("━━━╋");
                    }
                    println!("━━━┫");
                }
                else {
                    print!("┗");
                    for _ in 0..(COLUMNS-1) {
                        print!("━━━┻");
                    }
                    println!("━━━┛");
                }
            }
        } else {
            for (j, row) in rows.rev().enumerate() {
                print!("┃");
                for cell in row.iter().rev() {
                    match cell {
                        Cell::Empty     => print!("   ┃"),
                        Cell::BlackPawn => print!(" ◇ ┃"),
                        Cell::WhitePawn => print!(" ◆ ┃"),
                    };
                }
                println!();
                if j < (ROWS-1) {
                    print!("┣");
                    for _ in 0..(COLUMNS-1) {
                        print!("━━━╋");
                    }
                    println!("━━━┫");
                }
                else {
                    print!("┗");
                    for _ in 0..(COLUMNS-1) {
                        print!("━━━┻");
                    }
                    println!("━━━┛");
                }
            }

        }
    }

    pub fn set_cell(&mut self, row : usize, column : usize, cell_type : Cell) {
        self.0[row][column] = cell_type;
    }
    
    pub fn gen_next_moves(&self, side : Side) -> Vec<Board> {
        let mut possible_next_boards : Vec<Board> = Vec::new();
        match side {
            Side::Black => {
                // it is useless to check the last line, 
                for i in 0..(ROWS-1) {
                    for j in 0..(COLUMNS) {
                        match self.0[i][j] {
                            Cell::BlackPawn => {
                                //if the cell in front of the pawn is empty
                                if self.0[i+1][j] == Cell::Empty {
                                    // clone the current board and move the pawn forward
                                    let mut possible_next_board = self.clone();
                                    possible_next_board.set_cell(i, j, Cell::Empty);
                                    possible_next_board.set_cell(i+1, j, Cell::BlackPawn);
                                    possible_next_boards.push(possible_next_board);
                                }
                                //if there is a white pawn on the left-diagonal cell
                                if j != 0 && self.0[i+1][j-1] == Cell::WhitePawn {
                                    // clone the current board and move the pawn
                                    let mut possible_next_board = self.clone();
                                    possible_next_board.set_cell(i, j, Cell::Empty);
                                    possible_next_board.set_cell(i+1, j-1, Cell::BlackPawn);
                                    possible_next_boards.push(possible_next_board);
                                }
                                //if there is a white pawn on the right-diagonal cell
                                if (j != (COLUMNS-1)) && self.0[i+1][j+1] == Cell::WhitePawn {
                                    //clone the current board and move the pawn
                                    let mut possible_next_board = self.clone();
                                    possible_next_board.set_cell(i, j, Cell::Empty);
                                    possible_next_board.set_cell(i+1, j+1, Cell::BlackPawn);
                                    possible_next_boards.push(possible_next_board);
                                }
                            },
                            _ => continue,
                        };
                    }
                }
            },
            Side::White => {
                // it is useless to check the first line, 
                for i in 1..(ROWS) {
                    for j in 0..(COLUMNS) {
                        match self.0[i][j] {
                            Cell::WhitePawn => {
                                //if the cell in front of the pawn is empty
                                if self.0[i-1][j] == Cell::Empty {
                                    // clone the current board and move the pawn forward
                                    let mut possible_next_board = self.clone();
                                    possible_next_board.set_cell(i, j, Cell::Empty);
                                    possible_next_board.set_cell(i-1, j, Cell::WhitePawn);
                                    possible_next_boards.push(possible_next_board);
                                }
                                //if there is a black pawn on the left-diagonal cell
                                if j != 0 && self.0[i-1][j-1] == Cell::BlackPawn {
                                    // clone the current board and move the pawn
                                    let mut possible_next_board = self.clone();
                                    possible_next_board.set_cell(i, j, Cell::Empty);
                                    possible_next_board.set_cell(i-1, j-1, Cell::WhitePawn);
                                    possible_next_boards.push(possible_next_board);
                                }
                                //if there is a black pawn on the right-diagonal cell
                                if (j != (COLUMNS-1)) && self.0[i-1][j+1] == Cell::BlackPawn {
                                    //clone the current board and move the pawn
                                    let mut possible_next_board = self.clone();
                                    possible_next_board.set_cell(i, j, Cell::Empty);
                                    possible_next_board.set_cell(i-1, j+1, Cell::WhitePawn);
                                    possible_next_boards.push(possible_next_board);
                                }
                            },
                            _ => continue,
                        }
                    }
                }   
            },
        }
        possible_next_boards
    }

    pub fn get_score_minmax(&self, side : Side, transposition_table : &mut TranspositionTable) -> isize {
        match transposition_table.get(self) {
            None => (),
            Some(score) => return score.score,
        }

        // if opponent pawn on our starting row
        if self.opponent_pawn_on_last_line(side) {
            transposition_table.insert(*self, Score::new(0, Bound::Exact));
            return 0;
        }

        let next_boards = self.gen_next_moves(side);
        // if no legal moves left
        if next_boards.is_empty() {
            transposition_table.insert(*self, Score::new(0, Bound::Exact));
            return 0;
        }
        let mut best_score : isize = 0;
        for next_board in next_boards {
            let mut next_score = next_board.get_score_minmax(side.switch(), transposition_table);
            if next_score > 0 {
                next_score = -(next_score+1);
            } else {
                next_score = -(next_score-1);
            }

            best_score = best(best_score, next_score);
        }
        transposition_table.insert(*self, Score::new(best_score, Bound::Exact));
        return best_score;
    }

    pub fn get_score_alpha_beta(&self, mut alpha : isize, beta : isize, side : Side, transposition_table : &mut TranspositionTable) -> isize {
        // look inside the transposition_table if we already have computed the board's value
        match transposition_table.get(self) {
            None => (),
            Some(score) => {
                match score {
                    Score {bound: Bound::Exact,       ..} => return score.score,
                    Score {bound: Bound::MaybeBetter, ..} => alpha = best(alpha, score.score),
                };

                if better_than(alpha, beta) {
                    //println!{"transition table partial result alpha cut"};
                    return score.score
                }
            },
        };

        // we will need to insert a new score into the transposition table
        
        // if opponent pawn on our starting row
        if self.opponent_pawn_on_last_line(side) {
            transposition_table.insert(*self, Score::new(0, Bound::Exact));
            //self.display(side);
            //println!{"0"};
            return 0;
        }

        let next_boards = self.gen_next_moves(side);
        // if no legal moves left
        if next_boards.is_empty() {
            transposition_table.insert(*self, Score::new(0, Bound::Exact));
            //self.display(side);
            //println!{"0"};
            return 0;
        }

        let mut best_score : isize = 0;
        let mut exhaustive_search : bool = true;
        let bound : Bound;
        for next_board in next_boards {
            let mut next_score = next_board.get_score_alpha_beta(-beta+1, -alpha+1, side.switch(), transposition_table);
            if next_score > 0 {
                next_score = -(next_score+1);
            } else {
                next_score = -(next_score-1);
            }

            if better_than(next_score, best_score) {
                best_score = next_score;
                alpha = next_score;
            }

            if better_than(alpha, beta) {
                exhaustive_search = false;
                //println!{"alpha cut ({} better than {})", alpha , beta};
                break;
            }
        }

        if exhaustive_search {
            bound = Bound::Exact;
        } else {
            bound = Bound::MaybeBetter;
        }

        transposition_table.insert(*self, Score::new(best_score, bound));
        //self.display(side);
        //println!{"{}", best_score};
        return best_score;
    }

    pub fn opponent_pawn_on_last_line (&self, side : Side) -> bool {
        match side {
            Side::White => {
                for cell in self.0[ROWS-1].iter() {
                    if *cell == Cell::BlackPawn {
                        return true;
                    }
                }  
            },
            Side::Black => {
                for cell in self.0[0].iter() {
                    if *cell == Cell::WhitePawn {
                        return true;
                    }
                }
            },
        };
        false
    }
}

type TranspositionTable = HashMap<Board, Score>;

#[macro_use]
extern crate lazy_static;

use std::mem;
use std::mem::MaybeUninit;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use rand::Rng;
//use rand::prelude::*;
use rand_pcg::Pcg64Mcg;

const ROWS : usize = 4;
const COLUMNS : usize = 4;
const RNG_SEED : u128 =  0xbadd0990d15ea5e5;

lazy_static! {
    static ref ZOBRIST_CELL_SIGNATURES : [[[u64 ; COLUMNS]; ROWS]; 2] = {
        //declare unitialized array
        let cell_sigs : [[[MaybeUninit<u64> ; COLUMNS ] ; ROWS ] ; 2] = unsafe { 
            MaybeUninit::uninit().assume_init()
        };

        let mut rng = Pcg64Mcg::new(RNG_SEED);
        let mut cell_sigs = unsafe { mem::transmute::<_, [u64 ; ROWS * COLUMNS * 2]>(cell_sigs) };
        rng.fill(&mut cell_sigs[..]);

        // change the type to match the initialized contents
        unsafe { mem::transmute::<_, [[[u64 ; COLUMNS]; ROWS]; 2]>(cell_sigs) }
    };
}

fn main() {
    //TODO : parse command line arguments
    let mut transposition_table = TranspositionTable::new();

    let initial_board = Board::new_initial_board();
    initial_board.display(Side::White);
    let score = initial_board.get_score_minmax(Side::White, &mut transposition_table);
    //let score = initial_board.get_score_alpha_beta(0, 1, Side::White, &mut transposition_table);
    println!("{}", score);

//    let next_boards = initial_board.gen_next_moves(Side::White);
//    for board in next_boards {
//        board.display();
//    }
}

