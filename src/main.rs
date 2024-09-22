use std::{io, thread};
use std::io::Write;
// use rand::Rng;
use crossterm::{cursor, event::{self, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode}, ExecutableCommand};
use std::time::{Duration, Instant};
use crossterm::event::KeyEventKind;
use colored::*;
use crossterm::terminal::{Clear, ClearType};
use rusty_audio::Audio;


struct Game {
    obstacles: [Vec<u8>; 24],
    coins: [Vec<u8>; 24],
    player: [u8; 2], // x y
    current_direction: u8,
    direction_queue: u8, // 0: w; 1: a; 2: s; 3: d
    is_finished: bool,
}

struct MapCalculator {
    map: [String; 24]
}

/*
fn random(x: i16, y: i16) -> i16 { // Creates random number between x and y
    let mut rng = rand::thread_rng();
    rng.gen_range(x..y)
}
*/

fn check_position(cur_pos: [u8; 2], obstacles: [Vec<u8>; 24]) -> bool {
    let x = cur_pos[0] as usize;
    let y = cur_pos[1] as usize;
    !obstacles[y].contains(&(x as u8))
}


impl MapCalculator {
    fn calculate_map(&self, character: char) -> [Vec<u8>; 24] {
        let mut coordinates_vector: [Vec<u8>; 24] = Default::default();

        for vec in &mut coordinates_vector {
            *vec = Vec::new();
        }

        for (row_idx, row) in self.map.iter().enumerate() {
            for (col_idx, ch) in row.chars().enumerate() {
                if ch == character {
                    coordinates_vector[row_idx].push(col_idx as u8);
                }
            }
        }
        coordinates_vector
    }
}


impl Game {
    fn draw(&mut self) {
        let mut map : String = Default::default();
        let mut coin_counter: u8 = 0;
        let mut stdout = io::stdout();
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        for y in 0..24 {
            let y: u8 = y;

            for x in 0..49 {
                let x: u8 = x;

                if self.obstacles[y as usize].contains(&x) {
                    map += &Colorize::blue("#").to_string();
                } else if self.player == [x, y] {
                    map += &Colorize::bright_yellow("@").to_string();
                } else if self.coins[y as usize].contains(&x) {
                    map += "•";
                    coin_counter += 1;
                } else {
                    map += " "
                }
            }
            map += "\n"
        }
        print!("{}", map);
        if coin_counter == 0 { self.is_finished = true; }
        stdout.flush().unwrap();
    }

    fn check_position(&mut self)  {
        let x = self.player[0] as usize;
        let y = self.player[1] as usize;

        let initial_length = self.coins[y].len();
        self.coins[y].retain(|&coin_x| coin_x != x as u8);

        if self.coins[y].len() < initial_length {
            thread::spawn( || {
                let mut coin_sound = Audio::new();
                coin_sound.add("coin", "src/sounds/collect_coin.mp3");
                coin_sound.play("coin");
                coin_sound.wait();
            });
        }
    }



    fn queue_checker(&mut self) {
        if !(self.player[0] % 2 == 0) { return; }
        let direction: u8 = self.direction_queue;
        match direction {
            0 => if check_position([self.player[0] , self.player[1] - 1], self.obstacles.clone()) { self.current_direction = direction }, // W
            1 => if check_position([self.player[0] - 2, self.player[1]], self.obstacles.clone()) { self.current_direction = direction }, // A
            2 => if check_position([self.player[0] , self.player[1] + 1], self.obstacles.clone()) { self.current_direction = direction }, // S
            3 => if check_position([self.player[0] + 2, self.player[1]], self.obstacles.clone()) { self.current_direction = direction }, // D
            _ => {}
        }
    }

    fn move_up(&mut self) { if check_position([self.player[0] , self.player[1] - 1], self.obstacles.clone()) { self.player[1] -= 1 }}
    fn move_left(&mut self) { if check_position([self.player[0] - 2, self.player[1]], self.obstacles.clone()) { self.player[0] -= 1 }}
    fn move_down(&mut self) { if check_position([self.player[0] , self.player[1] + 1], self.obstacles.clone()) { self.player[1] += 1 }}
    fn move_right(&mut self) { if check_position([self.player[0] + 2, self.player[1]], self.obstacles.clone()) { self.player[0] += 1 }}

    fn move_player(&mut self) {
        let direction: u8 = self.current_direction;
        match direction {
            0 => self.move_up(),
            1 => self.move_left(),
            2 => self.move_down(),
            3 => self.move_right(),
            _ => {}
        }
    }

    fn finished(&self) {
        let mut stdout = io::stdout();
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        stdout.flush().unwrap();
        for _i in 0..12 {
            thread::sleep(Duration::from_millis(100));
            println!("{}", Colorize::bright_blue("#######################").to_string());
            thread::sleep(Duration::from_millis(100));
            print!("{}", Colorize::bright_blue("## ").to_string());
            print!("{}", Colorize::bright_yellow("Y O U - @ - W O N").to_string());
            println!("{}", Colorize::bright_blue(" ##").to_string());
        }
        println!("{}", Colorize::bright_blue("#######################").to_string());
    }
}


fn prepare_game() {
    let map_arr: [String; 24] = [
        String::from("#################################################"),
        String::from("# . . . . . . . . . . . # . . . . . . . . . . . #"),
        String::from("# . ##### . ######### . # . ######### . ##### . #"),
        String::from("# . ##### . ######### . # . ######### . ##### . #"),
        String::from("# . . . . . . . . . . . . . . . . . . . . . . . #"),
        String::from("# . ##### . ### . ############# . ### . ##### . #"),
        String::from("# . . . . . ### . . . . # . . . . ### . . . . . #"),
        String::from("######### . ######### . # . ######### . #########"),
        String::from("        # . ######### . # . ######### . #        "),
        String::from("        # . ###                   ### . #        "),
        String::from("######### . ###   #####   #####   ### . #########"),
        String::from("          .       #           #       .          "),
        String::from("######### . ###   #           #   ### . #########"),
        String::from("        # . ###   #############   ### . #        "),
        String::from("        # . ###                   ### . #        "),
        String::from("######### . ###   #############   ### . #########"),
        String::from("# . . . . . . . . . . . # . . . . . . . . . . . #"),
        String::from("# . ##### . ######### . # . ######### . ##### . #"),
        String::from("# . . . # . . . . . . . . . . . . . . . # . . . #"),
        String::from("##### . # . # . ################# . # . # . #####"),
        String::from("# . . . . . # . . . . . # . . . . . # . . . . . #"),
        String::from("# . ################# . # . ################# . #"),
        String::from("# . . . . . . . . . . . . . . . . . . . . . . . #"),
        String::from("#################################################")
    ];
    let map_calc = MapCalculator { map: map_arr };

    let obstacle_coordinates: [Vec<u8>; 24] = map_calc.calculate_map('#');
    let coin_coordinates: [Vec<u8>; 24] = map_calc.calculate_map('.');

    let player_coordinates: [u8; 2] = [24, 18]; // [x, y]

    let mut game = Game {
        obstacles: obstacle_coordinates,
        coins: coin_coordinates,
        player: player_coordinates,
        current_direction: 1,
        direction_queue: 1,
        is_finished: false,
    };

    enable_raw_mode().expect("Could not enable raw mode.");

    let frame_duration = Duration::from_millis(120);
    let mut last_frame = Instant::now();

    let mut start_music = Audio::new();
    start_music.add("start", "src/sounds/PacMan_Start_,Music.mp3");
    start_music.play("start");
    start_music.wait();

    loop {
        if event::poll(Duration::from_millis(1)).expect("Something went wrong.") {
            if let Ok(Event::Key(key_event)) = event::read() {
                if key_event.kind == KeyEventKind::Release {
                    match key_event.code {
                        KeyCode::Char('q') => {
                            println!("Exiting...");
                            break;
                        }

                        KeyCode::Tab => {
                            game.finished();
                            break
                        }
                        KeyCode::Char('w') => { game.direction_queue = 0},
                        KeyCode::Char('a') => { game.direction_queue = 1},
                        KeyCode::Char('s') => { game.direction_queue = 2},
                        KeyCode::Char('d') => { game.direction_queue = 3 },
                        _ => {}
                    }
                }
            }
        }

        if last_frame.elapsed() >= frame_duration {
            game.move_player();
            game.queue_checker();
            game.check_position();
            if game.is_finished {
                game.finished();
                return;
            }
            game.draw();
            last_frame = Instant::now();
        }
    }

    disable_raw_mode().expect("Could not disable raw mode.");
}

fn main() {
    prepare_game();
}



// terminal::enable_raw_mode().expect("Could not enable raw terminal mode!");
/*loop {
    if event::poll(Duration::from_millis(500)).expect("Couldn't read input!") {
        if let Ok(Event::Key(key_event)) = event::read() {
            if key_event.kind == KeyEventKind::Release {
                match key_event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('w') => if !contains_array(&_map.obstacles, &mut [_robot.coordinates[0] ,_robot.coordinates[1] - 1]) { _robot.move_forward(-1, "y") },
                    KeyCode::Char('a') => if !contains_array(&_map.obstacles, &mut [_robot.coordinates[0] - 1 ,_robot.coordinates[1]]) { _robot.move_forward(-1, "x") },
                    KeyCode::Char('s') => if !contains_array(&_map.obstacles, &mut [_robot.coordinates[0] ,_robot.coordinates[1] + 1]) { _robot.move_forward(1, "y") },
                    KeyCode::Char('d') => if !contains_array(&_map.obstacles, &mut [_robot.coordinates[0] + 1 ,_robot.coordinates[1]]) { _robot.move_forward(1, "x") },
                    _ => {}
                }
            }
        }
    }
}*/
/*

#################################################
# . . . . . . . . . . . # . . . . . . . . . . . #
# . ##### . ######### . # . ######### . ##### . #
# . ##### . ######### . # . ######### . ##### . #
# . . . . . . . . . . . . . . . . . . . . . . . #
# . ##### . ### . ############# . ### . ##### . #
# . . . . . ### . . . . # . . . . ### . . . . . #
######### . ######### . # . ######### . #########
        # . ######### . # . ######### . #
        # . ###                   ### . #
######### . ###   #####   #####   ### . #########
          .       #           #       .
######### . ###   #           #   ### . #########
        # . ###   #############   ### . #
        # . ###                   ### . #
######### . ###   #############   ### . #########
# . . . . . . . . . . . # . . . . . . . . . . . #
# . ##### . ######### . # . ######### . ##### . #
# . . . # . . . . . . . . . . . . . . . # . . . #
##### . # . # . ################# . # . # . #####
# . . . . . # . . . . . # . . . . . # . . . . . #
# . ################# . # . ################# . #
# . . . . . . . . . . . . . . . . . . . . . . . #
#################################################

 */


/*
OBSTACLE COORDINATES:
[[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48],
[0, 24, 48],
[0, 4, 5, 6, 7, 8, 12, 13, 14, 15, 16, 17, 18, 19, 20, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 40, 41, 42, 43, 44, 48],
[0, 4, 5, 6, 7, 8, 12, 13, 14, 15, 16, 17, 18, 19, 20, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 40, 41, 42, 43, 44, 48],
[0, 48],
[0, 4, 5, 6, 7, 8, 12, 13, 14, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 34, 35, 36, 40, 41, 42, 43, 44, 48],
[0, 12, 13, 14, 24, 34, 35, 36, 48],
[0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14, 15, 16, 17, 18, 19, 20, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 40, 41, 42, 43, 44, 45, 46, 47, 48],
[8, 12, 13, 14, 15, 16, 17, 18, 19, 20, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 40],
[8, 12, 13, 14, 34, 35, 36, 40],
[0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14, 18, 19, 20, 21, 22, 26, 27, 28, 29, 30, 34, 35, 36, 40, 41, 42, 43, 44, 45, 46, 47, 48],
[18, 30],
[0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14, 18, 30, 34, 35, 36, 40, 41, 42, 43, 44, 45, 46, 47, 48],
[8, 12, 13, 14, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 34, 35, 36, 40],
[8, 12, 13, 14, 34, 35, 36, 40],
[0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 34, 35, 36, 40, 41, 42, 43, 44, 45, 46, 47, 48],
[0, 24, 48],
[0, 4, 5, 6, 7, 8, 12, 13, 14, 15, 16, 17, 18, 19, 20, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 40, 41, 42, 43, 44, 48],
[0, 8, 40, 48],
[0, 1, 2, 3, 4, 8, 12, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 36, 40, 44, 45, 46, 47, 48],
[0, 12, 24, 36, 48],
[0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 48],
[0, 48],
[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48]]
*/

/*
Coins:
[[],
[2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46],
[2, 10, 22, 26, 38, 46],
[2, 10, 22, 26, 38, 46],
[2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46],
[2, 10, 16, 32, 38, 46],
[2, 4, 6, 8, 10, 16, 18, 20, 22, 26, 28, 30, 32, 38, 40, 42, 44, 46],
[10, 22, 26, 38],
[10, 22, 26, 38],
[10, 38],
[10, 38],
[10, 38],
[10, 38],
[10, 38],
[10, 38],
[10, 38],
[2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46],
[2, 10, 22, 26, 38, 46],
[2, 4, 6, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 42, 44, 46],
[6, 10, 14, 34, 38, 42],
[2, 4, 6, 8, 10, 14, 16, 18, 20, 22, 26, 28, 30, 32, 34, 38, 40, 42, 44, 46],
[2, 22, 26, 46],
[2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46],
[]]
*/




/*
Map-Calculator

    let map_arr: [&str; 24] = [
        "#################################################",
        "# . . . . . . . . . . . # . . . . . . . . . . . #",
        "# . ##### . ######### . # . ######### . ##### . #",
        "# . ##### . ######### . # . ######### . ##### . #",
        "# . . . . . . . . . . . . . . . . . . . . . . . #",
        "# . ##### . ### . ############# . ### . ##### . #",
        "# . . . . . ### . . . . # . . . . ### . . . . . #",
        "######### . ######### . # . ######### . #########",
        "        # . ######### . # . ######### . #        ",
        "        # . ###                   ### . #        ",
        "######### . ###   #####   #####   ### . #########",
        "          .       #           #       .          ",
        "######### . ###   #           #   ### . #########",
        "        # . ###   #############   ### . #        ",
        "        # . ###                   ### . #        ",
        "######### . ###   #############   ### . #########",
        "# . . . . . . . . . . . # . . . . . . . . . . . #",
        "# . ##### . ######### . # . ######### . ##### . #",
        "# . . . # . . . . . . . . . . . . . . . # . . . #",
        "##### . # . # . ################# . # . # . #####",
        "# . . . . . # . . . . . # . . . . . # . . . . . #",
        "# . ################# . # . ################# . #",
        "# . . . . . . . . . . . . . . . . . . . . . . . #",
        "#################################################"
    ];
    let mut coordinates_vector: [Vec<u8>; 24] = Default::default();

    for (row_idx, row) in map_arr.iter().enumerate() {
        let mut coordinates = Vec::new();
        for (col_idx, ch) in row.chars().enumerate() {
            if ch == '.' {
                coordinates.push(col_idx as u8);
            }
        }

        coordinates_vector[row_idx] = coordinates;
    }

    println!("{:?}", coordinates_vector);
*/