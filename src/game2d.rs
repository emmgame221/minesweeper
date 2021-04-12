use crate::minesweeper::*;

use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::filesystem;
use ggez::graphics::{self, Color, DrawMode, Rect};
use ggez::nalgebra::Point2;
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use std::env;
use std::fmt::Display;
use std::io::{Read, Write};
use std::path;

#[derive(Debug)]
pub struct MinesweeperGame {
    board: Board,
    diff: DifficultySetting,
    custom_diff: Difficulty,
    state: GameState,
    unflagged_mines: i32,
    timer: f64,
    menu: MainMenu,
    popup: Option<PopupMenu>,
    hidden_image: graphics::Image,
    flag_image: graphics::Image,
    question_image: graphics::Image,
    zero_image: graphics::Image,
    one_image: graphics::Image,
    two_image: graphics::Image,
    three_image: graphics::Image,
    four_image: graphics::Image,
    five_image: graphics::Image,
    six_image: graphics::Image,
    seven_image: graphics::Image,
    eight_image: graphics::Image,
    mine_image: graphics::Image,
    best_easy: u16,
    best_medium: u16,
    best_hard: u16,
    time_since_click: f64,
}

#[derive(PartialEq, Debug)]
enum GameState {
    Default,
    Menu,
    Updated,
    Loss,
    Win,
}
#[derive(PartialEq, Copy, Clone, Debug)]
enum DifficultySetting {
    Easy,
    Medium,
    Hard,
    Custom,
}

impl Display for DifficultySetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DifficultySetting::*;
        match self {
            Easy => write!(f, "easy"),
            Medium => write!(f, "medium"),
            Hard => write!(f, "hard"),
            Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug)]
struct Difficulty(usize, usize, usize);

impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.0, self.1, self.2)
    }
}

const EASY: Difficulty = Difficulty(9, 9, 10);
const MEDIUM: Difficulty = Difficulty(16, 16, 40);
const HARD: Difficulty = Difficulty(30, 16, 99);

const MAX_WIDTH: usize = 30;
const MAX_HEIGHT: usize = 24;
const MIN_WIDTH: usize = 9;
const MIN_HEIGHT: usize = 9;

const TILE_SIZE: f32 = 25.;

const DOUBLE_CLICK_TIME: f64 = 0.1;

const BUTTON_BG: Color = Color {
    r: 0.5,
    b: 0.5,
    g: 0.5,
    a: 1.,
};
const TEXT_BG: Color = Color {
    r: 0.8,
    b: 0.8,
    g: 0.8,
    a: 1.,
};

const DEFAULT_CONFIG: &'static str = "easy
24,16,50
999
999
999";

impl MinesweeperGame {
    pub fn new(ctx: &mut Context) -> GameResult<MinesweeperGame> {
        let mut config = String::new();
        {
            if !filesystem::exists(ctx, "/config") {
                println!("Didn't find existing config file");
                let mut new_file = filesystem::create(ctx, "/config")
                    .expect("Unable to create config file at /config");
                write!(new_file, "{}", DEFAULT_CONFIG).unwrap();
            }
            let mut file = filesystem::open_options(
                ctx,
                "/config",
                filesystem::OpenOptions::new().read(true),
            )?;
            file.read_to_string(&mut config)?;
        }
        let config: Vec<&str> = config.trim().split('\n').collect();
        let config_diff = config[0].trim();
        let board: Board;
        let diff: DifficultySetting;
        let custom: Vec<usize> = config[1]
            .split(',')
            .map(|s| s.trim().parse().unwrap())
            .collect();
        let custom_diff = Difficulty(custom[0], custom[1], custom[2]);
        let best_easy: u16 = config[2].trim().parse().unwrap();
        let best_medium: u16 = config[3].trim().parse().unwrap();
        let best_hard: u16 = config[4].trim().parse().unwrap();
        if config_diff == "easy" {
            board = Board::new(EASY.0, EASY.1, EASY.2);
            diff = DifficultySetting::Easy;
        } else if config_diff == "medium" {
            board = Board::new(MEDIUM.0, MEDIUM.1, MEDIUM.2);
            diff = DifficultySetting::Medium;
        } else if config_diff == "hard" {
            board = Board::new(HARD.0, HARD.1, HARD.2);
            diff = DifficultySetting::Hard
        } else if config_diff == "custom" {
            board = Board::new(custom_diff.0, custom_diff.1, custom_diff.2);
            diff = DifficultySetting::Custom;
        } else {
            board = Board::new(EASY.0, EASY.1, EASY.2);
            diff = DifficultySetting::Easy;
        }

        let hidden_image = graphics::Image::new(ctx, "/hidden.png")?;
        let flag_image = graphics::Image::new(ctx, "/flag.png")?;
        let question_image = graphics::Image::new(ctx, "/question.png")?;
        let zero_image = graphics::Image::new(ctx, "/empty.png")?;
        let one_image = graphics::Image::new(ctx, "/one.png")?;
        let two_image = graphics::Image::new(ctx, "/two.png")?;
        let three_image = graphics::Image::new(ctx, "/three.png")?;
        let four_image = graphics::Image::new(ctx, "/four.png")?;
        let five_image = graphics::Image::new(ctx, "/five.png")?;
        let six_image = graphics::Image::new(ctx, "/six.png")?;
        let seven_image = graphics::Image::new(ctx, "/seven.png")?;
        let eight_image = graphics::Image::new(ctx, "/eight.png")?;
        let mine_image = graphics::Image::new(ctx, "/mine.png")?;

        let unflagged_mines = board.mines as i32;

        let menu = MainMenu::new(diff, Difficulty(24, 16, 50));

        let game = MinesweeperGame {
            board,
            diff,
            custom_diff,
            state: GameState::Updated,
            unflagged_mines,
            timer: 0.0,
            menu,
            popup: None,
            hidden_image,
            flag_image,
            question_image,
            zero_image,
            one_image,
            two_image,
            three_image,
            four_image,
            five_image,
            six_image,
            seven_image,
            eight_image,
            mine_image,
            best_easy,
            best_medium,
            best_hard,
            time_since_click: 1.0,
        };
        game.init_window_size(ctx)?;
        Ok(game)
    }

    fn init_window_size(&self, ctx: &mut Context) -> GameResult {
        set_window_size(
            ctx,
            (self.board.width as f32) * TILE_SIZE,
            ((self.board.height + 1) as f32) * TILE_SIZE,
        )
    }

    fn new_game(&mut self, ctx: &mut Context) -> GameResult {
        match self.diff {
            DifficultySetting::Custom => {
                self.board = Board::new(self.custom_diff.0, self.custom_diff.1, self.custom_diff.2)
            }
            DifficultySetting::Easy => self.board = Board::new(EASY.0, EASY.1, EASY.2),
            DifficultySetting::Medium => self.board = Board::new(MEDIUM.0, MEDIUM.1, MEDIUM.2),
            DifficultySetting::Hard => self.board = Board::new(HARD.0, HARD.1, HARD.2),
        }
        self.state = GameState::Updated;
        self.timer = 0.0;
        self.unflagged_mines = self.board.mines as i32;
        self.init_window_size(ctx)
    }

    fn check(&mut self, x: usize, y: usize) {
        let display = self.board.get_display_at(x, y);
        if display == Ok(TileDisplay::Hidden) {
            match self.board.reveal_at(x, y) {
                Ok(tile) => match tile {
                    Tile::Mine => self.state = GameState::Loss,
                    _ => self.state = GameState::Updated,
                },
                Err(message) => eprintln!("{}", message),
            }
        }
    }

    fn toggle(&mut self, x: usize, y: usize) {
        match self.board.toggle_display_at(x, y) {
            Err(message) => eprintln!("{}", message),
            Ok(display) => {
                if display == TileDisplay::Flag {
                    self.unflagged_mines -= 1;
                } else if display == TileDisplay::Question {
                    self.unflagged_mines += 1;
                }
                self.state = GameState::Updated;
            }
        }
    }

    fn chord(&mut self, x: usize, y: usize) {
        let req_flags = if let Ok(tile) = self.board.get_tile_at(x, y) {
            match tile {
                Tile::Mine => return (),
                Tile::Safe(digit) => Digit::to_int(digit),
            }
        } else {
            return ();
        };
        let mut count_adj_flags = 0;
        if x > 0 {
            // Check left
            count_adj_flags += match self.board.get_display_at(x - 1, y) {
                Ok(display) => match display {
                    TileDisplay::Flag => 1,
                    _ => 0,
                },
                _ => 0,
            };
            // Check Down-Left
            count_adj_flags += match self.board.get_display_at(x - 1, y + 1) {
                Ok(display) => match display {
                    TileDisplay::Flag => 1,
                    _ => 0,
                },
                _ => 0,
            };
            if y > 0 {
                // Check Up-Left
                count_adj_flags += match self.board.get_display_at(x - 1, y - 1) {
                    Ok(display) => match display {
                        TileDisplay::Flag => 1,
                        _ => 0,
                    },
                    _ => 0,
                };
            }
        }
        // Check Down-Right
        count_adj_flags += match self.board.get_display_at(x + 1, y + 1) {
            Ok(display) => match display {
                TileDisplay::Flag => 1,
                _ => 0,
            },
            _ => 0,
        };
        // Check Right
        count_adj_flags += match self.board.get_display_at(x + 1, y) {
            Ok(display) => match display {
                TileDisplay::Flag => 1,
                _ => 0,
            },
            _ => 0,
        };
        // Check Down
        count_adj_flags += match self.board.get_display_at(x, y + 1) {
            Ok(display) => match display {
                TileDisplay::Flag => 1,
                _ => 0,
            },
            _ => 0,
        };
        if y > 0 {
            // Check Up-Right
            count_adj_flags += match self.board.get_display_at(x + 1, y - 1) {
                Ok(display) => match display {
                    TileDisplay::Flag => 1,
                    _ => 0,
                },
                _ => 0,
            };
            // Check Up
            count_adj_flags += match self.board.get_display_at(x, y - 1) {
                Ok(display) => match display {
                    TileDisplay::Flag => 1,
                    _ => 0,
                },
                _ => 0,
            };
        }
        if count_adj_flags == req_flags {
            match self.board.reveal_adjacent(x, y) {
                Ok(hit_mine) => {
                    if hit_mine {
                        self.state = GameState::Loss;
                        self.board.reveal_all();
                        return;
                    }
                }
                Err(message) => eprintln!("{}", message),
            }
            self.state = GameState::Updated;
        }
    }

    fn draw_board(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);
        for x in 0..self.board.width {
            for y in 0..self.board.height {
                let dest = Point2::new((x as f32) * TILE_SIZE, (y as f32) * TILE_SIZE);
                let tile_display = self.board.get_display_at(x, y).unwrap();
                match tile_display {
                    TileDisplay::Revealed => {
                        let tile = self.board.get_tile_at(x, y).unwrap();
                        match tile {
                            Tile::Mine => {
                                graphics::draw(
                                    ctx,
                                    &self.mine_image,
                                    graphics::DrawParam::new().dest(dest),
                                )?;
                            }
                            Tile::Safe(Digit::Zero) => {
                                graphics::draw(
                                    ctx,
                                    &self.zero_image,
                                    graphics::DrawParam::new().dest(dest),
                                )?;
                            }
                            Tile::Safe(Digit::One) => {
                                graphics::draw(
                                    ctx,
                                    &self.one_image,
                                    graphics::DrawParam::new().dest(dest),
                                )?;
                            }
                            Tile::Safe(Digit::Two) => {
                                graphics::draw(
                                    ctx,
                                    &self.two_image,
                                    graphics::DrawParam::new().dest(dest),
                                )?;
                            }
                            Tile::Safe(Digit::Three) => {
                                graphics::draw(
                                    ctx,
                                    &self.three_image,
                                    graphics::DrawParam::new().dest(dest),
                                )?;
                            }
                            Tile::Safe(Digit::Four) => {
                                graphics::draw(
                                    ctx,
                                    &self.four_image,
                                    graphics::DrawParam::new().dest(dest),
                                )?;
                            }
                            Tile::Safe(Digit::Five) => {
                                graphics::draw(
                                    ctx,
                                    &self.five_image,
                                    graphics::DrawParam::new().dest(dest),
                                )?;
                            }
                            Tile::Safe(Digit::Six) => {
                                graphics::draw(
                                    ctx,
                                    &self.six_image,
                                    graphics::DrawParam::new().dest(dest),
                                )?;
                            }
                            Tile::Safe(Digit::Seven) => {
                                graphics::draw(
                                    ctx,
                                    &self.seven_image,
                                    graphics::DrawParam::new().dest(dest),
                                )?;
                            }
                            Tile::Safe(_) => {
                                graphics::draw(
                                    ctx,
                                    &self.eight_image,
                                    graphics::DrawParam::new().dest(dest),
                                )?;
                            }
                        }
                    }
                    TileDisplay::Hidden => {
                        graphics::draw(
                            ctx,
                            &self.hidden_image,
                            graphics::DrawParam::new().dest(dest),
                        )?;
                    }
                    TileDisplay::Flag => {
                        graphics::draw(
                            ctx,
                            &self.flag_image,
                            graphics::DrawParam::new().dest(dest),
                        )?;
                    }
                    TileDisplay::Question => {
                        graphics::draw(
                            ctx,
                            &self.question_image,
                            graphics::DrawParam::new().dest(dest),
                        )?;
                    }
                }
            }
        }
        let timer_string = format!("{}", self.timer as i32);
        let timer_text = graphics::Text::new(timer_string);
        graphics::draw(
            ctx,
            &timer_text,
            graphics::DrawParam::default()
                .dest(Point2::new(0., (self.board.height as f32) * TILE_SIZE + 5.))
                .color(graphics::BLACK),
        )?;
        let mines_string = format!("Mines: {}", self.unflagged_mines);
        let mines_text = graphics::Text::new(mines_string);
        let x = (self.board.width as f32) * TILE_SIZE - 100.;
        graphics::draw(
            ctx,
            &mines_text,
            graphics::DrawParam::default()
                .dest(Point2::new(x, (self.board.height as f32) * TILE_SIZE + 5.))
                .color(graphics::BLACK),
        )?;
        Ok(())
    }
}

impl EventHandler for MinesweeperGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.state == GameState::Default || self.state == GameState::Updated {
            self.timer += timer::delta(_ctx).as_secs_f64();
        }
        self.time_since_click += timer::delta(_ctx).as_secs_f64();
        if self.timer > 999. {
            self.timer = 999.
        }
        while timer::check_update_time(_ctx, 60) {
            ()
        }
        match self.state {
            GameState::Updated => {
                if self.board.check_victory() {
                    self.state = GameState::Win;
                }
            }
            GameState::Menu => (),
            GameState::Loss => {
                self.board.reveal_all();
            }
            GameState::Win => match self.diff {
                DifficultySetting::Easy => {
                    if self.timer < self.best_easy as f64 {
                        self.best_easy = self.timer as u16;
                    }
                }
                DifficultySetting::Medium => {
                    if self.timer < self.best_medium as f64 {
                        self.best_medium = self.timer as u16;
                    }
                }
                DifficultySetting::Hard => {
                    if self.timer < self.best_hard as f64 {
                        self.best_hard = self.timer as u16;
                    }
                }
                _ => (),
            },
            GameState::Default => (),
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        match self.state {
            GameState::Updated => {
                self.draw_board(ctx)?;
                self.state = GameState::Default;
            }
            GameState::Win => {
                self.draw_board(ctx)?;
                if let Some(menu) = &self.popup {
                    menu.draw(ctx)?;
                } else {
                    let menu = PopupMenu::new("You Win!", "Restart", "Quit", 10., 10.);
                    menu.draw(ctx)?;
                    self.popup = Some(menu);
                }
            }
            GameState::Loss => {
                self.draw_board(ctx)?;
                if let Some(menu) = &self.popup {
                    menu.draw(ctx)?;
                } else {
                    let menu = PopupMenu::new("You Lose!", "Retry?", "Quit", 10., 10.);
                    menu.draw(ctx)?;
                    self.popup = Some(menu);
                }

            }
            GameState::Menu => {
                self.menu.draw(ctx)?;
            }
            _ => {
                self.draw_board(ctx)?;
            }
        }
        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match self.state {
            GameState::Default => {
                let x = (x / TILE_SIZE) as usize;
                let y = (y / TILE_SIZE) as usize;
                match button {
                    MouseButton::Left => {
                        if self.time_since_click < DOUBLE_CLICK_TIME {
                            self.chord(x, y);
                        } else {
                            self.check(x, y);
                        }
                    }
                    MouseButton::Right => self.toggle(x, y),
                    MouseButton::Middle => self.chord(x, y),
                    _ => (),
                }
            }
            GameState::Win => {
                if button == MouseButton::Left {
                    if let Some(menu) = &self.popup {
                        let result = menu.mouse_button_down_event(x, y);
                        if result == 1 {
                            self.popup = None;
                            self.new_game(ctx).unwrap();
                        } else if result == 2 {
                            self.quit_event(ctx);
                            event::quit(ctx);
                        }
                    }
                }
            }
            GameState::Loss => {
                if button == MouseButton::Left {
                    if let Some(menu) = &self.popup {
                        let result = menu.mouse_button_down_event(x, y);
                        if result == 1 {
                            self.popup = None;
                            self.new_game(ctx).unwrap();
                        } else if result == 2 {
                            self.quit_event(ctx);
                            event::quit(ctx);
                        }
                    }
                }
            }
            GameState::Menu => {
                if let Some((diff, custom_diff)) =
                    self.menu.mouse_button_down_event(ctx, button, x, y)
                {
                    self.diff = diff;
                    self.custom_diff = custom_diff;
                    self.new_game(ctx).unwrap();
                }
            }
            _ => (),
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        if button == MouseButton::Left {
            self.time_since_click = 0.0;
        }
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match self.state {
            GameState::Menu => {
                if self.menu.key_down_event(ctx, keycode, _keymods, _repeat) {
                    self.state = GameState::Updated;
                }
            }
            _ => match keycode {
                KeyCode::Space => {
                    self.state = GameState::Menu;
                }
                KeyCode::Escape => {
                    self.quit_event(ctx);
                    event::quit(ctx);
                }
                _ => (),
            },
        }
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        {
            eprintln!("{}", self);
            eprintln!("Attempting to write to config file");
            let mut config_file =
                filesystem::create(_ctx, "/config").expect("Unable to create/open config file");
            writeln!(config_file, "{}", self.diff).unwrap();
            writeln!(config_file, "{}", self.custom_diff).unwrap();
            writeln!(config_file, "{}", self.best_easy).unwrap();
            writeln!(config_file, "{}", self.best_medium).unwrap();
            writeln!(config_file, "{}", self.best_hard).unwrap();
        }
        return false;
    }
}

impl Display for MinesweeperGame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Difficulty Setting: {} Custom Settings: {} Best Times: Easy: {} Medium: {} Hard {}",
            self.diff, self.custom_diff, self.best_easy, self.best_medium, self.best_hard
        )
    }
}

#[derive(Debug)]
struct MainMenu {
    header: &'static str,
    easy: &'static str,
    easy_button: Rect,
    medium: &'static str,
    medium_button: Rect,
    hard: &'static str,
    hard_button: Rect,
    custom_prompt: &'static str,
    custom_button: Rect,
    custom_width: usize,
    custom_width_box: Rect,
    custom_height: usize,
    custom_height_box: Rect,
    custom_mines: usize,
    custom_mines_box: Rect,
    confirm: &'static str,
    confirm_button: Rect,
    selected: DifficultySetting,
    state: MainMenuState,
    cursor: usize,
}

#[derive(Debug, PartialEq)]
enum MainMenuState {
    EditingWidth,
    EditingHeight,
    EditingMines,
    Default,
}

impl MainMenu {
    fn new(diff: DifficultySetting, custom_diff: Difficulty) -> MainMenu {
        let header = "Difficulty Width Height Mines";
        let easy = "Easy           9      9    10";
        let medium = "Medium      16     16    40";
        let hard = "Hard          30     16    99";
        let custom_prompt = "Custom";
        let confirm = "Confirm";
        let confirm_button = Rect::new(200., 150., 100., 30.);
        let easy_button = Rect::new(10., 30., 10., 10.);
        let medium_button = Rect::new(10., 60., 10., 10.);
        let hard_button = Rect::new(10., 90., 10., 10.);
        let custom_button = Rect::new(10., 120., 10., 10.);
        let (custom_width, custom_height, custom_mines) =
            (custom_diff.0, custom_diff.1, custom_diff.2);
        let custom_width_box = Rect::new(90., 120., 30., 30.);
        let custom_height_box = Rect::new(130., 120., 30., 30.);
        let custom_mines_box = Rect::new(170., 120., 30., 30.);
        let menu = MainMenu {
            header,
            easy,
            easy_button,
            medium,
            medium_button,
            hard,
            hard_button,
            custom_prompt,
            custom_button,
            custom_width,
            custom_width_box,
            custom_height,
            custom_height_box,
            custom_mines,
            custom_mines_box,
            confirm,
            confirm_button,
            selected: diff,
            state: MainMenuState::Default,
            cursor: 0,
        };
        menu
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        set_window_size(ctx, 300., 180.)?;
        graphics::clear(ctx, graphics::WHITE);
        let fill = DrawMode::fill();
        let mut buttons_mesh = graphics::MeshBuilder::new();
        match self.selected {
            DifficultySetting::Easy => {
                buttons_mesh
                    .rectangle(fill, self.easy_button, graphics::BLACK)
                    .rectangle(fill, self.medium_button, BUTTON_BG)
                    .rectangle(fill, self.hard_button, BUTTON_BG)
                    .rectangle(fill, self.custom_button, BUTTON_BG);
            }
            DifficultySetting::Medium => {
                buttons_mesh
                    .rectangle(fill, self.easy_button, BUTTON_BG)
                    .rectangle(fill, self.medium_button, graphics::BLACK)
                    .rectangle(fill, self.hard_button, BUTTON_BG)
                    .rectangle(fill, self.custom_button, BUTTON_BG);
            }
            DifficultySetting::Hard => {
                buttons_mesh
                    .rectangle(fill, self.easy_button, BUTTON_BG)
                    .rectangle(fill, self.medium_button, BUTTON_BG)
                    .rectangle(fill, self.hard_button, graphics::BLACK)
                    .rectangle(fill, self.custom_button, BUTTON_BG);
            }
            DifficultySetting::Custom => {
                buttons_mesh
                    .rectangle(fill, self.easy_button, BUTTON_BG)
                    .rectangle(fill, self.medium_button, BUTTON_BG)
                    .rectangle(fill, self.hard_button, BUTTON_BG)
                    .rectangle(fill, self.custom_button, graphics::BLACK);
            }
        }
        buttons_mesh
            .rectangle(fill, self.confirm_button, BUTTON_BG)
            .rectangle(fill, self.custom_width_box, TEXT_BG)
            .rectangle(fill, self.custom_height_box, TEXT_BG)
            .rectangle(fill, self.custom_mines_box, TEXT_BG);
        let buttons_mesh = buttons_mesh.build(ctx)?;
        graphics::draw(ctx, &buttons_mesh, graphics::DrawParam::new())?;
        let params = graphics::DrawParam::default().color(graphics::BLACK);
        let header_text = graphics::Text::new(self.header.to_string());
        graphics::draw(ctx, &header_text, params.dest([20., 0.]))?;
        let easy_text = graphics::Text::new(self.easy.to_string());
        graphics::draw(
            ctx,
            &easy_text,
            params.dest(Point2::new(
                self.easy_button.right(),
                self.easy_button.top(),
            )),
        )?;
        let medium_text = graphics::Text::new(self.medium.to_string());
        graphics::draw(
            ctx,
            &medium_text,
            params.dest(Point2::new(
                self.medium_button.right(),
                self.medium_button.top(),
            )),
        )?;
        let hard_text = graphics::Text::new(self.hard.to_string());
        graphics::draw(
            ctx,
            &hard_text,
            params.dest(Point2::new(
                self.hard_button.right(),
                self.hard_button.top(),
            )),
        )?;
        let custom_text = graphics::Text::new(self.custom_prompt.to_string());
        graphics::draw(
            ctx,
            &custom_text,
            params.dest(Point2::new(
                self.custom_button.right(),
                self.custom_button.top(),
            )),
        )?;
        let custom_width_text = graphics::Text::new(self.custom_width.to_string());
        graphics::draw(
            ctx,
            &custom_width_text,
            params.dest(Point2::new(
                self.custom_width_box.left(),
                self.custom_width_box.top(),
            )),
        )?;
        let custom_height_text = graphics::Text::new(self.custom_height.to_string());
        graphics::draw(
            ctx,
            &custom_height_text,
            params.dest(Point2::new(
                self.custom_height_box.left(),
                self.custom_height_box.top(),
            )),
        )?;
        let custom_mines_text = graphics::Text::new(self.custom_mines.to_string());
        graphics::draw(
            ctx,
            &custom_mines_text,
            params.dest(Point2::new(
                self.custom_mines_box.left(),
                self.custom_mines_box.top(),
            )),
        )?;
        let confirm_text = graphics::Text::new(self.confirm.to_string());
        graphics::draw(
            ctx,
            &confirm_text,
            params.dest(Point2::new(
                self.confirm_button.left() + 5.,
                self.confirm_button.top(),
            )),
        )?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Option<(DifficultySetting, Difficulty)> {
        let p = Point2::new(x, y);
        if button == MouseButton::Left && self.state == MainMenuState::Default {
            if self.confirm_button.contains(p) {
                self.state = MainMenuState::Default;
                return Some((
                    self.selected,
                    Difficulty(self.custom_width, self.custom_height, self.custom_mines),
                ));
            } else if self.easy_button.contains(p) {
                self.selected = DifficultySetting::Easy;
            } else if self.medium_button.contains(p) {
                self.selected = DifficultySetting::Medium;
            } else if self.hard_button.contains(p) {
                self.selected = DifficultySetting::Hard;
            } else if self.custom_button.contains(p) {
                self.selected = DifficultySetting::Custom;
            } else if self.custom_width_box.contains(p) {
                self.state = MainMenuState::EditingWidth;
            } else if self.custom_height_box.contains(p) {
                self.state = MainMenuState::EditingHeight;
            } else if self.custom_mines_box.contains(p) {
                self.state = MainMenuState::EditingMines;
            }
        }
        None
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) -> bool {
        match self.state {
            MainMenuState::EditingWidth => {
                let mut width_string = self.custom_width.to_string();
                match keycode {
                    KeyCode::Back => {
                        if self.cursor > 0 {
                            width_string = [
                                &width_string[0..(self.cursor - 1)],
                                &width_string[self.cursor..width_string.len()],
                            ]
                            .concat()
                            .to_owned();
                            self.cursor -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor < width_string.len() {
                            self.cursor += 1;
                        }
                    }
                    KeyCode::Left => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        }
                    }
                    KeyCode::Return | KeyCode::NumpadEnter => {
                        self.state = MainMenuState::Default;
                        self.validate_custom_data();
                    }
                    _ => {
                        if let Some(num) = key_to_number(&keycode) {
                            width_string = format!(
                                "{}{}{}",
                                &width_string[0..self.cursor],
                                num,
                                &width_string[self.cursor..width_string.len()]
                            );
                        }
                    }
                }
                self.custom_width = width_string.parse().unwrap_or(0);
            }
            MainMenuState::EditingHeight => {
                let mut height_string = self.custom_height.to_string();
                match keycode {
                    KeyCode::Back => {
                        if self.cursor > 0 {
                            height_string = [
                                &height_string[0..(self.cursor - 1)],
                                &height_string[self.cursor..height_string.len()],
                            ]
                            .concat()
                            .to_owned();
                            self.cursor -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor < height_string.len() {
                            self.cursor += 1;
                        }
                    }
                    KeyCode::Left => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        }
                    }
                    KeyCode::Return | KeyCode::NumpadEnter => {
                        self.state = MainMenuState::Default;
                        self.validate_custom_data();
                    }
                    _ => {
                        if let Some(num) = key_to_number(&keycode) {
                            height_string = format!(
                                "{}{}{}",
                                &height_string[0..self.cursor],
                                num,
                                &height_string[self.cursor..height_string.len()]
                            );
                        }
                    }
                }
                self.custom_height = height_string.parse().unwrap_or(0);
            }
            MainMenuState::EditingMines => {
                let mut mines_string = self.custom_mines.to_string();
                match keycode {
                    KeyCode::Back => {
                        if self.cursor > 0 {
                            mines_string = [
                                &mines_string[0..(self.cursor - 1)],
                                &mines_string[self.cursor..mines_string.len()],
                            ]
                            .concat()
                            .to_owned();
                            self.cursor -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor < mines_string.len() {
                            self.cursor += 1;
                        }
                    }
                    KeyCode::Left => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        }
                    }
                    KeyCode::Return | KeyCode::NumpadEnter => {
                        self.state = MainMenuState::Default;
                        self.validate_custom_data();
                    }
                    _ => {
                        if let Some(num) = key_to_number(&keycode) {
                            mines_string = format!(
                                "{}{}{}",
                                &mines_string[0..self.cursor],
                                num,
                                &mines_string[self.cursor..mines_string.len()]
                            );
                        }
                    }
                }
                self.custom_mines = mines_string.parse().unwrap_or(0);
            }
            MainMenuState::Default => {
                if keycode == KeyCode::Escape {
                    return true;
                }
            }
        }
        false
    }

    fn validate_custom_data(&mut self) {
        if self.custom_width > MAX_WIDTH {
            self.custom_width = MAX_WIDTH;
        }
        if self.custom_width < MIN_WIDTH {
            self.custom_width = MIN_WIDTH;
        }
        if self.custom_height > MAX_HEIGHT {
            self.custom_height = MAX_HEIGHT;
        }
        if self.custom_height < MIN_HEIGHT {
            self.custom_height = MIN_HEIGHT;
        }
        if self.custom_mines > (self.custom_width - 1) * (self.custom_height - 1) {
            self.custom_mines = (self.custom_width - 1) * (self.custom_height - 1);
        }
        if self.custom_mines < 1 {
            self.custom_mines = 1;
        }
    }
}

fn key_to_number(keycode: &KeyCode) -> Option<usize> {
    match keycode {
        KeyCode::Key0 => Some(0),
        KeyCode::Key1 => Some(1),
        KeyCode::Key2 => Some(2),
        KeyCode::Key3 => Some(3),
        KeyCode::Key4 => Some(4),
        KeyCode::Key5 => Some(5),
        KeyCode::Key6 => Some(6),
        KeyCode::Key7 => Some(7),
        KeyCode::Key8 => Some(8),
        KeyCode::Key9 => Some(9),
        KeyCode::Numpad0 => Some(0),
        KeyCode::Numpad1 => Some(1),
        KeyCode::Numpad2 => Some(2),
        KeyCode::Numpad3 => Some(3),
        KeyCode::Numpad4 => Some(4),
        KeyCode::Numpad5 => Some(5),
        KeyCode::Numpad6 => Some(6),
        KeyCode::Numpad7 => Some(7),
        KeyCode::Numpad8 => Some(8),
        KeyCode::Numpad9 => Some(9),
        _ => None,
    }
}

#[derive(Debug)]
struct PopupMenu {
    prompt: &'static str,
    button_1_prompt: &'static str,
    button_2_prompt: &'static str,
    bounds: Rect,
    button_1_box: Rect,
    button_2_box: Rect,
}

impl PopupMenu {
    fn new(
        prompt: &'static str,
        button_1_prompt: &'static str,
        button_2_prompt: &'static str,
        x: f32,
        y: f32,
    ) -> PopupMenu {
        let bounds = Rect::new(x, y, 150., 80.);
        let button_1_box = Rect::new(x + 5., y + 40., 90., 30.);
        let button_2_box = Rect::new(x + 105., y + 40., 40., 30.);
        let m = PopupMenu {
            prompt,
            button_1_prompt,
            button_2_prompt,
            bounds,
            button_1_box,
            button_2_box,
        };
        m
    }

    fn draw(&self, ctx: &mut Context) -> GameResult {
        let params = graphics::DrawParam::default().dest(Point2::new(self.bounds.x, self.bounds.y));
        let window = graphics::MeshBuilder::new()
            .rectangle(
                DrawMode::Fill(graphics::FillOptions::DEFAULT),
                self.bounds,
                graphics::WHITE,
            )
            .rectangle(
                DrawMode::Fill(graphics::FillOptions::DEFAULT),
                self.button_1_box,
                BUTTON_BG,
            )
            .rectangle(
                DrawMode::Fill(graphics::FillOptions::DEFAULT),
                self.button_2_box,
                BUTTON_BG,
            )
            .build(ctx)?;
        graphics::draw(ctx, &window, params)?;
        let params = params.color(graphics::BLACK);
        let prompt_text = graphics::Text::new(self.prompt.to_string());
        graphics::draw(
            ctx,
            &prompt_text,
            params.dest(Point2::new(self.bounds.x + 30., self.bounds.y + 10.)),
        )?;
        let button_1_text = graphics::Text::new(self.button_1_prompt.to_string());
        graphics::draw(
            ctx,
            &button_1_text,
            params.dest(Point2::new(
                self.button_1_box.x + 30.,
                self.button_1_box.y + 10.,
            )),
        )?;
        let button_2_text = graphics::Text::new(self.button_2_prompt.to_string());
        graphics::draw(
            ctx,
            &button_2_text,
            params.dest(Point2::new(
                self.button_2_box.x + 10.,
                self.button_2_box.y + 10.,
            )),
        )?;
        Ok(())
    }

    fn mouse_button_down_event(&self, x: f32, y: f32) -> u8 {
        if self.button_1_box.contains(Point2::new(x, y)) {
            1
        } else if self.button_2_box.contains(Point2::new(x, y)) {
            2
        } else {
            0
        }
    }
}

fn set_window_size(ctx: &mut Context, width: f32, height: f32) -> GameResult {
    graphics::set_drawable_size(ctx, width, height)?;
    graphics::set_screen_coordinates(ctx, Rect::new(0., 0., width, height))?;
    Ok(())
}

pub fn start_game() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ContextBuilder::new("Minesweeperrs", "Eric McHugh")
        .add_resource_path(resource_dir)
        .window_setup(ggez::conf::WindowSetup {
            title: "Minesweeper.rs".to_owned(),
            samples: ggez::conf::NumSamples::Zero,
            vsync: true,
            icon: "/mine.png".to_owned(),
            srgb: true,
        });

    let (ctx, events_loop) = &mut cb.build()?;

    let mut game = MinesweeperGame::new(ctx)?;
    println!("{}", game);
    event::run(ctx, events_loop, &mut game)
}
