use rand::prelude::*;
use std::fmt;
use std::io::stdin;

// Models
#[derive(Debug, Copy, Clone)]
enum Player {
    User,
    Computer,
}

#[derive(Debug, Copy, Clone)]
enum Cell {
    Nought, // First player
    Cross,
    Unfilled,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Cell::Unfilled => ' ',
            Cell::Nought => 'o',
            Cell::Cross => 'x',
        };
        write!(f, "{}", c)
    }
}

type Board = [Cell; 9];

#[derive(Debug, Copy, Clone)]
enum GameStatus {
    Draw,
    NotFinished,
    Settled(Player),
}

#[derive(Debug, Copy, Clone)]
struct Model {
    first_player: Option<Player>,
    board: Board,
    status: GameStatus,
}

impl Model {
    fn new() -> Self {
        Self {
            status: GameStatus::NotFinished,
            board: [Cell::Unfilled; 9],
            first_player: None,
        }
    }
}

// Message
enum Message {
    CellClicked(usize),
    PlayerSelected { user_play_first: bool },
    NoMessage,
}

// View
fn view(model: Model) -> Message {
    if let GameStatus::Draw = model.status {
        print_board(&model.board);
        println!("======== Draw! =======");
        Message::NoMessage
    } else if let GameStatus::Settled(player) = model.status {
        let msg_string = match player {
            Player::User => "You win, nice!",
            Player::Computer => "You lose, too bad! Try again!",
        };
        print_board(&model.board);
        println!("======== {} =======", msg_string);
        Message::NoMessage
    } else {
        if model.first_player.is_none() {
            return select_first_player_view();
        }
        print_board(&model.board);
        let x = ask_move(&get_available_cells(&model.board));
        Message::CellClicked(x)
    }
}

fn select_first_player_view() -> Message {
    let do_user_play_first = ask_user_to_be_first();
    Message::PlayerSelected {
        user_play_first: do_user_play_first,
    }
}

fn print_board(board: &Board) {
    let top_row = format!("0|1|2  {}|{}|{}", board[0], board[1], board[2]);
    let mid_row = format!("3|4|5  {}|{}|{}", board[3], board[4], board[5]);
    let bot_row = format!("6|7|8  {}|{}|{}", board[6], board[7], board[8]);
    println!("{}\n{}\n{}", top_row, mid_row, bot_row);
}

fn get_user_input() -> Option<String> {
    let mut input = String::new();
    match stdin().read_line(&mut input) {
        Ok(_) => Some(input),
        Err(_) => None,
    }
}

fn ask_user_to_be_first() -> bool {
    println!("Do you want to play first? [y/n]: ");
    loop {
        let ans = get_user_input();
        if let Some(s) = ans {
            let first_letter = s.get(0..1);
            match first_letter {
                Some("y") => return true,
                Some("n") => return false,
                _ => println!("Please input 'y' or 'n' :"),
            }
        }
    }
}

fn ask_move(available: &Vec<usize>) -> usize {
    println!("What's your move? [0-8]: ");
    loop {
        let ans = get_user_input();
        if let Some(s) = ans {
            let ans = s.get(0..1).and_then(|s| s.parse::<usize>().ok());
            if let Some(i) = ans {
                if available.contains(&i) {
                    return i;
                } else {
                    println!("The cell {:?} is not available", i)
                }
            }
        }
        println!("Please input [0-8] :")
    }
}

// Update
fn update(model: Model, message: Message) -> Model {
    match message {
        Message::CellClicked(selected_cell) => update_board(model, selected_cell),
        Message::PlayerSelected {
            user_play_first: flag,
        } => update_player_selection(model, flag),
        Message::NoMessage => model,
    }
}

fn update_player_selection(model: Model, is_player_first: bool) -> Model {
    if is_player_first {
        Model {
            first_player: Some(Player::User),
            ..model
        }
    } else {
        let new_model = Model {
            first_player: Some(Player::Computer),
            ..model
        };
        update_board_with_random_play_computer_move(new_model)
    }
}

fn update_board_helper(board: &Board, selected_index: usize, cell: Cell) -> Board {
    let mut new_board = [Cell::Unfilled; 9];
    for i in 0..9 {
        if i == selected_index {
            new_board[i] = cell
        } else {
            new_board[i] = board[i]
        }
    }
    new_board
}

fn update_board(model: Model, selected_cell: usize) -> Model {
    update_board_with_random_play_computer_move(update_board_with_user_move(model, selected_cell))
}

fn update_board_with_user_move(model: Model, selected_cell: usize) -> Model {
    let user_cell_type = if let Some(Player::User) = model.first_player {
        Cell::Nought
    } else {
        Cell::Cross
    };
    let new_board = update_board_helper(&model.board, selected_cell, user_cell_type);
    let new_model = Model {
        board: new_board,
        ..model
    };
    update_game_status(new_model)
}

fn update_board_with_random_play_computer_move(model: Model) -> Model {
    let mut availables = get_available_cells(&model.board);
    let mut rng = rand::thread_rng();
    availables.shuffle(&mut rng);
    let computer_cell_type = if let Some(Player::User) = model.first_player {
        Cell::Cross
    } else {
        Cell::Nought
    };
    if let Some(i) = availables.pop() {
        let new_board = update_board_helper(&model.board, i, computer_cell_type);
        let new_model = Model {
            board: new_board,
            ..model
        };
        update_game_status(new_model)
    } else {
        update_game_status(model)
    }
}

fn update_game_status(model: Model) -> Model {
    let mut noughts = Vec::new();
    let mut crosses = Vec::new();
    let mut num_unfilled = 0;
    for i in 0..9 {
        if let Cell::Nought = model.board[i] {
            noughts.push(i);
        } else if let Cell::Cross = model.board[i] {
            crosses.push(i);
        } else {
            num_unfilled += 1;
        }
    }
    if has_bingo(noughts) {
        return Model {
            status: GameStatus::Settled(if let Some(Player::User) = model.first_player {
                Player::User
            } else {
                Player::Computer
            }),
            ..model
        };
    }
    if has_bingo(crosses) {
        return Model {
            status: GameStatus::Settled(if let Some(Player::User) = model.first_player {
                Player::Computer
            } else {
                Player::User
            }),
            ..model
        };
    }
    if num_unfilled == 0 {
        return Model {
            status: GameStatus::Draw,
            ..model
        };
    }
    model
}

// Helpers
fn get_available_cells(board: &Board) -> Vec<usize> {
    let mut availables: Vec<usize> = Vec::new();
    for i in 0..9 {
        if match board[i] {
            Cell::Unfilled => true,
            _ => false,
        } {
            availables.push(i)
        }
    }
    availables
}

fn has_bingo(indices: Vec<usize>) -> bool {
    let pattern = [
        (0, 1, 2),
        (3, 4, 5),
        (6, 7, 8),
        (0, 3, 6),
        (1, 4, 7),
        (2, 5, 8),
        (0, 4, 8),
        (2, 4, 6),
    ];
    for (a, b, c) in &pattern {
        if indices.contains(&a) & indices.contains(&b) & indices.contains(&c) {
            return true;
        }
    }
    false
}

fn main() {
    let mut model = Model::new();
    while let GameStatus::NotFinished = model.status {
        let msg = view(model);
        model = update(model, msg);
    }
    view(model);
}
