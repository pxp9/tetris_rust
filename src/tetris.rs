#![allow(dead_code)]
//#![allow(unused_imports)]
use oorandom::Rand32;

// Next we need to actually `use` the pieces of ggez that we are going
// to need frequently.
use ggez::conf::FullscreenType;
use ggez::event::{KeyCode, KeyMods};
use ggez::graphics::{Color, DrawParam, Font, PxScale, Text};
use ggez::{event, graphics, timer, Context, GameResult};
use mint::Point2;
// Now we define the pixel size of each tile, which we make 48x48 pixels.
const GRID_CELL_SIZE: i16 = 48;
// 717.0 for 1080p
const INIT_GRID: f32 = (SCREEN_SIZE.0 / 3.0)
    + ((SCREEN_SIZE.0 / 3.0) - (11.0 * 5.0 + 10.0 * ((GRID_CELL_SIZE - 5) as f32))) / 2.0;
const END_GRID: f32 = (SCREEN_SIZE.0 / 3.0) + (11.0 * 5.0 + 11.0 * ((GRID_CELL_SIZE - 5) as f32));
const END_GRID_BOTTOM: f32 = ((GRID_CELL_SIZE) as f32) + 19.0 * (GRID_CELL_SIZE as f32);
// Next we define how large we want our actual window to be by multiplying
// the components of our grid size by its corresponding pixel size.
const SCREEN_SIZE: (f32, f32) = (1920 as f32, 1080 as f32);
const FPS: u32 = 60;
#[derive(Clone, Copy, PartialEq, Debug)]
struct Block {
    x: i16,
    y: i16,
    color: Color,
}
impl Block {
    /// We make a standard helper function so that we can create a new `GridPosition`
    /// more easily.
    pub fn new(x: i16, y: i16, color: Color) -> Self {
        Block { x, y, color }
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Direction {
    /// We create a helper function that will allow us to easily get the inverse
    /// of a `Direction` which we can use later to check if the player should be
    /// able to move the snake in a certain direction.
    pub fn inverse(&self) -> Self {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// We also create a helper function that will let us convert between a
    /// `ggez` `Keycode` and the `Direction` that it represents. Of course,
    /// not every keycode represents a direction, so we return `None` if this
    /// is the case.
    pub fn from_keycode(key: KeyCode) -> Option<Direction> {
        match key {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
enum PieceType {
    T,
    Square,
    Stick,
    LL,
    LR,
    ZL,
    ZR,
}
#[derive(Clone, Copy, PartialEq, Debug)]
struct Figure {
    piece_type: PieceType,
    rotation: Direction,
    blocks: [Block; 4],
}
impl Figure {
    fn new(n: u32) -> Figure {
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));
        let number: u32;
        if n > 7 {
            number = rng.rand_range(0..7);
        } else {
            number = n;
        }
        let blocks: [Block; 4];
        let color: Color;
        let piece_type = match number {
            0 => {
                color = Color::CYAN;
                blocks = [
                    Block::new(957, 96, color),
                    Block::new(957, 48, color),
                    Block::new(1005, 96, color),
                    Block::new(909, 96, color),
                ];
                PieceType::T
            }
            1 => {
                color = Color::BLUE;
                blocks = [
                    Block::new(909, 48, color),
                    Block::new(909, 96, color),
                    Block::new(957, 48, color),
                    Block::new(957, 96, color),
                ];
                PieceType::Square
            }
            2 => {
                color = Color::RED;
                blocks = [
                    Block::new(909, 144, color),
                    Block::new(909, 96, color),
                    Block::new(909, 192, color),
                    Block::new(909, 48, color),
                ];

                PieceType::Stick
            }
            3 => {
                color = Color::MAGENTA;
                blocks = [
                    Block::new(909, 144, color),
                    Block::new(909, 48, color),
                    Block::new(909, 96, color),
                    Block::new(957, 144, color),
                ];
                PieceType::LR
            }
            4 => {
                color = Color::GREEN;
                blocks = [
                    Block::new(957, 144, color),
                    Block::new(957, 48, color),
                    Block::new(957, 96, color),
                    Block::new(909, 144, color),
                ];
                PieceType::LL
            }
            5 => {
                color = Color::YELLOW;
                blocks = [
                    Block::new(957, 96, color),
                    Block::new(957, 48, color),
                    Block::new(909, 48, color),
                    Block::new(1005, 96, color),
                ];
                PieceType::ZL
            }
            6 => {
                color = Color::from_rgba(255, 128, 0, 255);
                blocks = [
                    Block::new(909, 96, color),
                    Block::new(909, 48, color),
                    Block::new(957, 48, color),
                    Block::new(861, 96, color),
                ];
                PieceType::ZR
            }
            _ => panic!("wtf is this ?"),
        };
        Figure {
            piece_type: piece_type,
            rotation: Direction::Up,
            blocks: blocks,
        }
    }
    fn some_block_is_in_y(&self, y: f32) -> bool {
        let len = self
            .blocks
            .iter()
            .filter(|block| -> bool {
                let n: f32 = block.y as f32;
                n == y
            })
            .collect::<Vec<&Block>>()
            .len();
        len != 0
    }

    fn ilegal_coords(x: f32, y: f32) -> bool {
        x < INIT_GRID.floor() || x > END_GRID.floor() || y > END_GRID_BOTTOM.floor()
    }

    fn legal_move(&self) -> bool {
        let len = self
            .blocks
            .iter()
            .filter(|block| -> bool {
                let x: f32 = block.x as f32;
                let y: f32 = block.y as f32;
                Self::ilegal_coords(x, y)
            })
            .collect::<Vec<&Block>>()
            .len();
        len == 0
    }
    fn restore_blocks(&mut self, prev_blocks: [Block; 4], dir: Direction) {
        for n in 0..self.blocks.len() {
            self.blocks[n] = prev_blocks[n];
        }
        self.rotation = dir;
    }
    fn rotate(&mut self) {
        let center_block = self.blocks[0];
        let prev_blocks = self.blocks;
        match self.piece_type {
            PieceType::T => match self.rotation {
                Direction::Up => {
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - GRID_CELL_SIZE;
                    self.rotation = Direction::Right;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Up);
                    }
                }
                Direction::Down => {
                    self.blocks[1].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x;
                    self.blocks[3].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.rotation = Direction::Left;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Down);
                    }
                }
                Direction::Left => {
                    self.blocks[1].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y;
                    self.rotation = Direction::Up;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Left);
                    }
                }
                Direction::Right => {
                    self.blocks[1].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - GRID_CELL_SIZE;
                    self.rotation = Direction::Down;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Right);
                    }
                }
            },
            PieceType::Square => {}
            PieceType::Stick => match self.rotation {
                Direction::Up => {
                    self.blocks[1].x = center_block.x - 2 * GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[2].y = center_block.y;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y;
                    self.rotation = Direction::Left;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Up);
                    }
                }
                Direction::Down => {
                    self.blocks[1].x = center_block.x + 2 * GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[2].y = center_block.y;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y;
                    self.rotation = Direction::Right;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Down);
                    }
                }
                Direction::Left => {
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x;
                    self.blocks[3].y = center_block.y + GRID_CELL_SIZE;
                    self.rotation = Direction::Down;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Left);
                    }
                }
                Direction::Right => {
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x;
                    self.blocks[3].y = center_block.y + GRID_CELL_SIZE;
                    self.rotation = Direction::Up;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Right);
                    }
                }
            },
            PieceType::LR => match self.rotation {
                Direction::Up => {
                    self.blocks[1].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[2].y = center_block.y;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - GRID_CELL_SIZE;
                    self.rotation = Direction::Left;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Up);
                    }
                }
                Direction::Down => {
                    self.blocks[1].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[2].y = center_block.y;
                    self.blocks[3].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y + GRID_CELL_SIZE;
                    self.rotation = Direction::Right;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Down);
                    }
                }
                Direction::Left => {
                    self.blocks[0].x = center_block.x;
                    self.blocks[0].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.rotation = Direction::Down;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Left);
                    }
                }
                Direction::Right => {
                    self.blocks[0].x = center_block.x;
                    self.blocks[0].y = center_block.y + GRID_CELL_SIZE;
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y + GRID_CELL_SIZE;
                    self.rotation = Direction::Up;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Right);
                    }
                }
            },
            PieceType::LL => match self.rotation {
                Direction::Up => {
                    self.blocks[1].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[2].y = center_block.y;
                    self.blocks[3].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - GRID_CELL_SIZE;
                    self.rotation = Direction::Left;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Up);
                    }
                }
                Direction::Down => {
                    self.blocks[1].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[2].y = center_block.y;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y + GRID_CELL_SIZE;
                    self.rotation = Direction::Right;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Down);
                    }
                }
                Direction::Left => {
                    self.blocks[0].x = center_block.x;
                    self.blocks[0].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.rotation = Direction::Down;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Left);
                    }
                }
                Direction::Right => {
                    self.blocks[0].x = center_block.x;
                    self.blocks[0].y = center_block.y + GRID_CELL_SIZE;
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y + GRID_CELL_SIZE;
                    self.rotation = Direction::Up;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Right);
                    }
                }
            },
            PieceType::ZL => match self.rotation {
                Direction::Up => {
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[2].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.rotation = Direction::Left;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Up);
                    }
                }
                Direction::Down => {
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[2].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.rotation = Direction::Right;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Down);
                    }
                }
                Direction::Left => {
                    self.blocks[1].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - GRID_CELL_SIZE;
                    self.rotation = Direction::Down;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Left);
                    }
                }
                Direction::Right => {
                    self.blocks[1].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - GRID_CELL_SIZE;
                    self.rotation = Direction::Up;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Right);
                    }
                }
            },
            PieceType::ZR => match self.rotation {
                Direction::Up => {
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[2].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.rotation = Direction::Left;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Up);
                    }
                }
                Direction::Down => {
                    self.blocks[1].x = center_block.x;
                    self.blocks[1].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[2].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - 2 * GRID_CELL_SIZE;
                    self.rotation = Direction::Right;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Down);
                    }
                }
                Direction::Left => {
                    self.blocks[1].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - GRID_CELL_SIZE;
                    self.rotation = Direction::Down;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Left);
                    }
                }
                Direction::Right => {
                    self.blocks[1].x = center_block.x - GRID_CELL_SIZE;
                    self.blocks[1].y = center_block.y;
                    self.blocks[2].x = center_block.x;
                    self.blocks[2].y = center_block.y - GRID_CELL_SIZE;
                    self.blocks[3].x = center_block.x + GRID_CELL_SIZE;
                    self.blocks[3].y = center_block.y - GRID_CELL_SIZE;
                    self.rotation = Direction::Up;
                    if !self.legal_move() {
                        self.restore_blocks(prev_blocks, Direction::Right);
                    }
                }
            },
        }
    }
}
#[derive(Clone, PartialEq, Debug)]
struct GameState {
    actual_figure: Option<Figure>,
    keep_figure: Option<Figure>,
    gameover: bool,
    counter: u8,
    pause: bool,
    score: u32,
    // blocks on board
    static_blocks: Vec<Block>,
    next_figures: Vec<Figure>,
}
impl GameState {
    /// Our new function will set up the initial state of our game.
    pub fn new() -> Self {
        GameState {
            actual_figure: None,
            keep_figure: None,
            gameover: false,
            score: 0,
            counter: 0,
            pause: false,
            static_blocks: vec![],
            next_figures: vec![
                Figure::new(8),
                Figure::new(8),
                Figure::new(8),
                Figure::new(8),
                Figure::new(8),
                Figure::new(8),
                Figure::new(8),
            ],
        }
    }
}
fn ilegal_move(static_blocks: &Vec<Block>, fig: &Figure) -> bool {
    let mut is_ilegal = false;
    let mut iter1 = static_blocks.iter().peekable();
    let mut iter2 = fig.blocks.iter().peekable();
    while matches!(iter1.peek(), Some(_)) && !is_ilegal {
        let block = iter1.next().unwrap();
        while matches!(iter2.peek(), Some(_)) && !is_ilegal {
            let block_fig = iter2.next().unwrap();
            if block_fig.y == block.y && block.x == block_fig.x {
                is_ilegal = true;
                print!("Not legal :/\n");
            }
        }
        iter2 = fig.blocks.iter().peekable();
    }
    is_ilegal
}
impl event::EventHandler<ggez::GameError> for GameState {
    /// Update will happen on every frame before it is drawn. This is where we update
    /// our game state to react to whatever is happening in the game world.
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Rely on ggez's built-in timer for deciding when to update the game, and how many times.
        // If the update is early, there will be no cycles, otherwises, the logic will run once for each
        // frame fitting in the time since the last update.
        while timer::check_update_time(ctx, FPS) {
            // We check to see if the game is over. If not, we'll update. If so, we'll just do nothing.
            let mut iter1 = self.static_blocks.iter().peekable();
            while matches!(iter1.peek(), Some(_)) && !self.gameover {
                let block = iter1.next().unwrap();
                if block.y <= GRID_CELL_SIZE {
                    self.gameover = true;
                }
            }
            if self.gameover {
                return Ok(());
            }
            if let None = self.actual_figure {
                self.actual_figure = Some(self.next_figures.remove(0));
                self.next_figures.push(Figure::new(8));
            } else {
                let prev = self.actual_figure.unwrap();
                let fig = self.actual_figure.as_mut().unwrap();
                if self.counter >= 60 && !self.pause {
                    for block in fig.blocks.iter_mut() {
                        block.y += 48;
                    }
                    if ilegal_move(&self.static_blocks, fig) {
                        fig.restore_blocks(prev.blocks, fig.rotation);
                    }
                    self.counter = 0;
                }
                if !self.pause {
                    self.counter += 1;
                }
                if fig.some_block_is_in_y(END_GRID_BOTTOM) {
                    for block in fig.blocks.into_iter() {
                        self.static_blocks.push(block);
                    }
                    self.actual_figure = Some(self.next_figures.remove(0));
                    self.next_figures.push(Figure::new(8));
                } else {
                    let b = self.static_blocks.clone();
                    let mut iter1 = b.iter().peekable();
                    let mut iter2 = fig.blocks.iter().peekable();
                    let mut not_added = false;
                    while matches!(iter1.peek(), Some(_)) && !not_added {
                        let block = iter1.next().unwrap();
                        while matches!(iter2.peek(), Some(_)) && !not_added {
                            let block_fig = iter2.next().unwrap();
                            if block_fig.x == block.x && block.y - GRID_CELL_SIZE == block_fig.y {
                                for block in fig.blocks.into_iter() {
                                    self.static_blocks.push(block);
                                }
                                not_added = true;
                            }
                        }
                        iter2 = fig.blocks.iter().peekable();
                    }
                    if not_added {
                        self.actual_figure = Some(self.next_figures.remove(0));
                        self.next_figures.push(Figure::new(8));
                    }
                }
                let b = self.static_blocks.clone();
                let mut iter1 = b.iter().peekable();
                while matches!(iter1.peek(), Some(_)) {
                    let block = iter1.next().unwrap();
                    let n = self
                        .static_blocks
                        .iter()
                        .filter(|bl| block.y == bl.y)
                        .collect::<Vec<&Block>>()
                        .len();
                    if n == 10 {
                        self.score += 1;
                        self.static_blocks.retain(|bl| block.y != bl.y);
                        for bl in self.static_blocks.iter_mut() {
                            if bl.y < block.y {
                                bl.y = bl.y + GRID_CELL_SIZE;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// draw is where we should actually render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // First we clear the screen to a nice (well, maybe pretty glaring ;)) green
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        /*let mut init_pos = 640.0;
        let mut init_pos_vert = 48.0;
        for _n in 0..20 {
            for _n in 0..10 {
                let rect = graphics::Rect::new(init_pos, init_pos_vert, 43.0, 43.0);
                let r1 = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rect,
                    Color::BLUE,
                )?;
                graphics::draw(ctx, &r1, DrawParam::default())?;
                init_pos += 48.0;
            }
            init_pos = 640.0;
            init_pos_vert += 48.0;
        }*/

        let mut init_pos = INIT_GRID.floor() - 5.0;
        let mut init_pos_vert = (GRID_CELL_SIZE - 5) as f32;
        for _n in 0..11 {
            let rect = graphics::Rect::new(init_pos, init_pos_vert, 5.0, 965.0);
            let r1 = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                rect,
                Color::new(0.251, 0.251, 0.251, 1.0),
            )?;
            graphics::draw(ctx, &r1, DrawParam::default())?;
            init_pos += GRID_CELL_SIZE as f32;
        }
        init_pos = INIT_GRID.floor() - 5.0;
        for _n in 0..21 {
            let rect = graphics::Rect::new(init_pos, init_pos_vert, 485.0, 5.0);
            let r1 = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                rect,
                Color::new(0.251, 0.251, 0.251, 1.0),
            )?;
            graphics::draw(ctx, &r1, DrawParam::default())?;
            init_pos_vert += GRID_CELL_SIZE as f32;
        }

        if !self.gameover && matches!(self.actual_figure, Some(_)) {
            // this will draw the actual_figure
            let fig = self.actual_figure.as_ref().unwrap();
            for block in fig.blocks.iter() {
                let rect = graphics::Rect::new(block.x as f32, block.y as f32, 43.0, 43.0);
                let r1 = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rect,
                    block.color,
                )?;
                graphics::draw(ctx, &r1, DrawParam::default())?;
            }
            // this will draw the static_blocks
            for block in self.static_blocks.iter() {
                let rect = graphics::Rect::new(block.x as f32, block.y as f32, 43.0, 43.0);
                let r1 = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rect,
                    block.color,
                )?;
                graphics::draw(ctx, &r1, DrawParam::default())?;
            }
            // This will draw the queue of figures
            let right_pos: f32 = (GRID_CELL_SIZE as f32) * 22.0;
            let mut right_pos_y: f32 = (GRID_CELL_SIZE as f32) * 2.0;
            for figure in self.next_figures.iter() {
                for block in figure.blocks {
                    let rect = graphics::Rect::new(
                        ((block.x / 2) as f32) + right_pos,
                        ((block.y / 2) as f32) + right_pos_y,
                        (GRID_CELL_SIZE - 5) as f32 / 2.0,
                        (GRID_CELL_SIZE - 5) as f32 / 2.0,
                    );
                    let r1 = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        rect,
                        block.color,
                    )?;
                    graphics::draw(ctx, &r1, DrawParam::default())?;
                }
                right_pos_y += (GRID_CELL_SIZE) as f32 * 2.5;
            }
            // this will draw the keep_figure
            if let Some(figure) = self.keep_figure {
                let left_pos: f32 = (GRID_CELL_SIZE as f32) * 6.0;
                let right_pos_y: f32 = (GRID_CELL_SIZE as f32) * 2.0;
                for block in figure.blocks {
                    let rect = graphics::Rect::new(
                        ((block.x / 2) as f32) - left_pos,
                        ((block.y / 2) as f32) + right_pos_y,
                        (GRID_CELL_SIZE - 5) as f32 / 2.0,
                        (GRID_CELL_SIZE - 5) as f32 / 2.0,
                    );
                    let r1 = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        rect,
                        block.color,
                    )?;
                    graphics::draw(ctx, &r1, DrawParam::default())?;
                }
            }
            let string = format!("Score : {}", self.score);
            let mut text = Text::new(string);
            //let path = env::current_dir()?.join("resources/Hack_Regular_Nerd_Font.ttf");
            //            let font = Font::new(ctx, "/Hack_Regular_Nerd_Font.ttf").expect("Font not found bro");
            let scale = PxScale::from(32.0);
            text.set_font(Font::default(), scale);
            graphics::draw(
                ctx,
                &text,
                DrawParam::default().dest(Point2 {
                    x: INIT_GRID.floor() - 10.0 * (GRID_CELL_SIZE as f32),
                    y: SCREEN_SIZE.1 / 2.0,
                }),
            )?;
        } else {
            let string = format!(
                "Game Over :(  press R to restart a new game , your score was {}",
                self.score
            );
            let mut text = Text::new(string);
            // Maybe i can put my own custom font with this
            //            let font = Font::new(ctx, "/Hack_Regular_Nerd_Font.ttf").expect("Font not found bro");
            let scale = PxScale::from(32.0);
            let font = Font::default();
            text.set_font(font, scale);
            graphics::draw(
                ctx,
                &text,
                DrawParam::default().dest(Point2 {
                    x: INIT_GRID.floor() - 4.0 * (GRID_CELL_SIZE as f32),
                    y: 1.0,
                }),
            )?;
        }
        // Then we tell the snake and the food to draw themselves
        // Finally we call graphics::present to cycle the gpu's framebuffer and display
        // the new frame we just drew.
        graphics::present(ctx)?;
        // We yield the current thread until the next update
        ggez::timer::yield_now();
        // And return success.
        Ok(())
    }

    /// key_down_event gets fired when a key gets pressed.
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        let mut var_block: i16 = 0;
        if let KeyCode::R = keycode {
            self.actual_figure = None;
            self.static_blocks = vec![];
            self.gameover = false;
            self.keep_figure = None;
            self.score = 0;
        } else if let KeyCode::P = keycode {
            self.pause = !self.pause;
        }
        if !self.gameover {
            if matches!(keycode, KeyCode::C) && matches!(self.actual_figure, Some(_)) {
                match self.keep_figure {
                    None => {
                        let number: u32 = match self.actual_figure.unwrap().piece_type {
                            PieceType::T => 0,
                            PieceType::Square => 1,
                            PieceType::Stick => 2,
                            PieceType::LR => 3,
                            PieceType::LL => 4,
                            PieceType::ZL => 5,
                            PieceType::ZR => 6,
                        };
                        self.keep_figure = Some(Figure::new(number));
                        self.actual_figure = Some(self.next_figures.remove(0));
                        self.next_figures.push(Figure::new(8));
                    }
                    Some(figure) => {
                        let number: u32 = match self.actual_figure.unwrap().piece_type {
                            PieceType::T => 0,
                            PieceType::Square => 1,
                            PieceType::Stick => 2,
                            PieceType::LR => 3,
                            PieceType::LL => 4,
                            PieceType::ZL => 5,
                            PieceType::ZR => 6,
                        };
                        self.keep_figure = Some(Figure::new(number));
                        let number: u32 = match figure.piece_type {
                            PieceType::T => 0,
                            PieceType::Square => 1,
                            PieceType::Stick => 2,
                            PieceType::LR => 3,
                            PieceType::LL => 4,
                            PieceType::ZL => 5,
                            PieceType::ZR => 6,
                        };
                        self.actual_figure = Some(Figure::new(number));
                    }
                }
            }
            if matches!(Direction::from_keycode(keycode), Some(Direction::Up))
                && matches!(self.actual_figure, Some(_))
            {
                let fig = self.actual_figure.as_mut().unwrap();
                let prev_blocks = fig.blocks;
                fig.rotate();
                if ilegal_move(&self.static_blocks, fig) {
                    fig.restore_blocks(prev_blocks, fig.rotation);
                }
            } else if let Some(Direction::Left) = Direction::from_keycode(keycode) {
                var_block = -48;
            } else if let Some(Direction::Right) = Direction::from_keycode(keycode) {
                var_block = 48;
            }
            if matches!(Direction::from_keycode(keycode), Some(Direction::Down))
                && matches!(self.actual_figure, Some(_))
            {
                let fig = self.actual_figure.as_mut().unwrap();
                let prev_blocks = fig.blocks;
                for block in fig.blocks.iter_mut() {
                    block.y += 48;
                }
                self.counter = 0;
                if ilegal_move(&self.static_blocks, fig) {
                    fig.restore_blocks(prev_blocks, fig.rotation);
                }
            }
            if matches!(Direction::from_keycode(keycode), Some(Direction::Right))
                || matches!(Direction::from_keycode(keycode), Some(Direction::Left))
                    && matches!(self.actual_figure, Some(_))
            {
                let mut fig = self.actual_figure.unwrap();
                let prev_fig = fig;
                for block in fig.blocks.iter_mut() {
                    block.x += var_block;
                }
                if ilegal_move(&self.static_blocks, &fig) || !fig.legal_move() {
                    fig.restore_blocks(prev_fig.blocks, fig.rotation);
                }
                self.actual_figure = Some(fig);
            }
        }
    }
}

fn main() -> GameResult {
    // Here we use a ContextBuilder to setup metadata about our game. First the title and author
    let (ctx, events_loop) = ggez::ContextBuilder::new("tetris", "Pepe Márquez")
        .window_setup(ggez::conf::WindowSetup::default().title("Tetris!"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1)
                .borderless(true)
                .fullscreen_type(FullscreenType::True),
        )
        .build()?;

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let state = GameState::new();
    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}
