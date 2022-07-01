use crate::tetromino::{Tetromino, TetrominoL, TETROMINO_HEIGHT, TETROMINO_WIDTH, TetrominoStraight, get_random_tetromino};
use rand::Rng;
use std::thread::sleep;
use std::time::Duration;
use std::io::{Stdout, Write, stdout};
use crossterm::{
    QueueableCommand,
    cursor,
    style,
    terminal,
    ExecutableCommand,
    event::KeyCode
};
use crossterm::style::{Stylize, StyledContent};
use crossterm::event::{poll, read, Event};
use crate::tetromino;
use std::error::Error;
use std::cmp::{min, max};
use std::time::Instant;
use std::borrow::BorrowMut;

pub const WELL_WIDTH: usize = 14;
pub const WELL_HEIGHT: usize = 20;

macro_rules! cmdline_color_white {
    () => {
        "█".white();
    }
}
macro_rules! cmdline_color_black {
    () => {
        " ".black();
    }
}

pub struct Well {
    grid: [[i32; WELL_WIDTH]; WELL_HEIGHT],
    stdout: Stdout,
    current_tetromino: Tetromino,
    score: i32,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

pub fn random_direction() -> Direction {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..4) {
        0 => Direction::Up,
        1 => Direction::Down,
        2 => Direction::Left,
        _ => Direction::Right,
    }
}

pub trait BoardCommandLine {
    /*
    pub is implied in traits
     */
    fn new() -> Self;
    fn render_edges(&mut self) -> ();
    fn render_score(&mut self, score: i32);
    fn write_to_stdout(&mut self, x: usize, y: usize, style: StyledContent<&str>);
    fn render_tetromino(&mut self, erase: bool) -> ();
    fn render_falling_blocks(&mut self) -> ();
    fn run(&mut self) -> crossterm::Result<()>;
    fn move_tetromino(&mut self, direction: Direction) -> ();
    fn log_grid(&self) -> ();
    fn quit(&mut self) -> ();
}


impl BoardCommandLine for Well {

    /// Creates a new well for command line
    fn new() -> Well {
        let mut stdout = stdout();
        stdout.queue(cursor::Hide);
        stdout.execute(terminal::Clear(terminal::ClearType::All));
        let mut result = Well {
            // |<---------- 12 --------->| plus 2 chars to display edge of wells = 14 x 20
            // where the well is of height 18 with two lines for the top (if needed) and bottom
            grid: [[0; WELL_WIDTH]; WELL_HEIGHT],
            stdout: stdout,
            current_tetromino: get_random_tetromino(),
            score: 0,
        };
        result.render_edges();
        result.render_score(result.score);

        return result;
    }

    fn render_edges(&mut self) -> () {
        // paint the outline of the board
        let mut output = cmdline_color_black!();
        for x in 0..WELL_WIDTH {
            for y in 0..WELL_HEIGHT {
                if y == 0 || y == WELL_HEIGHT - 1 {
                    output = cmdline_color_white!();
                    self.grid[y][x] = 1;
                }
                else if x == 0 || x == WELL_WIDTH - 1 {
                    output = cmdline_color_white!();
                    self.grid[y][x] = 1;
                } else {
                    output = cmdline_color_black!();
                    self.grid[y][x] = 0;
                }
                self.write_to_stdout(x, y, output);
                self.stdout.flush();
            }
        }

    }

    fn render_score(&mut self, score: i32) {
        let x = 100 + (WELL_WIDTH / 3) as i32;
        let y = 8;
        self.stdout.queue(cursor::MoveTo((x) as u16, (y) as u16)); // must be reversed
        let current_score = format!("Score: {}", score);
        self.stdout.queue(style::Print(current_score));
    }

    fn write_to_stdout(&mut self, x: usize, y: usize, style: StyledContent<&str>) {
        let x_offset = 100;
        let y_offset = 10;
        self.stdout.queue(cursor::MoveTo((x+x_offset) as u16, (y+y_offset) as u16)); // must be reversed
        self.stdout.queue(style::PrintStyledContent(style));
    }

    /// Render the tetromino 4x4 grid onto the tetris well
    /// Only the grid's walls and stuck tetrominos are marked as 1
    /// empty spaces, including the current tetromino, are left as 0 on the grid
    /// until they are stuck
    /// 2 on grid means tetrominomo is not stuck
    /// 1 on grid corresponds with a border or a stuck tetromino
    fn render_tetromino(&mut self, erase: bool) -> () {

        let x_min = self.current_tetromino.x;
        let x_max = self.current_tetromino.x + TETROMINO_WIDTH;
        let y_min = self.current_tetromino.y;
        let y_max = self.current_tetromino.y + TETROMINO_HEIGHT;
        for x in x_min..x_max {
            for y in y_min..y_max {
                let yy = max(0, y - self.current_tetromino.y);
                let xx = max(0, x - self.current_tetromino.x);
                if !erase && self.current_tetromino.area[yy][xx] == 1 {
                    self.grid[y][x] = 2;
                    self.write_to_stdout(x, y, cmdline_color_white!());
                } else {
                    if y > 0 && y < WELL_HEIGHT - 1 && x > 0 && x < WELL_WIDTH - 1
                        && self.grid[y][x] == 2 {
                        self.grid[y][x] = 0;
                        self.write_to_stdout(x, y, cmdline_color_black!());
                    }
                }
                self.stdout.flush();
            }
        }
        self.log_grid();
    }

    /// Check if any row is full, if so, clear it and let blocks above fall down
    fn render_falling_blocks(&mut self) -> () {
        let mut blocks_falling: bool = false;
        for y in 1..self.grid.len()-1 {
            if self.grid[y] == [1; WELL_WIDTH] {
                blocks_falling = true;
                log::info!("Clearing row {}", y);
                self.score += 100;
                self.render_score(self.score);
                self.grid[y] = [0; WELL_WIDTH];
                self.grid[y][0] = 1;
                self.grid[y][WELL_WIDTH-1] = 1;
                for x in 1..self.grid[y].len()-1 {
                    self.write_to_stdout(x, y, cmdline_color_black!());
                }
                self.stdout.flush();
            }
        }
        while blocks_falling {
            // let blocks fall down
            blocks_falling = false;
            for y in 2..self.grid.len()-1 {
                for x in 1..self.grid[y].len()-1 {
                    if self.grid[y-1][x] == 1 && self.grid[y][x] == 0 {
                        self.grid[y-1][x] = 0;
                        self.grid[y][x] = 1;
                        self.write_to_stdout(x, y - 1, cmdline_color_black!());
                        self.write_to_stdout(x, y, cmdline_color_white!());
                        self.stdout.flush();
                        blocks_falling = true;
                    }
                }
            }
        }
    }

    /// Gradually increases the refresh rate, moving the tetromino down a block faster with each
    /// finished epoch.
    fn run(&mut self) -> crossterm::Result<()> {
        let mut last_instant = Instant::now();
        loop {
            let current_instant= Instant::now();
            if current_instant.duration_since(last_instant) > Duration::from_secs(1) {
                last_instant = current_instant;
                self.move_tetromino(Direction::Down);
            }
            if poll(Duration::from_millis(5))? {
                match read().unwrap() {
                    Event::Key(event) => {
                        if event.code == KeyCode::Char('q') {
                            self.stdout.execute(terminal::Clear(terminal::ClearType::FromCursorUp));
                            // exit
                            return Ok(());
                        }
                        else if event.code == KeyCode::Left || event.code == KeyCode::Char('h') {
                            self.move_tetromino(Direction::Left);
                        }
                        else if event.code == KeyCode::Right || event.code == KeyCode::Char('l') {
                            self.move_tetromino(Direction::Right);
                        }
                        else if event.code == KeyCode::Down || event.code == KeyCode::Char('j') {
                            self.move_tetromino(Direction::Down);
                        }
                        else if event.code == KeyCode::Up || event.code == KeyCode::Char('k') {
                            self.move_tetromino(Direction::Up);
                        }
                        else if event.code == KeyCode::Char('r') {
                            self.render_tetromino(true);
                            let mut i = 0;
                            loop {
                                self.current_tetromino.rotate(false);
                                if (!self.current_tetromino
                                    .will_collide(self.grid, 0, 0)) || i == 4 {
                                    break;
                                }
                                i += 1;
                            }
                            self.render_tetromino(false);
                        }
                    },
                    Event::Mouse(event) => {
                        log::info!("{:?}", event);
                    },
                    Event::Resize(width, height) => {
                        log::info!("New size {}x{}", width, height);
                    },
                }
            }
        }
    }

    fn move_tetromino(&mut self, direction: Direction) -> () {
        self.render_tetromino(true);
        match direction {
            Direction::Left => {
                if !self.current_tetromino.will_collide(self.grid, -1, 0) {
                    self.current_tetromino.x -= 1;
                }
            }
            Direction::Right => {
                if !self.current_tetromino.will_collide(self.grid, 1, 0) {
                    self.current_tetromino.x += 1;
                }
            }
            Direction::Down => {
                if !self.current_tetromino.will_collide(self.grid, 0, 1) {
                    self.current_tetromino.y += 1;
                }
            }
            Direction::Up => {
                // if !self.current_tetromino.will_collide(self.grid, 0, -1) {
                //     self.current_tetromino.y -= 1;
                // }
            }
        }
        self.render_tetromino(false);
        log::info!("Current position ({},{})", self.current_tetromino.x, self.current_tetromino.y);
        if self.current_tetromino.is_stuck(self.grid) {
            self.current_tetromino.stick_to_grid(&mut self.grid);
            log::info!("Current tetromino is stuck!");
            self.current_tetromino = get_random_tetromino();
        }
        self.render_falling_blocks();
    }

    fn log_grid(&self) -> () {
        log::info!("Grid: ");
        for x in 0..self.grid.len() {
            log::info!("{:?}", self.grid[x]);
        }
    }

    fn quit(&mut self) -> () {

        self.stdout.flush();
    }
}

pub trait BoardBrowser {
    /*
    pub is implied in traits
     */
    fn render() -> ();
}

impl BoardBrowser for Well {
    fn render() -> () {

    }
}
