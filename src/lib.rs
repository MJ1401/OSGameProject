#![cfg_attr(not(test), no_std)]

use pluggable_interrupt_os::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color, plot_num, plot_str};
use pc_keyboard::{DecodedKey, KeyCode};
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rand::RngCore;

// Mostly from Dr. Ferrer in class 3/13 and 3/15

const SHOOT_FREQ: isize = 20;
const ADD_SHOOTER_FREQ: isize = 250;
const MOVE_SHOOT_FREQ: isize = 5;

const WALLS: &str = "################################################################################
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
################################################################################";

pub enum Status {
    Normal,
    Over,
}

// Modified from class on 3/13
pub struct Game {
    player: Player,
    walls: Walls,
    tick_count: isize,
    shooters: [Shooter; 100],
    projectiles: [Projectile; 1000],
    proj_count: isize,
    shot_count: isize,
    // From https://stackoverflow.com/questions/67627335/how-do-i-use-the-rand-crate-without-the-standard-library
    rng: SmallRng,
    status: Status,
}

impl Game {
    pub fn new() -> Self {
        Self {player: Player::new(), walls: Walls::new(WALLS), tick_count: 0, 
            shooters: [Shooter::new(); 100], projectiles: [Projectile::new(); 1000], 
            proj_count: 0, shot_count: 0, rng: SmallRng::seed_from_u64(6), status: Status::Normal}
    }

    pub fn key(&mut self, key: DecodedKey) {
        match self.status {
            Status::Normal => match key {
                DecodedKey::RawKey(key) => {
                    let mut future = self.player;
                    match key {
                        KeyCode::ArrowDown => {
                            future.down();
                        }
                        KeyCode::ArrowLeft => {
                            future.left();
                        }
                        KeyCode::ArrowRight => {
                            future.right();
                        }
                        KeyCode::ArrowUp => {
                            future.up();
                        }
                        KeyCode::R => {
                            self.reset_game();
                        }
                        _ => {}
                    }
                    if !future.is_colliding(&self.walls) {
                        self.player = future;
                    }
                },
                DecodedKey::Unicode(_) => {}
            },
            Status::Over => {
                match key {
                    DecodedKey::RawKey(KeyCode::S) | DecodedKey::Unicode('s') => self.reset_game(),
                    _ => {}
                }
            },
        }
        
    }

    pub fn reset_game(&mut self) {
        self.status = Status::Normal;
        self.player = Player::new();
        self.walls = Walls::new(WALLS);
        self.tick_count = 0;
        self.shooters = [Shooter::new(); 100];
        self.projectiles = [Projectile::new(); 1000];
        self.proj_count = 0;
        self.shot_count = 0;
        self.rng = SmallRng::seed_from_u64(6);
    }

    pub fn add_proj(&mut self) {
        self.proj_count += 1;
        self.proj_count %= 999;
    }

    pub fn add_shot(&mut self) {
        self.shot_count += 1;
        self.shot_count %= 99;
    }

    pub fn tick(&mut self) {
        if self.tick_count % ADD_SHOOTER_FREQ == 0 {
            let nx = 1 + self.rng.next_u32() as usize % (BUFFER_WIDTH - 1);
            let ny = 1 + self.rng.next_u32() as usize % (BUFFER_HEIGHT - 1);
            self.shooters[self.shot_count as usize].move_to(nx, ny);
            self.add_shot();
        }
        self.tick_count += 1;
        if self.tick_count % SHOOT_FREQ == 0 {

        }
        if self.tick_count % MOVE_SHOOT_FREQ == 0 {

        }
        self.walls.draw();
        plot('*', self.player.x, self.player.y, ColorCode::new(Color::Green, Color::Black));
        for shot in self.shooters {
            if shot.x < 80 && shot.y < 25 {
                shot.draw();
                self.projectiles[self.proj_count as usize] = shot.shoot_down();
                self.add_proj();
                self.projectiles[self.proj_count as usize] = shot.shoot_left();
                self.add_proj();
                self.projectiles[self.proj_count as usize] = shot.shoot_up();
                self.add_proj();
                self.projectiles[self.proj_count as usize] = shot.shoot_right();
                self.add_proj();
            }
        }
        for mut proj in self.projectiles {
            if proj.x < 80 && proj.y < 25 {
                proj.momentum(self.walls);
            }
        }
        for proj in self.projectiles {
            if proj.x < 80 && proj.y < 25 {
                proj.draw();
            }
        }
        for proj in self.projectiles {
            if proj.x < 80 && proj.y < 25 {
                if self.player.proj_collision(&proj) {
                    self.status = Status::Over;
                }
            }
        }
        plot_num(self.tick_count, 7, 0, ColorCode::new(Color::White, Color::Black));
        plot_str("Score:", 1, 0, ColorCode::new(Color::White, Color::Black));
        match self.status {
            Status::Normal => {},
            Status::Over => {
                self.tick_count -= 1;
                plot_str("Game Over! Press 's' to restart", BUFFER_HEIGHT / 2, 0, ColorCode::new(Color::White, Color::Black));
            },
        }
    }
}

#[derive(Copy, Clone)]
pub struct Walls {
    walls: [[bool; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

impl Walls {
    pub fn new(map: &str) -> Self {
        let mut walls = [[false; BUFFER_WIDTH]; BUFFER_HEIGHT];
        for (row, chars) in map.split('\n').enumerate() {
            for (col, value) in chars.char_indices() {
                walls[row][col] = value == '#';
            }
        }
        Self {walls}
    }

    pub fn draw(&self) {
        for row in 0..self.walls.len() {
            for col in 0..self.walls[row].len() {
                plot(self.char_at(row, col), col, row, ColorCode::new(Color::Blue, Color::Black));
            }
        }
    }

    pub fn occupied(&self, row: usize, col: usize) -> bool {
        if row > 24 || col > 79 {
            return true;
        }
        self.walls[row][col]
    }

    fn char_at(&self, row: usize, col: usize) -> char {
        if self.walls[row][col] {
            '#'
        } else {
            ' '
        }
    }
}

#[derive(Copy, Clone)]
pub struct Projectile {
    x: usize,
    y: usize,
    dir: usize, 
}

impl Projectile {
    pub fn new() -> Self {
        Self {x: 100, y: 100, dir: 0}
    }

    pub fn draw(&self) {
        plot('X', self.x, self.y, ColorCode::new(Color::Yellow, Color::Black));
    }

    pub fn momentum(&mut self, walls: Walls) {
        if self.dir == 0 && !walls.occupied(self.x+1, self.y){
            self.x += 1;
        } else if self.dir == 1 && !walls.occupied(self.x, self.y+1){
            self.y += 1;
        } else if self.dir == 2 && !walls.occupied(self.x-1, self.y){
            self.x -= 1;
        } else if !walls.occupied(self.x, self.y-1){
            self.y -= 1;
        } else {
            self.remove();
        }
    }

    pub fn occupied(&self, row: usize, col: usize) -> bool {
        if row > 24 || col > 79 {
            return true;
        } else if self.x == col && self.y == row {
            return true;
        } else {
            return false;
        }
    }

    pub fn remove(&mut self) {
        self.x = 100;
        self.y = 100;
    }
}

#[derive(Copy, Clone)]
pub struct Shooter {
    x: usize, 
    y: usize,
}

impl Shooter {
    pub fn new() -> Self {
        Self {x: 100, y : 100}
    }

    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    pub fn shoot_right(&self) -> Projectile {
        Projectile { x: (self.x + 1), y: (self.y), dir: (0) }
    }
    pub fn shoot_down(&self) -> Projectile {
        Projectile { x: (self.x), y: (self.y + 1), dir: (1) }
    }
    pub fn shoot_left(&self) -> Projectile {
        Projectile { x: (self.x - 1), y: (self.y), dir: (2) }
    }
    pub fn shoot_up(&self) -> Projectile {
        Projectile { x: (self.x), y: (self.y - 1), dir: (3) }
    }

    pub fn draw(&self) {
        plot('S', self.x, self.y, ColorCode::new(Color::Magenta, Color::Black));
    }

    pub fn shift(&mut self, rng: &mut SmallRng, walls: Walls) {
        let dir = rng.next_u32() % 4;
        if dir == 0 && !walls.occupied(self.x+1, self.y){
            self.x += 1;
        } else if dir == 1 && !walls.occupied(self.x, self.y+1){
            self.y += 1;
        } else if dir == 2 && !walls.occupied(self.x-1, self.y){
            self.x -= 1;
        } else if !walls.occupied(self.x, self.y-1){
            self.y -= 1;
        }
    }
}

#[derive(Copy, Clone)]
pub struct Player {
    x: usize,
    y: usize,
}

impl Player {
    pub fn new() -> Self {
        Self {x: BUFFER_WIDTH / 2, y: BUFFER_HEIGHT / 2}
    }

    pub fn is_colliding(&self, walls: &Walls) -> bool {
        walls.occupied(self.y, self.x)
    }

    pub fn proj_collision(&self, proj: &Projectile) -> bool {
        proj.occupied(self.y, self.x)
    }

    pub fn down(&mut self) {
        self.y += 1;
    }

    pub fn up(&mut self) {
        self.y -= 1;
    }

    pub fn left(&mut self) {
        self.x -= 1;
    }

    pub fn right(&mut self) {
        self.x += 1;
    }
}

// #![cfg_attr(not(test), no_std)]

// use bare_metal_modulo::{ModNumC, MNum, ModNumIterator};
// use pluggable_interrupt_os::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color, is_drawable};
// use pc_keyboard::{DecodedKey, KeyCode};
// use num::traits::SaturatingAdd;

// #[derive(Copy,Debug,Clone,Eq,PartialEq)]
// pub struct LetterMover {
//     letters: [char; BUFFER_WIDTH],
//     num_letters: ModNumC<usize, BUFFER_WIDTH>,
//     next_letter: ModNumC<usize, BUFFER_WIDTH>,
//     col: ModNumC<usize, BUFFER_WIDTH>,
//     row: ModNumC<usize, BUFFER_HEIGHT>,
//     dx: ModNumC<usize, BUFFER_WIDTH>,
//     dy: ModNumC<usize, BUFFER_HEIGHT>
// }

// impl LetterMover {
//     pub fn new() -> Self {
//         LetterMover {
//             letters: ['A'; BUFFER_WIDTH],
//             num_letters: ModNumC::new(1),
//             next_letter: ModNumC::new(1),
//             col: ModNumC::new(BUFFER_WIDTH / 2),
//             row: ModNumC::new(BUFFER_HEIGHT / 2),
//             dx: ModNumC::new(0),
//             dy: ModNumC::new(0)
//         }
//     }

//     fn letter_columns(&self) -> impl Iterator<Item=usize> {
//         ModNumIterator::new(self.col)
//             .take(self.num_letters.a())
//             .map(|m| m.a())
//     }

//     pub fn tick(&mut self) {
//         self.clear_current();
//         self.update_location();
//         self.draw_current();
//     }

//     fn clear_current(&self) {
//         for x in self.letter_columns() {
//             plot(' ', x, self.row.a(), ColorCode::new(Color::Black, Color::Black));
//         }
//     }

//     fn update_location(&mut self) {
//         self.col += self.dx;
//         self.row += self.dy;
//     }

//     fn draw_current(&self) {
//         for (i, x) in self.letter_columns().enumerate() {
//             plot(self.letters[i], x, self.row.a(), ColorCode::new(Color::Cyan, Color::Black));
//         }
//     }

//     pub fn key(&mut self, key: DecodedKey) {
//         match key {
//             DecodedKey::RawKey(code) => self.handle_raw(code),
//             DecodedKey::Unicode(c) => self.handle_unicode(c)
//         }
//     }

//     fn handle_raw(&mut self, key: KeyCode) {
//         match key {
//             KeyCode::ArrowLeft => {
//                 self.dx -= 1;
//             }
//             KeyCode::ArrowRight => {
//                 self.dx += 1;
//             }
//             KeyCode::ArrowUp => {
//                 self.dy -= 1;
//             }
//             KeyCode::ArrowDown => {
//                 self.dy += 1;
//             }
//             _ => {}
//         }
//     }

//     fn handle_unicode(&mut self, key: char) {
//         if is_drawable(key) {
//             self.letters[self.next_letter.a()] = key;
//             self.next_letter += 1;
//             self.num_letters = self.num_letters.saturating_add(&ModNumC::new(1));
//         }
//     }
// }