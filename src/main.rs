extern crate ggez;
extern crate ncollide2d;
extern crate rand;
extern crate nalgebra as na;

mod map;

use std::path::Path;
use std::f32::consts::PI;
use ggez::*;
use na::{Vector2, Isometry2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use ncollide2d::world::{CollisionGroups, CollisionObjectHandle, CollisionWorld, GeometricQueryType};
use rand::{thread_rng, Rng};

use std::time::{Duration, Instant};


const SCREEN_SIZE: (f32, f32) = (
    640.0, 480.0
    );

const UPDATES_PER_SECOND: f32 = 30.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

#[derive(Debug)]
struct Animation {
    spritesheet: graphics::Image,
    width: f32,
    height: f32,
    frames_hor: f32,
    frames_ver: f32,
    curr_frame: f32,
    src_rect: graphics::Rect,
}

impl Animation {
    pub fn new<P>(ctx: &mut Context, filename: P) -> Self    // sta animacija prima kao argument
    where
    P: AsRef<Path>,
    {
        Animation {
            spritesheet: graphics::Image::new(ctx, filename).unwrap(),
            width: 224.0,        // width slike robin_run* u pikselima
            height: 32.0,        // height slike robin_run* u pikselima
            frames_hor: 7.0,
            frames_ver: 1.0,
            curr_frame: 0.0,
            src_rect: graphics::Rect::new(0.0,0.0,1.0/7.0, 1.0),
        }
    }

    fn reset(&mut self) {
        self.curr_frame = 0.0;
        self.src_rect.x = 0.0;
    }

    fn next_frame(&mut self) {
        self.curr_frame = ((self.curr_frame as i32 + 1) % 7) as f32;
        self.src_rect.x = self.curr_frame/self.frames_hor;
        // y ne moramo da pomeramo jer je slika horizontalna
    }

    fn draw(&self,ctx: &mut Context, pos: mint::Point2<f32>) -> GameResult<()> {
        graphics::draw(ctx, &self.spritesheet, graphics::DrawParam::new().src(self.src_rect).dest(pos))?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    Null,
}

#[derive(Debug)]
struct Player {
    pos: mint::Point2<f32>,
    direction: Vector2<f32>,
    collision_ver: Direction,
    collision_hor: Direction,
    walking: bool,
    run_left: Animation,
    run_right: Animation,
    run_down: Animation,
    run_up: Animation,
    idle: graphics::Image,
    animation_state: Direction,
    spd: f32,
    col_handle: CollisionObjectHandle,
}

impl Player {
    pub fn new(ctx: &mut Context, handle: CollisionObjectHandle) -> Self {
        Player {
            pos: mint::Point2 {x: 64.0, y: 64.0},
            direction: Vector2::new(0.0, 0.0),
            collision_ver: Direction::Null,
            collision_hor: Direction::Null,
            walking: false,
            run_left: Animation::new(ctx, "/images/robin_runleft.png"),
            run_right: Animation::new(ctx, "/images/robin_runright.png"),
            run_down: Animation::new(ctx, "/images/robin_rundown.png"),
            run_up: Animation::new(ctx, "/images/robin_runup.png"),
            idle: graphics::Image::new(ctx, "/images/robin_idle.png").unwrap(),
            animation_state: Direction::Null,
            spd: 4.0,
            col_handle: handle,

        }
    }

    fn pos_from_move(&self) -> mint::Point2<f32> {
        // ova f-ja se poziva pri svakom apdejtu
        // na osnovu trenutne pozicije i pravca kretanja
        // vraca "sledecu" poziciju igraca
        let mut norm_dir = Vector2::new(0.0, 0.0);
        if self.direction.x != 0.0 || self.direction.y != 0.0 {
            norm_dir = self.direction.normalize();
        }

        mint::Point2 { x: self.pos.x + norm_dir.x * self.spd, y: self.pos.y + norm_dir.y * self.spd }
    }

    pub fn shape_pos(&self, p: Option<mint::Point2<f32>>) -> Isometry2<f32> {
        // Collider zahteva Isometry2 za poziciju oblika
        match p {
            Some(d) => Isometry2::new(Vector2::new(d.x-1.5, d.y+8.0), 0.0),
            None => Isometry2::new(Vector2::new(self.pos.x-1.5, self.pos.y+8.0), 0.0),
        }
    }

    fn update(&mut self, ctx: &mut Context, world: &mut CollisionWorld<f32, ()>, map_handle: CollisionObjectHandle) {
        /* self.walking je korisno za animaciju
         * npr. if self.walking {
         *          curr_animation = walk_animation;
         *      } else {
         *          curr_animation = idle_animation;
         *      }
         */
        if self.direction.x == 0.0 && self.direction.y == 0.0 {
            self.walking = false;
            self.animation_state = Direction::Null;
            self.run_right.reset();
            self.run_left.reset();
            self.run_down.reset();
            self.run_up.reset();

        } else if self.direction.x == 1.0 && self.direction.y == 0.0 {
            self.animation_state = Direction::Right;
            self.walking = true;
        }
        else if self.direction.x == -1.0 && self.direction.y == 0.0 {
            self.animation_state = Direction::Left;
            self.walking = true;
        }
        else if self.direction.x == 0.0 && self.direction.y == 1.0 {
            self.animation_state = Direction::Down;
            self.walking = true;
        }
        else if self.direction.x == 0.0 && self.direction.y == -1.0 {
            self.animation_state = Direction::Up;
            self.walking = true;
        }
        else { self.walking = true; }
        if self.walking {
            match self.animation_state {
                Direction::Right => self.run_right.next_frame(),
                Direction::Left => self.run_left.next_frame(),
                Direction::Up => self.run_up.next_frame(),
                Direction::Down => self.run_down.next_frame(),
                Direction::Null => (),
            }

            // mora da postoji neki bolji nacin da se ovo uradi
            let mut old_dir = self.direction.clone();
            if self.collision_hor == Direction::Left && input::keyboard::is_key_pressed(ctx, event::KeyCode::Left) {
                self.direction = old_dir;
                self.direction.x = -1.0;
                let new_pos = self.pos_from_move();
                world.set_position(self.col_handle, self.shape_pos(Some(new_pos)));
                match world.contact_pair(self.col_handle, map_handle, true) {
                    Some(_) => (),
                    None => {
                        self.collision_hor = Direction::Null;
                        old_dir.x = -1.0;
                        ()
                    },
                }
            }
            if self.collision_hor == Direction::Right && input::keyboard::is_key_pressed(ctx, event::KeyCode::Right){
                self.direction = old_dir;
                self.direction.x = 1.0;
                let new_pos = self.pos_from_move();
                world.set_position(self.col_handle, self.shape_pos(Some(new_pos)));
                match world.contact_pair(self.col_handle, map_handle, true) {
                    Some(_) => (),
                    None => {
                        self.collision_hor = Direction::Null;
                        old_dir.x = 1.0;
                        ()
                    },
                }
            }
            if self.collision_ver == Direction::Up && input::keyboard::is_key_pressed(ctx, event::KeyCode::Up) {
                self.direction = old_dir;
                self.direction.y = -1.0;
                let new_pos = self.pos_from_move();
                world.set_position(self.col_handle, self.shape_pos(Some(new_pos)));
                match world.contact_pair(self.col_handle, map_handle, true) {
                    Some(_) => (),
                    None => {
                        self.collision_hor = Direction::Null;
                        old_dir.y = -1.0;
                        ()
                    },
                }
            }
            if self.collision_ver == Direction::Down && input::keyboard::is_key_pressed(ctx, event::KeyCode::Down) {
                self.direction = old_dir;
                self.direction.y = 1.0;
                let new_pos = self.pos_from_move();
                world.set_position(self.col_handle, self.shape_pos(Some(new_pos)));
                match world.contact_pair(self.col_handle, map_handle, true) {
                    Some(_) => (),
                    None => {
                        self.collision_hor = Direction::Null;
                        old_dir.y = 1.0;
                        ()
                    },
                }
            }
            self.direction = old_dir;
            world.set_position(self.col_handle, self.shape_pos(None));
            if self.collision_hor == Direction::Right && self.direction.x == -1.0 {
                self.collision_hor = Direction::Null;
            }
            if self.collision_hor == Direction::Left && self.direction.x == 1.0 {
                self.collision_hor = Direction::Null;
            }
            if self.collision_ver == Direction::Down && self.direction.y == -1.0 {
                self.collision_ver = Direction::Null;
            }
            if self.collision_ver == Direction::Up && self.direction.y == 1.0 {
                self.collision_ver = Direction::Null;
            }

            match world.contact_pair(self.col_handle, map_handle, true) {
                Some(c) => {
                    let col = c.3;
                    let dcontact = &col.deepest_contact().unwrap().contact;
                    let ddepth = dcontact.depth;
                    let dvector = dcontact.normal.into_inner();
                    if ddepth >= 0.0 && ddepth < 13.0 {
                        if dvector.x == -1.0 && self.direction.x == 1.0 {
                            self.collision_hor = Direction::Right;
                            self.direction.x = 0.0;
                            self.pos = mint::Point2 { x: self.pos.x-ddepth, y: self.pos.y };
                        }
                        if dvector.x == 1.0 && self.direction.x == -1.0 {
                            self.collision_hor = Direction::Left;
                            self.direction.x = 0.0;
                            self.pos = mint::Point2 { x: self.pos.x-ddepth, y: self.pos.y };
                        }
                        if dvector.y == -1.0 && self.direction.y == 1.0 {
                            self.collision_ver = Direction::Down;
                            self.direction.y = 0.0;
                            self.pos = mint::Point2 { x: self.pos.x, y: self.pos.y-ddepth };
                        }
                        if dvector.y == 1.0 && self.direction.y == -1.0 {
                            self.collision_ver = Direction::Up;
                            self.direction.y = 0.0;
                            self.pos = mint::Point2 { x: self.pos.x, y: self.pos.y+ddepth};
                        }
                        // self.pos = mint::Point2 { x: self.pos.x+dvector.x*(ddepth), y: self.pos.y+dvector.y*(ddepth) };
                    }
                    // self.pos = self.pos_from_move();

                    // world.set_position(self.col_handle, self.shape_pos(None));
                    ()
                },
                None => {
                    self.collision_ver = Direction::Null;
                    self.collision_hor = Direction::Null;
                    ()
                }

            }
            self.pos = self.pos_from_move();
            world.set_position(self.col_handle, self.shape_pos(None));

        }
    }

    fn draw(&self, ctx: &mut Context, show_mesh: bool) -> GameResult<()> {
        match self.animation_state {
            Direction::Right => self.run_right.draw(ctx, self.pos)?,
            Direction::Left => self.run_left.draw(ctx, self.pos)?,
            Direction::Up => self.run_up.draw(ctx, self.pos)?,
            Direction::Down => self.run_down.draw(ctx, self.pos)?,
            Direction::Null => graphics::draw(ctx, &self.idle, graphics::DrawParam::new().dest(self.pos))?,
        }
        if show_mesh {
            let shape_mesh = graphics::MeshBuilder::new().rectangle(graphics::DrawMode::stroke(3.0), graphics::Rect::new(self.shape_pos(None).translation.vector.x, self.shape_pos(None).translation.vector.y, 24.0, 16.0), [1.0, 0.0, 0.0, 1.0].into()).build(ctx)?;
            graphics::draw(ctx, &shape_mesh, graphics::DrawParam::new())?;
        }
        Ok(())
    }
}

struct Guard {
    pos: mint::Point2<f32>,
    direction: Vector2<f32>,
    run_left: Animation,
    run_right: Animation,
    run_down: Animation,
    run_up: Animation,
    animation_state: Direction,
    spd: f32,
    next_point: mint::Point2<f32>,
    patrol_points: Vec<mint::Point2<f32>>,
    current_patrol: usize,
}

impl Guard {
    pub fn new (ctx: &mut Context, coor_1: mint::Point2<f32>, coor_2: mint::Point2<f32> ) -> Self {
    // konstruktoru saljemo tacke koje oznacavaju koordinate sobe
        let mut patrol: Vec<mint::Point2<f32>> = Vec::new();
        let mut rng = thread_rng();
        for i in 0..4 {
            patrol.push(mint::Point2 { x: rng.gen_range(coor_1.x, coor_2.x),
                                       y: rng.gen_range(coor_1.y, coor_2.y) });
        }
        Guard {
            pos: mint:: Point2 {x: patrol[0].x , y: patrol[0].y }, // pocetna tacka strazara je sredina sobe
            direction: Vector2::new(0.0, 0.0),
            run_left: Animation::new(ctx, "/images/guard_runleft.png"),
            run_right: Animation::new(ctx, "/images/guard_runright.png"),
            run_down: Animation::new(ctx, "/images/guard_rundown.png"),
            run_up: Animation::new(ctx, "/images/guard_runup.png"), // imena slika nisu prava
            animation_state: Direction::Down,  // bilo kako
            spd: 3.3,
            next_point: mint:: Point2 {x: patrol[0].x , y: patrol[0].y },
            patrol_points: patrol,
            current_patrol: 0,
        }
    }
    fn direction_maker ( &self, point_1: mint::Point2<f32>, point_2: mint::Point2<f32>) -> Vector2<f32> {
        let dist_x = point_2.x - point_1.x;
        let dist_y = point_2.y - point_1.y;
        let norm = (dist_x * dist_x + dist_y * dist_y).sqrt();
        Vector2::new(dist_x, dist_y).normalize()
    }
    fn next_rand_coor (&mut self) {
        self.current_patrol = (self.current_patrol +1) % self.patrol_points.len();
        self.next_point = self.patrol_points[self.current_patrol];
    }
    fn pos_from_move(&self) -> mint::Point2<f32> {  // kopirana funkcija iz Player
        // ova f-ja se poziva pri svakom apdejtu
        // na osnovu trenutne pozicije i pravca kretanja
        // vraca "sledecu" poziciju igraca
        let mut norm_dir = Vector2::new(0.0, 0.0);
        if self.direction.x != 0.0 || self.direction.y != 0.0 {
            norm_dir = self.direction.normalize();
        }

        mint::Point2 { x: self.pos.x + norm_dir.x * self.spd, y: self.pos.y + norm_dir.y * self.spd }
    }
    fn update (&mut self, ctx: &mut Context) {
        if (self.pos.x.abs() - self.next_point.x.abs()).abs() > (self.spd + 0.2) &&
           (self.pos.y.abs() - self.next_point.y.abs()).abs() > (self.spd + 0.2) {
            self.pos = self.pos_from_move();
        }
        else {
            self.next_rand_coor();
            self.direction = self.direction_maker(self.pos, self.next_point);
        }

        if self.direction.x > 0.0  && ((self.direction.x).abs() - (self.direction.y).abs()) > 0.0 {
            self.animation_state = Direction::Right;
        }
        else if self.direction.x < 0.0  && (self.direction.x.abs() - self.direction.y.abs()) > 0.0  {
            self.animation_state = Direction::Left;
        }
        else if self.direction.y > 0.0  && (self.direction.x.abs() - self.direction.y.abs()) < 0.0 {
            self.animation_state = Direction::Down;
        }
        else if self.direction.y < 0.0  && (self.direction.x.abs() - self.direction.y.abs()) < 0.0 {
            self.animation_state = Direction::Up;
        }
        match self.animation_state {
            Direction::Right => self.run_right.next_frame(),
            Direction::Left => self.run_left.next_frame(),
            Direction::Up => self.run_up.next_frame(),
            Direction::Down => self.run_down.next_frame(),
            Direction::Null => (),
        }


    }
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let vec1 = Isometry2::new(Vector2::new(0.0,0.0), PI/6.0).transform_vector(&self.direction)*64.0;
        let vec2 = Isometry2::new(Vector2::new(0.0,0.0), -PI/6.0).transform_vector(&self.direction)*64.0;
        let origin_point = mint::Point2 {x: self.pos.x+16.0, y: self.pos.y +13.0};
        let vision = graphics::Mesh::from_triangles(ctx, &[origin_point, mint::Point2 {x: origin_point.x + vec1.x, y: origin_point.y + vec1.y},
            mint::Point2{x: origin_point.x + vec2.x, y: origin_point.y + vec2.y}], [1.0, 0.0, 0.0, 0.5].into())?;
        graphics::draw(ctx, &vision, graphics::DrawParam::new())?;
        match self.animation_state {
            Direction::Right => self.run_right.draw(ctx, self.pos)?,
            Direction::Left => self.run_left.draw(ctx, self.pos)?,
            Direction::Up => self.run_up.draw(ctx, self.pos)?,
            Direction::Down => self.run_down.draw(ctx, self.pos)?,
            Direction::Null => (),
        }
        Ok(())
    }
}

struct GameState {
    castle_map: map::Map,
    player: Player,
    guard: Guard,
    world: CollisionWorld<f32, ()>,
    last_update: Instant,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> Self {
        let mut world_mut = CollisionWorld::new(0.02);
        let shape = ShapeHandle::new(Cuboid::new(Vector2::new(12.0, 8.0)));
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[0 as usize]);
        groups.set_blacklist(&[0 as usize]);
        groups.set_whitelist(&[1 as usize]);
        let query = GeometricQueryType::Contacts(0.0, 0.0);

        GameState {
            castle_map: map::Map::load(ctx, "/levels/level1.txt", "/images/castle_spritesheet.png", mint::Point2 { x:0.0, y:0.0 }, mint::Point2 { x:32.0, y:32.0 }, &mut world_mut).unwrap(),
            player: Player::new(ctx, world_mut.add(Isometry2::new(Vector2::new(64.0, 74.0), 0.0), shape.clone(), groups, query, ()).handle()),
            guard: Guard::new(ctx, mint::Point2{x: 1.0*32.0, y: 8.0*32.0}, mint::Point2{x: 10.0*32.0, y: 11.0*32.0}),
            world: world_mut,
            last_update: Instant::now(),
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // da kontrolisemo broj apdejta u sekundi, ili FPS
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            self.player.update(ctx, &mut self.world, self.castle_map.map_handle);
            self.world.update();
            self.guard.update(ctx);
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.2, 0.2, 0.2, 1.0].into());
        self.castle_map.draw(ctx, 1, false)?;
        self.player.draw(ctx, false)?;
        self.guard.draw(ctx)?;
        self.castle_map.draw(ctx, 2, false)?;
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        _repeat: bool,
        ) {
        match keycode {
            event::KeyCode::Up if self.player.collision_ver != Direction::Up => {
                self.player.direction.y = -1.0;
                self.player.animation_state = Direction::Up;
            },
            event::KeyCode::Down if self.player.collision_ver != Direction::Down => {
                self.player.direction.y = 1.0;
                self.player.animation_state = Direction::Down;
            },
            event::KeyCode::Left if self.player.collision_hor != Direction::Left => {
                self.player.direction.x = -1.0;
                self.player.animation_state = Direction::Left;
            },
            event::KeyCode::Right if self.player.collision_hor != Direction::Right => {
                self.player.direction.x = 1.0;
                self.player.animation_state = Direction::Right;
            },
            _ => (),
        }
    }
    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        ) {
        match keycode {
            event::KeyCode::Up if self.player.direction.y == -1.0 => self.player.direction.y = 0.0,
            event::KeyCode::Down if self.player.direction.y == 1.0 => self.player.direction.y = 0.0,
            event::KeyCode::Left if self.player.direction.x == -1.0 => self.player.direction.x = 0.0,
            event::KeyCode::Right if self.player.direction.x == 1.0 => self.player.direction.x = 0.0,
            _ => (),
        }
    }
}

fn main() -> GameResult {
    let (ctx, events_loop) = &mut ContextBuilder::new("robin_hood", "lkh01")
        .window_setup(conf::WindowSetup::default().title("Robin Hood"))
        .window_mode(conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    let state = &mut GameState::new(ctx);
    event::run(ctx, events_loop, state)
}
