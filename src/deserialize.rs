#![allow(dead_code, unused_variables, unused_imports, unused_mut)]


use std::collections::BTreeMap;
use super::player::Player;
use super::autoplayers::XMLTestPlayer;
use super::dice::Dice;
use super::board::{Color, Board, Pawn, Loc, MoveResult, PawnLocs};
use super::game::{Move, MoveType};
use super::constants::*;
use super::serialize::*;
use super::board::*;
use super::quick_xml::reader::Reader;
use super::quick_xml::events::Event;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

#[derive(Debug, PartialEq, Eq)]
pub enum XmlMessage {
    StartGame,
    DoMove,
    DoublesPenalty,
    Error,
}

/// This function will decide which deserialization functino gets called given an xml string
pub fn deserialize_decision(request: String) -> XmlMessage {
    let mut reader = Reader::from_str(&request);
    reader.trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"start-game" => return XmlMessage::StartGame,
                    b"do-move" => return XmlMessage::DoMove,
                    b"doubles-penalty" => return XmlMessage::DoublesPenalty,
                    _ => {
                        panic!("PANIK WITH {:#?}", e.name());
                        return XmlMessage::Error;
                    }
                }
            }
            _ => return XmlMessage::Error,
        }
    }


}


/// This function will receive a string about a new game starting. It will
pub fn deserialize_start_game(request: String) -> Color {
    let mut reader = Reader::from_str(&request);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut txt = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"start-game" => println!("start game"),
                    _ => (),
                }
            }
            Ok(Event::Text(e)) => {
                txt.push(e.unescape_and_decode(&reader)
                             .unwrap())
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!(),
            _ => (),
        }
        buf.clear();
    }
    match txt.pop()
              .unwrap()
              .as_ref() {
        "Red" | "red" => Color::Red,
        "Blue" | "blue" => Color::Blue,
        "Yellow" | "yellow" => Color::Yellow,
        "Green" | "green" => Color::Green,
        _ => panic!("That's not a color"),
    }
}


/// This function takes in the raw xml string and converts it to a vector of moves
/// The first function call works to split the string up into a vector of strings and remove xml tags
/// The second function call then builds up the vector of moves
pub fn deserialize_moves(xml: String) -> Vec<Move> {
    let string_vec: Vec<String> = move_string_to_vec_string(xml);
    let result = vec_string_to_vec_move(string_vec);
    result
}

/// This function will take in an xml string and return
/// a vector of strings corresponding to the moves.
pub fn move_string_to_vec_string(xml: String) -> Vec<String> {
    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut txt = Vec::new();
    // let mut pos_vec = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                // We match on the name of the tag, then add the tag to the text vector
                // <enter-piece> yadadada </enter-piece> becomes Vec<String> => vec![enter-piece, enterpiece args, ...]
                // This vector is then concatenated with the other move types
                match e.name() {
                    b"moves" => println!("test"),
                    b"enter-piece" => {
                        txt.push(e.unescape_and_decode(&reader)
                                     .unwrap())
                    }
                    b"move-piece-home" => {
                        txt.push(e.unescape_and_decode(&reader)
                                     .unwrap())
                    }
                    b"move-piece-main" => {
                        txt.push(e.unescape_and_decode(&reader)
                                     .unwrap())
                    }
                    _ => (),
                }
            }
            Ok(Event::Text(e)) => {
                txt.push(e.unescape_and_decode(&reader)
                             .unwrap())
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!(),
            _ => (),
        }
        buf.clear();
    }
    txt
}

/// This function will take a vector of strings and dispatch on the terms to build up moves
/// The vector of strings will have the type of move and the necessary components to build
/// up the move.
pub fn vec_string_to_vec_move(vec_string: Vec<String>) -> Vec<Move> {
    let mut vec_move: Vec<Move> = Vec::new();
    let mut it = vec_string.iter();
    // We use the iterate through the vec of strings to build up individual moves
    // This implemenation heavily relies on no malformed moves
    loop {
        match it.next() {
            Some(x) => {
                match x.as_ref() {
                    "enter-piece" => {
                        let curr_move: Move = Move {
                            m_type: MoveType::EnterPiece,
                            pawn: Pawn {
                                color: string_to_color(it.next()
                                                           .unwrap()
                                                           .to_string()),
                                id: it.next()
                                    .unwrap()
                                    .parse::<usize>()
                                    .unwrap(), //WoW!
                            },
                        };
                        vec_move.push(curr_move);
                    }
                    "move-piece-home" => {
                        let curr_move: Move = Move {
                            pawn: Pawn {
                                color: string_to_color(it.next()
                                                           .unwrap()
                                                           .to_string()),
                                id: it.next()
                                    .unwrap()
                                    .parse::<usize>()
                                    .unwrap(),
                            },
                            m_type: MoveType::MoveHome {
                                start: it.next()
                                    .unwrap()
                                    .parse::<usize>()
                                    .unwrap(),
                                distance: it.next()
                                    .unwrap()
                                    .parse::<usize>()
                                    .unwrap(),
                            },
                        };
                        vec_move.push(curr_move);
                    }
                    "move-piece-main" => {
                        let curr_move: Move = Move {
                            pawn: Pawn {
                                color: string_to_color(it.next()
                                                           .unwrap()
                                                           .to_string()),
                                id: it.next()
                                    .unwrap()
                                    .parse::<usize>()
                                    .unwrap(),
                            },
                            m_type: MoveType::MoveMain {
                                start: it.next()
                                    .unwrap()
                                    .parse::<usize>()
                                    .unwrap(),
                                distance: it.next()
                                    .unwrap()
                                    .parse::<usize>()
                                    .unwrap(),
                            },
                        };
                        vec_move.push(curr_move);
                    }
                    _ => panic!("XML MOVE NOT RECOGNIZED"),
                }
            }

            None => break,
        }
    }
    vec_move
}

pub fn string_to_color(string: String) -> Color {
    match string
              .to_lowercase()
              .as_ref() {
        "red" => Color::Red,
        "blue" => Color::Blue,
        "yellow" => Color::Yellow,
        "green" => Color::Green,
        _ => panic!("string to color: {}", string),             
    }
}



pub fn build_pawn_from_strings(color: String, id: String) -> Pawn {
    let pawn: Pawn = Pawn {
        color: string_to_color(color),
        id: id.parse::<usize>()
            .unwrap(),
    };
    pawn
}

/// Tags are removed because the ordering in the vector is enough to build up board.
/// In the future, it may be wise to keep the tags and use contracts to make sure no
/// move xml is malformed.
pub fn trim_xml(xml_string: &Vec<String>) -> Vec<String> {
    let mut xml = xml_string.clone();
    xml.retain(|x| *x != "id".to_string());
    xml.retain(|x| *x != "color".to_string());
    xml.retain(|x| *x != "pawn".to_string());
    xml
}

/// We start we a new board and build up the board's position BTreeMap
pub fn split_up_vec_xml_string(vec_xml_string: Vec<String>) -> Board {
    let mut board: Board = Board::new();
    // These repeated indexing and split off are all organize the vector of strings into vector of strings that all correspond to same class
    // of spots on the board
    let mut start_end_index = vec_xml_string
        .clone()
        .iter()
        .position(|x| *x == "main".to_string())
        .unwrap();

    let mut home_row_index = vec_xml_string
        .clone()
        .iter()
        .position(|x| *x == "home".to_string())
        .unwrap();
    let mut start = vec_xml_string.clone();

    let mut main = start.split_off(start_end_index);
    let mut main_end_index = main.clone()
        .iter()
        .position(|x| *x == "home-rows".to_string())
        .unwrap();
    let mut home_rows = main.split_off(main_end_index);

    let mut home_row_end_index = home_rows
        .clone()
        .iter()
        .position(|x| *x == "home".to_string())
        .unwrap();
    let mut home = home_rows.split_off(home_row_end_index);

    // Since home-rows and main have the same structure in our board representation,
    // we will concatenate them.
    // The retain call here will knock off the front home-rows tag from the string,
    // and go through loop.
    let mut main = trim_xml(&main);
    home_rows = trim_xml(&home_rows);
    home_rows.retain(|x| *x != "home-rows".to_string());
    //main.append(&mut home_rows);


    // TODO, Robby's board handles things differently than our boards.
    // For main row stuff, when we get his index, we must add 50 and % 68 to get to our representation
    // For Homerows, we must add the color's homerow offset (i.e. 100,200,300 or 400).
    let mut it = main.iter();
    it.next();
    loop {
        if let Some(loc_string) = it.next() {
            match loc_string.as_ref() {
                "piece-loc" => {
                    let curr_element = it.next().unwrap();
                    let curr_color: Color = string_to_color(curr_element
                                                                .clone());
                    let mut curr_id = it.next()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    assert!("loc" == it.next().unwrap());
                    let robby_spot_index = it.next()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    let curr_spot_index = (robby_spot_index + 50) % 68; // Translate robby position to our position in main ring
                    let mut positions_copy = board.positions.clone();
                    let mut pawn_locs = positions_copy
                        .get_mut(&curr_color)
                        .unwrap();
                    pawn_locs[curr_id] = Loc::Spot { index: curr_spot_index };
                    board
                        .positions
                        .insert(curr_color, pawn_locs.clone());
                }
                _ => break,
            };
        } else {
            break;
        }
    }
    let mut it = home_rows.iter();
    loop {
        if let Some(loc_string) = it.next() {
            match loc_string.as_ref() {
                "piece-loc" => {
                    let curr_element = it.next().unwrap();
                    let curr_color: Color = string_to_color(curr_element
                                                                .clone());
                    let mut curr_id = it.next()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    assert!("loc" == it.next().unwrap());
                    let robby_spot_index = it.next()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    let curr_spot_index = robby_spot_index +
                                          Board::get_home_row(&curr_color); // Added for Robby Translation
                    let mut positions_copy = board.positions.clone();
                    let mut pawn_locs = positions_copy
                        .get_mut(&curr_color)
                        .unwrap();
                    pawn_locs[curr_id] = Loc::Spot { index: curr_spot_index };
                    board
                        .positions
                        .insert(curr_color, pawn_locs.clone());
                }
                _ => break,
            };
        } else {
            break;
        }
    }
    home = trim_xml(&home);
    let mut it = home.iter();

    it.next();
    // Skip the "main" tag in the vector of strings
    loop {
        if let Some(color_string) = it.next() {
            println!("{}", color_string);
            let curr_color: Color = string_to_color(color_string.clone());
            let mut curr_id = it.next()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let mut positions_copy = board.positions.clone();
            let mut pawn_locs = positions_copy
                .get_mut(&curr_color)
                .unwrap();
            pawn_locs[curr_id] = Loc::Home;
            board
                .positions
                .insert(curr_color, pawn_locs.clone());
        } else {
            break;
        }
    }

    board


}

/// For deserializing do move, we decided to not use the xml library for deserialize dice because this was quicker.
/// This makes for a little messier code
pub fn deserialize_do_move(xml: String) -> (Board, Dice) {
    // we need to split up the string into the board and dice components.
    // Each function will be passed the xml string.
    // The board function will break out of its loop when it sees the tag <dice>
    let board: Board = deserialize_board(xml.clone());
    let dice: Dice = deserialize_dice(xml);
    (board, dice)
}

pub fn deserialize_dice(xml: String) -> Dice {

    /*
    let mut string_vector: Vec<&str> = xml.split(' ').collect();
    println!("deserializing dice"); 
    // This collapses the string into a vector
    let mut dice_index: usize = string_vector
        .iter()
        .position(|x| *x == "<dice>")
        .unwrap();
    println!("{}", dice_index);

    let mut dice_vector = string_vector.split_off(dice_index);
    println!("After split {:#?}", dice_vector);
    dice_vector.retain(|x| *x != "<dice>");
    dice_vector.retain(|x| *x != "</dice>");
    dice_vector.retain(|x| *x != "<die>");
    dice_vector.retain(|x| *x != "</die>");
    dice_vector.retain(|x| *x != "</do-move>");

    println!("After retention {:#?}", dice_vector);
    let usize_vector: Vec<usize> = dice_vector
        .iter()
        .map(|s| s.parse::<usize>().unwrap())
        .collect();
    let dice: Dice = Dice { rolls: usize_vector };
    dice
     */
    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut txt = Vec::new();
    let mut dice_p = false; // WE have reached the part of the xml string where the dice live
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"dice" => dice_p = true,
                    _ => (),
                };
            }
            Ok(Event::Text(e)) => {
                if dice_p {
                    txt.push(e.unescape_and_decode(&reader)
                                 .unwrap());
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Dice Parse Error {}", e),
            _ => (),
        }
        buf.clear()
    }
    let usize_vector: Vec<usize> = txt.iter()
        .map(|s| s.parse::<usize>().unwrap())
        .collect();
    let dice: Dice = Dice { rolls: usize_vector };
    dice
}

pub fn deserialize_board(xml: String) -> Board {
    let mut vec_xml_string: Vec<String> = xml_board_to_vec_xml_string(xml);
    //println!("This is the board string: {:#?}", vec_xml_string);
    //let mut board: Board = vec_string_to_board(vec_string);
    let board: Board = split_up_vec_xml_string(vec_xml_string);
    board
}

pub fn xml_board_to_vec_xml_string(xml: String) -> Vec<String> {
    // Board is a BTreeMap from Color to PawnLocs
    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut txt = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"board" => (),
                    b"dice" => break,
                    _ => {
                        txt.push(e.unescape_and_decode(&reader)
                                     .unwrap())
                    }
                }
            }
            Ok(Event::Text(e)) => {
                txt.push(e.unescape_and_decode(&reader)
                             .unwrap())
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!(),
            _ => (),
        }
        buf.clear();
    }
    txt


}



mod tests {
    use super::*;
    use serialize;


    #[test]
    /// Test the initial decision of the parser to send to right deserializer
    fn deserialize_decision_test() {
        let start_response: String = "<start-game> egal".to_string();
        let do_move: String = "<do-move> egal".to_string();
        let doubles_penalty: String = "<doubles-penalty egal".to_string();
        assert!(XmlMessage::StartGame == deserialize_decision(start_response));
        assert!(XmlMessage::DoMove == deserialize_decision(do_move));
        assert!(XmlMessage::DoublesPenalty ==
                deserialize_decision(doubles_penalty));
        //assert!(XmlMessage::Error == deserialize_decision("<not> a tag".to_string()));

    }

    #[test]
    /// Parse then unparse and check if results are the same
    fn move_vector_test() {
        let m_1: Move = Move {
            m_type: MoveType::EnterPiece,
            pawn: Pawn {
                color: Color::Red,
                id: 2,
            },
        };
        let m_2: Move = Move {
            m_type: MoveType::MoveHome {
                start: 101,
                distance: 3,
            },
            pawn: Pawn {
                color: Color::Red,
                id: 2,
            },
        };
        let m_3: Move = Move {
            m_type: MoveType::MoveMain {
                start: 12,
                distance: 3,
            },
            pawn: Pawn {
                color: Color::Red,
                id: 2,
            },
        };

        let m_vec: Vec<Move> = vec![m_1.clone(), m_2.clone(), m_3.clone()];
        let xml = serialize::xml_moves(&m_vec);
        let test: Vec<Move> = deserialize_moves(xml);
        assert!(m_vec == test);
    }

    #[test]
    /// Parse the board
    fn deserialize_board_test() {
        assert!(Board::new() == deserialize_board(Board::new().xmlify()));
    }

    #[test]
    /// Deserialize the board given to us on the website
    fn deserialize_board_real_test() {
        let test_board: Board = Board::from(map!{
            Color::Red => [Loc::Home, Loc::Spot { index: 4}, Loc::Nest, Loc::Spot{ index: 101}],
            Color::Blue => [Loc::Nest, Loc::Spot { index: 202}, Loc::Home, Loc::Spot{ index: 21}],
            Color::Green => [Loc::Spot { index: 55 }, Loc::Nest, Loc::Spot{ index: 400 }, Loc::Home],
            Color::Yellow => [Loc::Spot { index: 303 }, Loc::Home, Loc::Spot{ index: 38}, Loc::Nest]
        });
        let test_response: String = "<board> <start> <pawn> <color> yellow </color> <id> 3 </id> </pawn> <pawn> <color> red </color> <id> 2 </id> </pawn> <pawn> <color> green </color> <id> 1 </id> </pawn> <pawn> <color> blue </color> <id> 0 </id> </pawn> </start> <main> <piece-loc> <pawn> <color> yellow </color> <id> 2 </id> </pawn> <loc> 56 </loc> </piece-loc> <piece-loc> <pawn> <color> blue </color> <id> 3 </id> </pawn> <loc> 39 </loc> </piece-loc> <piece-loc> <pawn> <color> red </color> <id> 1 </id> </pawn> <loc> 22 </loc> </piece-loc> <piece-loc> <pawn> <color> green </color> <id> 0 </id> </pawn> <loc> 5 </loc> </piece-loc> </main> <home-rows> <piece-loc> <pawn> <color> green </color> <id> 2 </id> </pawn> <loc> 0 </loc> </piece-loc> <piece-loc> <pawn> <color> red </color> <id> 3 </id> </pawn> <loc> 1 </loc> </piece-loc> <piece-loc> <pawn> <color> blue </color> <id> 1 </id> </pawn> <loc> 2 </loc> </piece-loc> <piece-loc> <pawn> <color> yellow </color> <id> 0 </id> </pawn> <loc> 3 </loc> </piece-loc> </home-rows> <home> <pawn> <color> yellow </color> <id> 1 </id> </pawn> <pawn> <color> red </color> <id> 0 </id> </pawn> <pawn> <color> green </color> <id> 3 </id> </pawn> <pawn> <color> blue </color> <id> 2 </id> </pawn> </home> </board>".to_string();
        assert!(test_board == deserialize_board(test_response));
    }

    #[test]
    /// Parse real game board
    fn deserialize_board_basic_test() {
        let board: Board = Board::new();
        let test_string = "<board> <start> <pawn> <color> yellow </color> <id> 3 </id> </pawn> <pawn> <color> yellow </color> <id> 2 </id> </pawn> <pawn> <color> yellow </color> <id> 1 </id> </pawn> <pawn> <color> yellow </color> <id> 0 </id> </pawn> <pawn> <color> red </color> <id> 3 </id> </pawn> <pawn> <color> red </color> <id> 2 </id> </pawn> <pawn> <color> red </color> <id> 1 </id> </pawn> <pawn> <color> red </color> <id> 0 </id> </pawn> <pawn> <color> green </color> <id> 3 </id> </pawn> <pawn> <color> green </color> <id> 2 </id> </pawn> <pawn> <color> green </color> <id> 1 </id> </pawn> <pawn> <color> green </color> <id> 0 </id></pawn> <pawn> <color> blue </color> <id> 3 </id> </pawn> <pawn> <color> blue </color> <id> 2 </id> </pawn> <pawn> <color> blue </color> <id> 1 </id> </pawn> <pawn> <color> blue </color> <id> 0 </id> </pawn> </start> <main> </main> <home-rows></home-rows> <home> </home> </board>".to_string();

        //assert!(false);
        assert!(board == deserialize_board(test_string));

    }

    #[test]
    fn deserialize_dice_test() {
        let dice: Dice = Dice { rolls: vec![1, 2, 3, 4] };
        assert!(dice == deserialize_dice(dice.xmlify()));

    }
    #[test]
    fn deserialize_do_move_test1() {
        let result: String = "<do-move><board><start><pawn><color>yellow</color><id>3</id></pawn><pawn><color>yellow</color><id>2</id></pawn><pawn><color>yellow</color><id>1</id></pawn><pawn><color>yellow</color><id>0</id></pawn><pawn><color>red</color><id>3</id></pawn><pawn><color>red</color><id>2</id></pawn><pawn><color>red</color><id>1</id></pawn><pawn><color>red</color><id>0</id></pawn><pawn><color>green</color><id>3</id></pawn><pawn><color>green</color><id>2</id></pawn><pawn><color>green</color><id>1</id></pawn><pawn><color>green</color><id>0</id></pawn><pawn><color>blue</color><id>3</id></pawn><pawn><color>blue</color><id>2</id></pawn><pawn><color>blue</color><id>1</id></pawn><pawn><color>blue</color><id>0</id></pawn></start><main></main><home-rows></home-rows><home></home></board><dice><die>2</die><die>3</die></dice></do-move>".to_string();
        let (board, dice) = deserialize_do_move(result);
        let test_dice: Dice = Dice { rolls: vec![2, 3] };
        let test_board: Board = Board::new();
        assert!(test_dice == dice);
        assert!(test_board == board);
    }

    #[test]
    fn deserialize_do_move_test() {
        let board: Board = Board::new();
        let dice: Dice = Dice { rolls: vec![1, 2] };

        let expected: String = "<board> <start> <pawn> <color> yellow </color> <id> 3 </id> </pawn> <pawn> <color> yellow </color> <id> 2 </id> </pawn> <pawn> <color> yellow </color> <id> 1 </id> </pawn> <pawn> <color> yellow </color> <id> 0 </id> </pawn> <pawn> <color> red </color> <id> 3 </id> </pawn> <pawn> <color> red </color> <id> 2 </id> </pawn> <pawn> <color> red </color> <id> 1 </id> </pawn> <pawn> <color> red </color> <id> 0 </id> </pawn> <pawn> <color> green </color> <id> 3 </id> </pawn> <pawn> <color> green </color> <id> 2 </id> </pawn> <pawn> <color> green </color> <id> 1 </id> </pawn> <pawn> <color> green </color> <id> 0 </id> </pawn> <pawn> <color> blue </color> <id> 3 </id> </pawn> <pawn> <color> blue </color> <id> 2 </id> </pawn> <pawn> <color> blue </color> <id> 1 </id> </pawn> <pawn> <color> blue </color> <id> 0 </id> </pawn> </start> <main> </main> <home-rows> </home-rows> <home> </home> </board>".to_string();
        //assert!(deserialize_do_move(parse::xml_do_move(&board,&dice)) == (Board::new(),));

        assert!((board.clone(), dice.clone()) ==
                deserialize_do_move(serialize::xml_do_move(&board, &dice)));
    }

}
