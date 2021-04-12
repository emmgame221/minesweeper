use rand;
use rand::Rng;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Digit {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl Digit {
    pub fn to_int(d: Digit) -> i32 {
        use Digit::*;
        match d {
            Zero => 0,
            One => 1,
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8,
            Nine => 9,
        }
    }

    pub fn from_int<T: Into<usize>>(i: T) -> Digit {
        use Digit::*;
        let i: usize = i.into();
        match i {
            0 => Zero,
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            6 => Six,
            7 => Seven,
            8 => Eight,
            9 => Nine,
            _ => panic!("Digits can only be created from a single digit number"),
        }
    }
}

impl Display for Digit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Digit::*;
        match self {
            Zero => write!(f, "{}", 0),
            One => write!(f, "{}", 1),
            Two => write!(f, "{}", 2),
            Three => write!(f, "{}", 3),
            Four => write!(f, "{}", 4),
            Five => write!(f, "{}", 5),
            Six => write!(f, "{}", 6),
            Seven => write!(f, "{}", 7),
            Eight => write!(f, "{}", 8),
            Nine => write!(f, "{}", 9),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Tile {
    Safe(Digit),
    Mine,
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Digit::*;
        use Tile::*;
        match self {
            Safe(Zero) => write!(f, "|_|"),
            Safe(other) => write!(f, "|{}|", other),
            Mine => write!(f, "|*|"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TileDisplay {
    Hidden,
    Revealed,
    Flag,
    Question,
}

#[derive(Debug)]
pub struct Board {
    tiles: Vec<Vec<Tile>>,
    display: Vec<Vec<TileDisplay>>,
    pub width: usize,
    pub height: usize,
    pub mines: usize,
    any_revealed: bool,
}

impl Board {
    pub fn new(width: usize, height: usize, mines: usize) -> Board {
        let display: Vec<Vec<TileDisplay>> = vec![vec![TileDisplay::Hidden; height]; width];
        let mut tiles: Vec<Vec<Tile>> = vec![vec![Tile::Safe(Digit::Zero); height]; width];
        let mut num_mines = 0;
        let mut rng = rand::thread_rng();
        while num_mines < mines {
            let x = rng.gen_range(0, width);
            let y = rng.gen_range(0, height);
            if tiles[x][y] == Tile::Mine {
                continue;
            } else {
                tiles[x][y] = Tile::Mine;
                num_mines += 1;
            }
        }
        Self::update_digits(&mut tiles, height, width);

        Board {
            tiles,
            display,
            width,
            height,
            mines,
            any_revealed: false,
        }
    }

    fn update_digits(tiles: &mut Vec<Vec<Tile>>, height: usize, width: usize) {
        let mut counts: Vec<Vec<usize>> = vec![vec![0; height]; width];
        // Count Mines Adjacent to each safe tile
        for x in 0..width {
            for y in 0..height {
                if tiles[x][y] == Tile::Mine {
                    // We don't care how many mines are adjacent to a mine
                    continue;
                };
                if y > 0 {
                    // Check Up
                    counts[x][y] += match tiles[x][y - 1] {
                        Tile::Mine => 1,
                        _ => 0,
                    };
                }
                if y < height - 1 {
                    // Check Down
                    counts[x][y] += match tiles[x][y + 1] {
                        Tile::Mine => 1,
                        _ => 0,
                    };
                }
                if x < width - 1 && y < height - 1 {
                    // Check Down-Right
                    counts[x][y] += match tiles[x + 1][y + 1] {
                        Tile::Mine => 1,
                        _ => 0,
                    };
                }
                if x < width - 1 {
                    // Check Right
                    counts[x][y] += match tiles[x + 1][y] {
                        Tile::Mine => 1,
                        _ => 0,
                    };
                }
                if x < width - 1 && y > 0 {
                    // Check Up-Right
                    counts[x][y] += match tiles[x + 1][y - 1] {
                        Tile::Mine => 1,
                        _ => 0,
                    };
                }
                if x > 0 && y < height - 1 {
                    // Check Down-Left
                    counts[x][y] += match tiles[x - 1][y + 1] {
                        Tile::Mine => 1,
                        _ => 0,
                    };
                }
                if x > 0 {
                    // Check Left
                    counts[x][y] += match tiles[x - 1][y] {
                        Tile::Mine => 1,
                        _ => 0,
                    };
                }
                if x > 0 && y > 0 {
                    // Check Up-Left
                    counts[x][y] += match tiles[x - 1][y - 1] {
                        Tile::Mine => 1,
                        _ => 0,
                    };
                }
            }
        }
        // Update the tiles to reflect the counts
        for x in 0..width {
            for y in 0..height {
                if tiles[x][y] != Tile::Mine {
                    tiles[x][y] = Tile::Safe(Digit::from_int(counts[x][y]));
                }
            }
        }
    }

    pub fn reveal_all(&mut self) {
        for row in self.display.iter_mut() {
            for slot in row.iter_mut() {
                *slot = TileDisplay::Revealed;
            }
        }
    }

    pub fn reveal_at(&mut self, x: usize, y: usize) -> Result<Tile, &'static str> {
        if x >= self.width || y >= self.height {
            return Err("x and y must be less than width and height");
        } else {
            if !self.any_revealed {
                self.guarantee_zero(x, y);
            }
            self.any_revealed = true;
            self.display[x][y] = TileDisplay::Revealed;
            if self.tiles[x][y] == Tile::Safe(Digit::Zero) {
                self.reveal_adjacent(x, y).unwrap();
            }
            return Ok(self.tiles[x][y]);
        }
    }

    fn guarantee_zero(&mut self, x: usize, y: usize) {
        //Should be called the first time a tile is revealed
        //Moves any mines from (x, y) or adjacent to somewhere else at random
        assert!(x < self.width && y < self.height);
        let mut removed_mines = 0;

        // Check x, y and all adjacent tiles
        // If any is a Mine set it to a Safe(Zero) temporarily and increment removed_mines
        if self.tiles[x][y] == Tile::Mine {
            self.tiles[x][y] = Tile::Safe(Digit::Zero);
            removed_mines += 1;
        }
        if x > 0 {
            // Check Left
            if self.tiles[x - 1][y] == Tile::Mine {
                self.tiles[x - 1][y] = Tile::Safe(Digit::Zero);
                removed_mines += 1;
            }
            // Check Up-Left
            if y > 0 {
                if self.tiles[x - 1][y - 1] == Tile::Mine {
                    self.tiles[x - 1][y - 1] = Tile::Safe(Digit::Zero);
                    removed_mines += 1;
                }
            }
            // Check Down-Left
            if y < self.height - 1 {
                if self.tiles[x - 1][y + 1] == Tile::Mine {
                    self.tiles[x - 1][y + 1] = Tile::Safe(Digit::Zero);
                    removed_mines += 1;
                }
            }
        }
        if x < self.width - 1 {
            // Check Right
            if self.tiles[x + 1][y] == Tile::Mine {
                self.tiles[x + 1][y] = Tile::Safe(Digit::Zero);
                removed_mines += 1;
            }
            // Check Up-Right
            if y > 0 {
                if self.tiles[x + 1][y - 1] == Tile::Mine {
                    self.tiles[x + 1][y - 1] = Tile::Safe(Digit::Zero);
                    removed_mines += 1;
                }
            }
            // Check Down-Right
            if y < self.height - 1 {
                if self.tiles[x + 1][y + 1] == Tile::Mine {
                    self.tiles[x + 1][y + 1] = Tile::Safe(Digit::Zero);
                    removed_mines += 1;
                }
            }
        }
        if y > 0 {
            // Check Up
            if self.tiles[x][y - 1] == Tile::Mine {
                self.tiles[x][y - 1] = Tile::Safe(Digit::Zero);
                removed_mines += 1;
            }
        }
        if y < self.height - 1 {
            // Check Down
            if self.tiles[x][y + 1] == Tile::Mine {
                self.tiles[x][y + 1] = Tile::Safe(Digit::Zero);
                removed_mines += 1;
            }
        }
        assert!(removed_mines <= self.mines);
        // Reinsert the removed mines at random locations that aren't adjacent to or at x, y
        let x_range: (usize, usize, usize, usize);
        if x > 0 {
            x_range = (0, x - 1, x + 2, self.width);
        } else {
            x_range = (0, 1, x + 2, self.width);
        }
        let y_range: (usize, usize, usize, usize);
        if y > 0 {
            y_range = (0, y - 1, y + 2, self.height);
        } else {
            y_range = (0, 1, y + 2, self.height);
        }
        while removed_mines > 0 {
            let mut rng = rand::thread_rng();
            let mine_x: usize;
            let mine_y: usize;
            if x > 0 && x + 2 < self.width {
                if rng.gen_bool(0.5) {
                    // use the left side range
                    mine_x = rng.gen_range(x_range.0, x_range.1);
                } else {
                    // use the right side range
                    mine_x = rng.gen_range(x_range.2, x_range.3);
                }
            } else if x > 0 {
                // we can only use the left range
                mine_x = rng.gen_range(x_range.0, x_range.1);
            } else if x + 2 < self.width {
                // we can only use the right range
                mine_x = rng.gen_range(x_range.2, x_range.3);
            } else {
                //uh there's nowhere to put the mines abort!
                break;
            }
            if y > 0 && y + 2 < self.height {
                if rng.gen_bool(0.5) {
                    // use the top side range
                    mine_y = rng.gen_range(y_range.0, y_range.1);
                } else {
                    // use the bottom side range
                    mine_y = rng.gen_range(y_range.2, y_range.3);
                }
            } else if y > 0 {
                // we can only use the top range
                mine_y = rng.gen_range(y_range.0, y_range.1);
            } else if y + 2 < self.height {
                // we can only use the bottom range
                mine_y = rng.gen_range(y_range.2, y_range.3);
            } else {
                //uh there's nowhere to put the mines abort!
                break;
            }
            if self.tiles[mine_x][mine_y] != Tile::Mine {
                self.tiles[mine_x][mine_y] = Tile::Mine;
                removed_mines -= 1;
            }
        }

        // Reinitialize the digits of the entire board because a bunch of them are probably wrong now.
        Self::update_digits(&mut self.tiles, self.height, self.width);
    }

    /// Reveals all adjacent tiles. returns true if a mine was hit or false if not
    pub fn reveal_adjacent(&mut self, x: usize, y: usize) -> Result<bool, &'static str> {
        if !(self.display[x][y] == TileDisplay::Revealed) {
            return Err("Shouldn't try to reveal adjacent to unrevealed tile");
        }
        if x > 0 {
            // Check Left
            if self.display[x - 1][y] == TileDisplay::Hidden {
                if let Ok(tile) = self.reveal_at(x - 1, y) {
                    if tile == Tile::Mine {
                        return Ok(true);
                    }
                }
            }
            if y < self.height - 1 {
                // Check Down-Left
                if self.display[x - 1][y + 1] == TileDisplay::Hidden {
                    if let Ok(tile) = self.reveal_at(x - 1, y + 1) {
                        if tile == Tile::Mine {
                            return Ok(true);
                        }
                    }
                }
            }
            if y > 0 {
                // Check Up-Left
                if self.display[x - 1][y - 1] == TileDisplay::Hidden {
                    if let Ok(tile) = self.reveal_at(x - 1, y - 1) {
                        if tile == Tile::Mine {
                            return Ok(true);
                        }
                    }
                }
            }
        }
        if x < self.width - 1 {
            // Check Right
            if self.display[x + 1][y] == TileDisplay::Hidden {
                if let Ok(tile) = self.reveal_at(x + 1, y) {
                    if tile == Tile::Mine {
                        return Ok(true);
                    }
                }
            }
            if y < self.height - 1 {
                // Check Down-Right
                if self.display[x + 1][y + 1] == TileDisplay::Hidden {
                    if let Ok(tile) = self.reveal_at(x + 1, y + 1) {
                        if tile == Tile::Mine {
                            return Ok(true);
                        }
                    }
                }
            }
            if y > 0 {
                // Check Up-Right
                if self.display[x + 1][y - 1] == TileDisplay::Hidden {
                    if let Ok(tile) = self.reveal_at(x + 1, y - 1) {
                        if tile == Tile::Mine {
                            return Ok(true);
                        }
                    }
                }
            }
        }
        if y < self.height - 1 {
            // Check Up
            if self.display[x][y + 1] == TileDisplay::Hidden {
                if let Ok(tile) = self.reveal_at(x, y + 1) {
                    if tile == Tile::Mine {
                        return Ok(true);
                    }
                }
            }
        }
        if y > 0 {
            // Check Down
            if self.display[x][y - 1] == TileDisplay::Hidden {
                if let Ok(tile) = self.reveal_at(x, y - 1) {
                    if tile == Tile::Mine {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    pub fn toggle_display_at(&mut self, x: usize, y: usize) -> Result<TileDisplay, String> {
        if x >= self.width || y >= self.height {
            return Err(format!(
                "Index out of bounds: x is {}, y is {}, width is {}, height is {}",
                x, y, self.width, self.height
            ));
        }
        let next = match self.display[x][y] {
            TileDisplay::Hidden => TileDisplay::Flag,
            TileDisplay::Flag => TileDisplay::Question,
            TileDisplay::Question => TileDisplay::Hidden,
            TileDisplay::Revealed => TileDisplay::Revealed,
        };
        self.display[x][y] = next;
        Ok(self.display[x][y])
    }

    pub fn get_display_at(&mut self, x: usize, y: usize) -> Result<TileDisplay, String> {
        if x >= self.width {
            return Err(format!("x must be less than {}; it was {}", self.width, x));
        }
        if y >= self.height {
            return Err(format!("y must be less than {}; it was {}", self.height, y));
        }
        Ok(self.display[x][y])
    }

    pub fn get_tile_at(&mut self, x: usize, y: usize) -> Result<Tile, String> {
        if x >= self.width {
            return Err(format!("x must be less than {}; it was {}", self.width, x));
        }
        if y >= self.height {
            return Err(format!("y must be less than {}; it was {}", self.height, y));
        }
        Ok(self.tiles[x][y])
    }

    pub fn check_victory(&mut self) -> bool {
        for x in 0..self.width {
            for y in 0..self.height {
                match self.tiles[x][y] {
                    Tile::Safe(_) => {
                        if self.display[x][y] != TileDisplay::Revealed {
                            return false;
                        }
                    }
                    _ => (),
                }
            }
        }
        true
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "")?;
        write!(f, "   ")?;
        if self.width < 11 {
            for i in 0..self.width {
                write!(f, " {} ", i)?;
            }
        } else {
            for i in 0..10 {
                write!(f, " {} ", i)?;
            }
            for i in 10..self.width {
                write!(f, " {}", i)?;
            }
        }
        writeln!(f, "")?;
        for y in 0..self.height {
            write!(f, "{:2} ", y)?;
            for x in 0..self.width {
                match self.display[x][y] {
                    TileDisplay::Revealed => write!(f, "{}", self.tiles[x][y])?,
                    TileDisplay::Hidden => write!(f, "| |")?,
                    TileDisplay::Flag => write!(f, "|!|")?,
                    TileDisplay::Question => write!(f, "|?|")?,
                };
            }
            writeln!(f, "")?;
        }
        write!(f, "")
    }
}
