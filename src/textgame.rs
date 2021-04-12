use crate::minesweeper::*;
use std::io;

type Difficulty = (usize, usize, usize);

const EASY: Difficulty = (10, 10, 10);
const MEDIUM: Difficulty = (15, 15, 30);
const HARD: Difficulty = (30, 15, 99);

pub struct TextGame {
    board: Board,
    state: GameState,
}

#[derive(PartialEq)]
enum GameState {
    Run,
    End,
}

impl TextGame {
    pub fn new() -> TextGame {
        println!("To use a predefined difficulty enter Easy, Medium or Hard");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line.");
        let input = input.trim().to_lowercase();
        if input == "easy" {
            return TextGame::_new(EASY.0, EASY.1, EASY.2);
        } else if input == "medium" {
            return TextGame::_new(MEDIUM.0, MEDIUM.1, MEDIUM.2);
        } else if input == "hard" {
            return TextGame::_new(HARD.0, HARD.1, HARD.2);
        }
        let width: usize;
        let height: usize;
        let mines: usize;

        loop {
            println!("Enter the width of the board: ");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line.");
            let input: Result<usize, _> = input.trim().parse();
            if input.is_ok() {
                width = input.unwrap();
                break;
            }
            println!("You must enter a whole number.");
        }
        loop {
            println!("Enter the height of the board: ");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line.");
            let input: Result<usize, _> = input.trim().parse();
            if input.is_ok() {
                height = input.unwrap();
                break;
            }
            println!("You must enter a whole number.");
        }
        loop {
            println!("Enter the number of mines: ");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line.");
            let input: Result<usize, _> = input.trim().parse();
            if input.is_ok() {
                let input = input.unwrap();
                if input > width * height {
                    println!("There can't be more mines than tiles!");
                    continue;
                }
                mines = input;
                break;
            }
            println!("You must enter a whole number.");
        }
        TextGame::_new(width, height, mines)
    }

    fn _new(width: usize, height: usize, mines: usize) -> TextGame {
        let board = Board::new(width, height, mines);

        TextGame {
            board,
            state: GameState::Run,
        }
    }

    fn check(&mut self, x: usize, y: usize) {
        let result = self.board.reveal_at(x, y);
        match result {
            Ok(tile) => {
                if tile == Tile::Mine {
                    self.board.reveal_all();
                    self.game_over(true);
                }
            }
            Err(message) => panic!(message),
        }
    }

    fn toggle(&mut self, x: usize, y: usize) {
        self.board.toggle_display_at(x, y).unwrap();
    }

    fn flag(&mut self, x: usize, y: usize) {
        let cur_display = self
            .board
            .get_display_at(x, y)
            .expect("Tried to flag an invalid tile");
        match cur_display {
            TileDisplay::Hidden => {
                self.board.toggle_display_at(x, y).unwrap();
            }
            TileDisplay::Question => {
                self.board.toggle_display_at(x, y).unwrap();
                self.board.toggle_display_at(x, y).unwrap();
            }
            _ => (),
        };
    }

    fn chord(&mut self, x: usize, y: usize) {
        let display_at = self
            .board
            .get_display_at(x, y)
            .expect("Tried to chord at invalid tile.");
        if display_at != TileDisplay::Revealed {
            println!("Cannot chord from a tile that is not revealed.");
            return;
        };
        let tile = self.board.reveal_at(x, y).unwrap();
        let req_flags = match tile {
            Tile::Mine => panic!("How did we get here?(Trying to chord a mine)"),
            Tile::Safe(digit) => Digit::to_int(digit),
        };
        let mut count_flags = 0;
        count_flags += match self.board.get_display_at(x, y + 1) {
            Ok(display) => match display {
                TileDisplay::Flag => 1,
                _ => 0,
            },
            Err(_) => 0,
        };
        if y > 0 {
            count_flags += match self.board.get_display_at(x + 1, y - 1) {
                Ok(display) => match display {
                    TileDisplay::Flag => 1,
                    _ => 0,
                },
                Err(_) => 0,
            };
            count_flags += match self.board.get_display_at(x, y - 1) {
                Ok(display) => match display {
                    TileDisplay::Flag => 1,
                    _ => 0,
                },
                Err(_) => 0,
            };
        }
        count_flags += match self.board.get_display_at(x + 1, y) {
            Ok(display) => match display {
                TileDisplay::Flag => 1,
                _ => 0,
            },
            Err(_) => 0,
        };
        count_flags += match self.board.get_display_at(x + 1, y + 1) {
            Ok(display) => match display {
                TileDisplay::Flag => 1,
                _ => 0,
            },
            Err(_) => 0,
        };
        if x > 0 {
            if y > 0 {
                count_flags += match self.board.get_display_at(x - 1, y - 1) {
                    Ok(display) => match display {
                        TileDisplay::Flag => 1,
                        _ => 0,
                    },
                    Err(_) => 0,
                };
            }
            count_flags += match self.board.get_display_at(x - 1, y) {
                Ok(display) => match display {
                    TileDisplay::Flag => 1,
                    _ => 0,
                },
                Err(_) => 0,
            };
            count_flags += match self.board.get_display_at(x - 1, y + 1) {
                Ok(display) => match display {
                    TileDisplay::Flag => 1,
                    _ => 0,
                },
                Err(_) => 0,
            };
        }
        if count_flags == req_flags {
            self.board.reveal_adjacent(x, y).unwrap();
        } else {
            println!("Chording is only allowed when there are exactly the right number of flags adjacent to a tile.");
        }
    }

    fn game_over(&mut self, lose: bool) {
        if lose {
            println!("You Lose!");
            self.board.reveal_all();
        } else {
            println!("You Win!");
        }
        println!("{}", self.board);
        self.state = GameState::End;
    }

    fn print_menu(&self) {
        println!("Menu: ");
        println!("All capital letters are treated as lowercase");
        println!("Replace x and y with numbers - they represent coordinates");
        println!("Check square - 'check x y' or 'c x y'");
        println!("Toggle square - 'toggle x y' or 't x y'");
        println!("Flag square - 'flag x y' or 'f x y'");
        println!("Chord at square - 'chord x y' or 'ch x y'");
        println!("Show this menu - 'menu' or 'm'");
        println!("Quit game - 'quit' or 'q'");
    }

    pub fn main_loop(&mut self) {
        loop {
            if self.state == GameState::End {
                break;
            }
            println!("{}", self.board);
            println!("Enter your selection(menu for options): ");
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line.");
            let input = input.to_lowercase();
            let input: Vec<&str> = input.split_whitespace().collect();
            if input.len() < 1 {
                println!("You must select an option.");
                continue;
            }
            let option = input[0];
            if option == "m" || option == "menu" {
                self.print_menu();
                continue;
            } else if option == "q" || option == "quit" {
                self.game_over(true);
                continue;
            }
            if input.len() < 3 {
                println!("Your option require 2 arguments or is invalid.");
                continue;
            }
            let x: usize = match input[1].parse() {
                Ok(val) => {
                    if val >= self.board.width {
                        println!("x must be less than {}", self.board.width);
                        continue;
                    };
                    val
                }
                Err(_) => {
                    println!("x must be a whole number");
                    continue;
                }
            };
            let y: usize = match input[2].parse() {
                Ok(val) => {
                    if val >= self.board.height {
                        println!("y must be less than {}", self.board.height);
                        continue;
                    };
                    val
                }
                Err(_) => {
                    println!("y must be a whole number");
                    continue;
                }
            };
            if option == "c" || option == "check" {
                self.check(x, y);
            } else if option == "t" || option == "toggle" {
                self.toggle(x, y);
            } else if option == "f" || option == "flag" {
                self.flag(x, y);
            } else if option == "ch" || option == "chord" {
                self.chord(x, y);
            }
            if self.state != GameState::End && self.board.check_victory() {
                self.game_over(false);
            }
        }
    }
}
