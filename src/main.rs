mod game2d;
mod minesweeper;
mod textgame;

use std::io;

use textgame::TextGame;

fn main() {
    match game2d::start_game() {
        Ok(_) => std::process::exit(0),
        Err(message) => println!("Game ended with an error message: {}", message),
    };
    println!("Enter the kind of game to run(console or 2d): ");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let input = input.trim();
    if input == "console" {
        let mut game = TextGame::new();
        game.main_loop();
    } else if input == "2d" {
        //
    } else {
        println!("You must enter either console or 2d.");
    }
}
