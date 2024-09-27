use std::{io, thread};
use std::io::Write;
use std::pin::pin;
// use rand::Rng;
use crossterm::{cursor, event::{self, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode}, ExecutableCommand};
use std::time::{Duration, Instant};
use crossterm::event::KeyEventKind;
use colored::*;
use crossterm::terminal::{Clear, ClearType};
use rand::Rng;
use rusty_audio::Audio;


struct Game<'a> {
    map_size: [u8; 2],
    obstacles: Vec<Vec<u8>>,
    coins: Vec<Vec<u8>>,
    is_finished: bool,
    speed_compensation: bool,
    _player: &'a mut Player
}

#[derive(Clone)]
struct Player {
    position: [u8; 2], // x y
    current_direction: u8,
    direction_queue: u8, // 0: w; 1: a; 2: s; 3: d
    hearts: u8,
    frames: u32,
}

struct Ghost {
    position: [u8; 2],
    direction: u8,
    mortal: bool,
    active: bool,
}

struct MapCalculator {
    map: Vec<String>
}


fn random(x: u8, y: u8) -> u8 { // Creates random number between x and y
    let mut rng = rand::thread_rng();
    rng.gen_range(x..y)
}


fn check_position(cur_pos: [u8; 2], obstacles: Vec<Vec<u8>>) -> bool {
    let x = cur_pos[0] as usize;
    let y = cur_pos[1] as usize;
    !obstacles[y].contains(&(x as u8))
}


impl Ghost {

    fn default(&mut self) {
        self.position = [21, 11];
        self.direction = 0;
        self.mortal = false;
        self.active = false;
    }
    fn move_ghost(&mut self, obstacles: &Vec<Vec<u8>>) {
        /*
        Structure:
         - Checks direction (0 - 3) 0 -> up (w); 1 -> left (a); 2 -> down (s); 3 -> right (d)
         - Check whether the next object (y-axis) is an obstacle (or) check whether there is an obstacle 2 positions ahead (x-axis).
            - If no obstacle was found: check whether the ghost can change direction or not
                - If changing the direction is possible: request direction change (not forced)
            - else: change direction
        */
        let random_value: u8 = random(0, 3);

        match self.direction {
            0 => { // UP (W)
                if !obstacles[(self.position[1] - 1) as usize].contains(&self.position[0]) {
                    self.position[1] -= 1;
                } else {
                    self.direction = self.change_direction();
                }
                if !obstacles[self.position[1] as usize].contains(&(self.position[0] + 1)) && random_value == 0{
                    self.direction = 3;
                }
                if !obstacles[self.position[1] as usize].contains(&(self.position[0] - 1)) && random_value == 1 {
                    self.direction = 1;
                }
            }
            1 => { // LEFT (A)
                if !obstacles[self.position[1] as usize].contains(&(&self.position[0] - 2)) {
                    self.position[0] -= 1;
                } else {
                    self.direction = self.change_direction();
                }
                if !obstacles[(self.position[1] + 1) as usize].contains(&(self.position[0])) && random_value == 0 && self.position[0] % 2 == 0 {
                    self.direction = 2;
                }
                if !obstacles[(self.position[1] - 1) as usize].contains(&(self.position[0])) && random_value == 1 && self.position[0] % 2 == 0  {
                    self.direction = 0;
                }
            }
            2 => { // DOWN (S)
                if !obstacles[(self.position[1] + 1) as usize].contains(&self.position[0]) {
                    self.position[1] += 1;
                } else {
                    self.direction = self.change_direction();
                }
                if !obstacles[self.position[1] as usize].contains(&(self.position[0] + 1)) && random_value == 0{
                    self.direction = 3;
                }
                if !obstacles[self.position[1] as usize].contains(&(self.position[0] - 1)) && random_value == 1 {
                    self.direction = 1;
                }
            }
            3 => { // RIGHT (D)
                if !obstacles[self.position[1] as usize].contains(&(&self.position[0] + 2)) {
                    self.position[0] += 1;
                } else {
                    self.direction = self.change_direction();
                }
                if !obstacles[(self.position[1] + 1) as usize].contains(&(self.position[0])) && random_value == 0 && self.position[0] % 2 == 0 {
                    self.direction = 2;
                }
                if !obstacles[(self.position[1] - 1) as usize].contains(&(self.position[0])) && random_value == 1 && self.position[0] % 2 == 0 {
                    self.direction = 0;
                }
            }
            _ => { }
        }
    }

    fn change_direction(&mut self) -> u8 {
        let mut new_direction: u8 = random(0, 4);
        while new_direction == self.direction { new_direction = random(0, 4) }
        new_direction
    }
}


impl MapCalculator {
    fn calculate_map(&self, character: char) -> Vec<Vec<u8>> {
        let mut coordinates_vector: Vec<Vec<u8>> = vec![];
        for (row_idx, row) in self.map.iter().enumerate() {
            coordinates_vector.push(vec![]);
            for (col_idx, ch) in row.chars().enumerate() {
                if ch == character {
                    coordinates_vector[row_idx].push(col_idx as u8);
                }
            }
        }
        coordinates_vector
    }
}


impl Player {
    fn default(&mut self) {
        self.position = [24, 18];
        self.direction_queue = 1;
        self.current_direction = 1;
    }

    fn queue_checker(&mut self, obstacles: &Vec<Vec<u8>>) {
        if !(self.position[0] % 2 == 0) { return; }
        let direction: u8 = self.direction_queue;
        match direction {
            0 => if check_position([self.position[0] , self.position[1] - 1], obstacles.clone()) { self.current_direction = direction }, // W
            1 => if check_position([self.position[0] - 2, self.position[1]], obstacles.clone()) { self.current_direction = direction }, // A
            2 => if check_position([self.position[0] , self.position[1] + 1], obstacles.clone()) { self.current_direction = direction }, // S
            3 => if check_position([self.position[0] + 2, self.position[1]], obstacles.clone()) { self.current_direction = direction }, // D
            _ => {}
        }
    }

    fn move_up(&mut self, obstacles: &Vec<Vec<u8>>) { if check_position([self.position[0] , self.position[1] - 1], obstacles.clone()) { if self.frames % 2 == 0 { self.position[1] -= 1; } }}
    fn move_left(&mut self, obstacles: &Vec<Vec<u8>>) { if check_position([self.position[0] - 2, self.position[1]], obstacles.clone()) { self.position[0] -= 1 }}
    fn move_down(&mut self, obstacles: &Vec<Vec<u8>>) { if check_position([self.position[0] , self.position[1] + 1], obstacles.clone()) { if self.frames % 2 == 0 { self.position[1] += 1; } }}
    fn move_right(&mut self, obstacles: &Vec<Vec<u8>>) { if check_position([self.position[0] + 2, self.position[1]], obstacles.clone()) { self.position[0] += 1 }}

    fn move_player(&mut self, obstacles: &Vec<Vec<u8>>) {
        let direction: u8 = self.current_direction;
        match direction {
            0 => self.move_up(&obstacles),
            1 => self.move_left(&obstacles),
            2 => self.move_down(&obstacles),
            3 => self.move_right(&obstacles),
            _ => {}
        }
    }

    fn check_position(&mut self, coins: &mut Vec<Vec<u8>>)  {
        let x = self.position[0] as usize;
        let y = self.position[1] as usize;

        let initial_length = coins[y].len();
        coins[y].retain(|&coin_x| coin_x != x as u8);

        if coins[y].len() < initial_length {
            thread::spawn( || {
                let mut coin_sound = Audio::new();
                coin_sound.add("coin", "coin_temp.mp3");
                coin_sound.play("coin");
                coin_sound.wait();
            });
        }
    }
}

impl Game<'_> {
    fn default(&mut self, obstacle_coordinates: Vec<Vec<u8>>, coin_coordinates: Vec<Vec<u8>>) {
        self.obstacles = obstacle_coordinates;
        self.coins = coin_coordinates;
    }

    fn draw(&mut self, red_ghost_pos: &[u8; 2], orange_ghost_pos: &[u8; 2], blue_ghost_pos: &[u8; 2], pink_ghost_pos: &[u8; 2],) {
        let mut map : String = Default::default();
        let mut coin_counter: u8 = 0;
        let mut stdout = io::stdout();

        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();

        for y in 0..self.map_size[1] {
            for x in 0..self.map_size[0] {
                if y < self.obstacles.len() as u8 && self.obstacles[y as usize].contains(&x) {
                    map += &Colorize::blue("#").to_string();
                } else if self._player.position == [x, y] {
                    map += &Colorize::bright_yellow("@").to_string();
                } else if *red_ghost_pos == [x, y] {
                    map += &Colorize::red("o").to_string();
                }
                else if *orange_ghost_pos == [x, y] {
                    map += &Colorize::bright_yellow("o").to_string();
                }
                else if *blue_ghost_pos == [x, y] {
                    map += &Colorize::cyan("o").to_string();
                }
                else if *pink_ghost_pos == [x, y] {
                    map += &Colorize::bright_magenta("o").to_string();
                }
                else if y < self.coins.len() as u8 && self.coins[y as usize].contains(&x) {
                    map += "•";
                    coin_counter += 1;
                } else {
                    map += " ";
                }
            }
            map += "\n";
        }
        println!("{}", map);
        if coin_counter == 0 { self.is_finished = true; }
        stdout.flush().unwrap();
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

    fn game_over(&self) {
        let mut stdout = io::stdout();
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        stdout.flush().unwrap();
        println!("{}", Colorize::red("#######################").to_string());
        println!("{}", Colorize::red("Game over.").to_string());
        println!("{}", Colorize::red("#######################").to_string());
    }
}


fn prepare_game() {
    fn calculate_max_map_width(map: Vec<String>) -> u8 {
        let mut max_width: u8 = 0;
        for i in map {
            if i.len() > max_width as usize {
                max_width = i.len() as u8;
            }
        }
        max_width
    }

    let map_arr: Vec<String> = vec![
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
        String::from("#         .       #           #       .         #"),
        String::from("######### . ###   #           #   ### . #########"),
        String::from("        # . ###   #############   ### . #        "),
        String::from("        # . ###                   ### . #        "),
        String::from("######### . ###   #############   ### . #########"),
        String::from("# . . . . . . . . . . . # . . . . . . . . . . . #"),
        String::from("# . ##### . ######### . # . ######### . ##### . #"),
        String::from("# . . . # . . . . . . .   . . . . . . . # . . . #"),
        String::from("##### . # . # . ################# . # . # . #####"),
        String::from("# . . . . . # . . . . . # . . . . . # . . . . . #"),
        String::from("# . ################# . # . ################# . #"),
        String::from("# . . . . . . . . . . . . . . . . . . . . . . . #"),
        String::from("#################################################")
    ];
    let map_calc = MapCalculator { map: map_arr.clone() };

    let obstacle_coordinates: Vec<Vec<u8>> = map_calc.calculate_map('#');
    let coin_coordinates: Vec<Vec<u8>> = map_calc.calculate_map('.');

    let player_coordinates: [u8; 2] = [24, 18]; // [x, y]

    let mut player = Player {
        position: player_coordinates,
        current_direction: 1,
        direction_queue: 1,
        hearts: 3,
        frames: 0,
    };

    let mut _red_ghost = Ghost { position: [21, 11], direction: 0, mortal: false, active: false };
    let mut _orange_ghost = Ghost { position: [24, 11], direction: 0, mortal: false, active: true };
    let mut _blue_ghost = Ghost { position: [27, 11], direction: 0, mortal: false , active: false };
    let mut _pink_ghost = Ghost { position: [24, 12], direction: 0, mortal: false, active: false };

    let mut game = Game {
        map_size: [calculate_max_map_width(map_arr.clone()), map_arr.len() as u8],
        obstacles: obstacle_coordinates.clone(),
        coins: coin_coordinates.clone(),
        is_finished: false,
        speed_compensation: true,
        _player: &mut player
    };

    enable_raw_mode().expect("Could not enable raw mode.");

    let frame_duration = Duration::from_millis(120);
    let mut last_frame = Instant::now();

    // Game sounds
    //let mut start_music = Audio::new();
    //let start_data = include_bytes!("../src/sounds/PacMan_Start_Music.mp3");
    //let temp_path = "start_music.mp3";
    //std::fs::write(temp_path, start_data).unwrap();

    let coin_data = include_bytes!("../src/sounds/collect_coin.mp3");
    std::fs::write("coin_temp.mp3", coin_data).unwrap();

    //start_music.add("start", temp_path);
    //start_music.play("start");
    game.draw(&_red_ghost.position, &_orange_ghost.position, &_blue_ghost.position, &_pink_ghost.position);
    //start_music.wait();
    //std::fs::remove_file(temp_path).unwrap();

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
                        KeyCode::Char('w') | KeyCode::Up => { game._player.direction_queue = 0 },
                        KeyCode::Char('a') | KeyCode::Left => { game._player.direction_queue = 1 },
                        KeyCode::Char('s') | KeyCode::Down => { game._player.direction_queue = 2 },
                        KeyCode::Char('d') | KeyCode::Right => { game._player.direction_queue = 3 },
                        _ => {}
                    }
                }
            }
        }

        if last_frame.elapsed() >= frame_duration {
            let mut remove_heart: bool = false;
            game._player.move_player(&game.obstacles);
            game._player.queue_checker(&game.obstacles);
            game._player.check_position(&mut game.coins);
            _red_ghost.move_ghost(&game.obstacles);
            _orange_ghost.move_ghost(&game.obstacles);
            _blue_ghost.move_ghost(&game.obstacles);
            _pink_ghost.move_ghost(&game.obstacles);
            if _red_ghost.position == game._player.position { remove_heart = true }
            if _orange_ghost.position == game._player.position { remove_heart = true }
            if _blue_ghost.position == game._player.position { remove_heart = true }
            if _pink_ghost.position == game._player.position { remove_heart = true }
            if remove_heart {
                game.default(obstacle_coordinates.clone(), coin_coordinates.clone());
                _red_ghost.default();
                _orange_ghost.default();
                _blue_ghost.default();
                _pink_ghost.default();
                game._player.hearts -= 1;
                game._player.default();
            }
            if game._player.hearts == 0 {
                game.game_over();
                return
            }
            if game.is_finished {
                game.finished();
                return;
            }
            game.draw(&_red_ghost.position, &_orange_ghost.position, &_blue_ghost.position, &_pink_ghost.position);
            if game.speed_compensation { game._player.frames += 1; }
            last_frame = Instant::now();
        }
    }

    disable_raw_mode().expect("Could not disable raw mode.");
    std::fs::remove_file("coin_temp.mp3").unwrap();
}

fn main() {
    prepare_game();
}


// I love writing in Rust because fighting with the compiler is the only form of social interaction I get.



/*
Ghost starting coordinates

[11, 21]
[11, 24]
[11, 27]
[12, 24]

*/


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

[[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48],
[0, 24, 48],
[0, 4, 5, 6, 7, 8, 12, 13, 14, 15, 16, 17, 18, 19, 20, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 40, 41, 42, 43, 44, 48],
[0, 4, 5, 6, 7, 8, 12, 13, 14, 15, 16, 17, 18, 19, 20, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 40, 41, 42, 43, 44, 48],
[0, 48], [0, 4, 5, 6, 7, 8, 12, 13, 14, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 34, 35, 36, 40, 41, 42, 43, 44, 48],
[0, 12, 13, 14, 24, 34, 35, 36, 48],
[0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14, 15, 16, 17, 18, 19, 20, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 40, 41, 42, 43, 44, 45, 46, 47, 48],
[8, 12, 13, 14, 15, 16, 17, 18, 19, 20, 24, 28, 29, 30, 31, 32, 33, 34, 35, 36, 40], [8, 12, 13, 14, 34, 35, 36, 40],
[0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14, 18, 19, 20, 21, 22, 26, 27, 28, 29, 30, 34, 35, 36, 40, 41, 42, 43, 44, 45, 46, 47, 48],
[18, 30], [0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 13, 14, 18, 30, 34, 35, 36, 40, 41, 42, 43, 44, 45, 46, 47, 48],
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
