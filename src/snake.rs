extern mod sdl;
extern mod extra;

use std::hashmap::HashMap;
use std::rand::RngUtil;
use extra::deque::Deque;
use extra::time;

use sdl::event::{UpKey, DownKey, LeftKey, RightKey};

static IMAGE_ROOT: &'static str = "img/";
static SNAKE_MOVE_SPEED: float = 0.035f;
static SCORPION_LIFETIME: float = 4.0f;

// TODO: Still need a good way to do tiles.
// TODO: Store an rng with the engine to use to create additional flowers.
// TODO: Less magic numbers.

struct Point {
    x: int,
    y: int
}

struct Snake {
    facing: Point,
    loc: Point,
    tail: Deque<Point>
}

struct Scorpion {
    loc: Point,
    spawn_time: float
}

struct World {
    // TODO: scorpions and flowers sholud really be sets.
    snakes: ~[Snake],
    flowers: ~[Point],
    scorpions: ~[Scorpion],
    last_move_time: float,
    tiles: [[int,.. 38],.. 50]
}

struct ImageBank {
    images: HashMap<~str, ~sdl::video::Surface>
}

struct Engine {
    running: bool,
    image_bank: ImageBank,
    screen: ~sdl::video::Surface,
    world: ~World
}

impl Point {
    fn new(x: int, y: int) -> Point {
        Point {
            x: x,
            y: y
        }
    }

    fn shift(&self, delta: Point) -> Point {
        Point {
            x: self.x + delta.x,
            y: self.y + delta.y
        }
    }

    fn translate(&mut self, delta: Point) {
        self.x += delta.x;
        self.y += delta.y;
    }

    fn as_rect(self, size: u16) -> sdl::Rect {
        sdl::Rect {
            x: self.x as i16 * (size as i16),
            y: self.y as i16 * (size as i16),
            w: size,
            h: size
        }
    }
}

impl Snake {
    fn new(loc: Point, facing: Point, length: int) -> Snake {
        let mut tail = Deque::new();

        assert!(length > 1);

        for std::int::range(1, length) |i| {
            tail.add_front(loc.shift(Point::new(-i, 0)))
        };

        Snake {
            facing: facing,
            loc: loc,
            tail: tail
        }
    }

    fn cut(&self, position: Point) -> ~[Snake] {
        // Chop a snake into two pieces.
        ~[]
    }
}

impl Scorpion {
    fn new(loc: Point, spawn_time: float) -> Scorpion {
        Scorpion {
            loc: loc,
            spawn_time: spawn_time
        }
    }
}

impl World {
    fn new() -> ~World {
        let mut rng = std::rand::rng();
        let mut flowers = ~[];

        for 7.times {
            flowers.push(Point {
                x: rng.gen_int_range(0, 50),
                y: rng.gen_int_range(0, 38)
            })
        };

        ~World {
            snakes: ~[Snake::new(Point::new(10, 10), Point::new(0, 1), 4)],
            flowers: flowers,
            scorpions: ~[],
            last_move_time: 0.0f,
            tiles: [[0,.. 38],.. 50]
        }
    }

    fn set_snake_facing(&mut self, facing: Point) {
        for self.snakes.mut_iter().advance |snake| {
            snake.facing = facing;
        };
    }
}

impl ImageBank {
    fn new(paths: ~[~str]) -> ImageBank {
        let mut images = HashMap::new();

        for paths.iter().advance |path| {
            // Remove the file extension from the name.
            let name = match path.rfind('.') {
                Some(ndx) => path.slice_chars(0, ndx).to_owned(),
                _ => ~""
            };

            match ImageBank::load_image(IMAGE_ROOT + *path) {
                Some(image) => {
                    images.insert(name, image);
                },
                _ => {}
            };
        };

        ImageBank {
            images: images
        }
    }

    fn load_image(filename: &str) -> Option<~sdl::video::Surface> {
        match sdl::img::load(&std::path::Path(filename)) {
            Ok(image) => {
                match image.display_format() {
                    Ok(image) => Some(image),
                    Err(_) => None
                }
            },
            Err(message) => {
                println(fmt!("Failed to load %s", message));
                None
            }
        }
    }
}

impl Engine {
    fn new() -> Result<~Engine, ~str> {
        if !sdl::init([sdl::InitVideo]) {
            return Err(fmt!("Unable to initialize SDL: %s", sdl::get_error()));
        }

        let maybe_screen = sdl::video::set_video_mode(
            800, 608, 32, [sdl::video::HWSurface], [sdl::video::DoubleBuf]);

        match maybe_screen {
            Ok(screen) => {
                Ok(~Engine {
                    running: true,
                    image_bank: ImageBank::new(~[
                        ~"flower.png",
                        ~"sand.bmp",
                        ~"snake_tail.bmp",
                        ~"snake_head.bmp"
                    ]),
                    screen: screen,
                    world: World::new()
                })
            },
            Err(_) => Err(fmt!("Unable to create surface: %s", sdl::get_error()))
        }
    }

    fn execute(&mut self) {
        while self.running {
            let mut polling = true;

            // TODO: That fail!() needs to die, there has to be a better way.
            while polling {
                match sdl::event::poll_event() {
                    sdl::event::QuitEvent => self.running = false,
                    sdl::event::NoEvent => polling = false,
                    sdl::event::KeyEvent(key, _, _, _) => match key {
                        UpKey | DownKey | LeftKey | RightKey => self.world.set_snake_facing(
                            match key {
                                UpKey => Point::new(0, -1),
                                DownKey => Point::new(0, 1),
                                LeftKey => Point::new(-1, 0),
                                RightKey => Point::new(1, 0),
                                _ => fail!()
                            }),
                        _ => {}
                    },
                    _ => {}
                }
            }
            self.tick();
            self.render();
        }

        self.cleanup();
    }

    fn tick(&mut self) {
        let now = time::precise_time_s();
        let dmove = now - self.world.last_move_time;

        if dmove >= SNAKE_MOVE_SPEED {
            for self.world.snakes.mut_iter().advance |snake| {
                let rem_section = snake.tail.pop_front();

                snake.tail.add_back(snake.loc);
                snake.loc.translate(snake.facing);

                // for self.world.scorpions.iter().advance |scorpion| {
                //     for snake.tail.iter().advance |section| {
                //         if section.x == scorpion.loc.x && section.y == scorpion.loc.y {
                //             snake.cut(section);
                //         }
                //     }
                // }

                let mut eaten_flower = None;

                for self.world.flowers.mut_iter().enumerate().advance |(i, flower)| {
                    if flower.x == snake.loc.x && flower.y == snake.loc.y {
                        snake.tail.add_front(rem_section);
                        eaten_flower = Some(i);
                        break;
                    }
                }

                match eaten_flower {
                    Some(flower) => { self.world.flowers.swap_remove(flower); },
                    _ => {}
                }

                for snake.tail.iter().advance |section| {
                    if section.x == snake.loc.x && section.y == snake.loc.y {
                        // snake.cut
                    }
                }
            }

            self.world.last_move_time = now + SNAKE_MOVE_SPEED;
        };
    }

    fn render(&mut self) {
        // Draw the tiles.
        for self.world.tiles.iter().enumerate().advance |(x, row)| {
            for row.iter().enumerate().advance |(y, _)| {
                self.draw_image(~"sand", Point::new(x as int, y as int));
            }
        }

        // Draw the flowers.
        for self.world.flowers.iter().advance |loc| {
            self.draw_image(~"flower", *loc);
        }

        // Draw the snakes.
        for self.world.snakes.iter().advance |snake| {
            self.draw_image(~"snake_head", snake.loc);

            for snake.tail.iter().advance |sec| {
                self.draw_image(~"snake_tail", *sec);
            };
        }

        self.screen.flip();
    }

    fn cleanup(&self) {
        sdl::quit();
    }

    fn draw_image(&self, image: ~str, dest: Point) -> bool {
        let src_rect = Some(sdl::Rect::new(0, 0, 16, 16));

        self.screen.blit_rect   (
            *self.image_bank.images.get(&image), src_rect, Some(dest.as_rect(16)))
    }
}

fn main() {
    do sdl::start {
        let mut engine = Engine::new();

        match engine {
            Ok(~ref mut engine) => engine.execute(),
            Err(message) => println(message)
        }
    }
}
