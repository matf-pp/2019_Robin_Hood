extern crate ggez;
extern crate ncollide2d;
extern crate nalgebra as na;

mod map;

use ggez::*;
use na::{Vector2, Isometry2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use ncollide2d::world::{CollisionGroups, CollisionObjectHandle, CollisionWorld, GeometricQueryType};

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
    frames_hor: i32,
    frames_ver: i32,
    curr_frame: i32,
    fps: f32,
}

#[derive(Debug, Clone, PartialEq)]
enum CollisionDirection {
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
    collision_ver: CollisionDirection,
    collision_hor: CollisionDirection,
    walking: bool,
    img: graphics::Image, // TODO: zameniti graphics::Image sa mnogo efikasnijom varijantom graphics::spritebatch
    spd: f32,
    col_handle: CollisionObjectHandle,
}

impl Player {
    pub fn new(ctx: &mut Context, handle: CollisionObjectHandle) -> Self {
        Player {
            pos: mint::Point2 {x: 64.0, y: 64.0},
            direction: Vector2::new(0.0, 0.0),
            collision_ver: CollisionDirection::Null,
            collision_hor: CollisionDirection::Null,
            walking: false,
            img: graphics::Image::new(ctx, "/images/robin_rundown.png").unwrap(),
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
            Some(d) => Isometry2::new(Vector2::new(d.x+0.0, d.y+10.0), 0.0),
            None => Isometry2::new(Vector2::new(self.pos.x+0.0, self.pos.y+10.0), 0.0),
        }
    }

    fn update(&mut self, world: &mut CollisionWorld<f32, ()>, map_handle: CollisionObjectHandle) {
        /* self.walking je korisno za animaciju
         * npr. if self.walking {
         *          curr_animation = walk_animation;
         *      } else {
         *          curr_animation = idle_animation;
         *      }
         */
        if self.direction.x == 0.0 && self.direction.y == 0.0 {
            self.walking = false;
        } else {
            self.walking = true;
        }
        if self.walking {
            if self.collision_hor == CollisionDirection::Right && self.direction.x == -1.0 {
                self.collision_hor = CollisionDirection::Null;
            }
            if self.collision_hor == CollisionDirection::Left && self.direction.x == 1.0 {
                self.collision_hor = CollisionDirection::Null;
            }
            if self.collision_ver == CollisionDirection::Down && self.direction.y == -1.0 {
                self.collision_ver = CollisionDirection::Null;
            }
            if self.collision_ver == CollisionDirection::Up && self.direction.y == 1.0 {
                self.collision_ver = CollisionDirection::Null;
            }
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::draw(ctx, &self.img, graphics::DrawParam::new()
                       .src(graphics::Rect::new(0.0, 0.0, 1.0/7.0, 1.0))
                       //.scale(mint::Vector2 { x: 1.3, y: 1.3 })
                       .dest(self.pos))?;
        Ok(())
    }
}

struct GameState {
    castle_map: map::Map,
    player: Player,
    world: CollisionWorld<f32, ()>,
    last_update: Instant,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> Self {
        let mut world_mut = CollisionWorld::new(0.02);
        let shape = ShapeHandle::new(Cuboid::new(Vector2::new(10.0, 10.0)));
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[0 as usize]);
        groups.set_blacklist(&[0 as usize]);
        groups.set_whitelist(&[1 as usize]);
        let query = GeometricQueryType::Contacts(0.0, 0.0);

        // let player_handle = world_mut.add(Isometry2::new(Vector2::new(64.0, 64.0), 0.0), shape.clone(), groups, query, ()); // player

        GameState {
            castle_map: map::Map::load(ctx, "/levels/level1.txt", "/images/castle_spritesheet.png", mint::Point2 { x:0.0, y:0.0 }, mint::Point2 { x:32.0, y:32.0 }, &mut world_mut).unwrap(),
            player: Player::new(ctx, world_mut.add(Isometry2::new(Vector2::new(64.0, 64.0), 0.0), shape.clone(), groups, query, ()).handle()),
            world: world_mut,
            last_update: Instant::now(),
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // da kontrolisemo broj apdejta u sekundi, ili FPS
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            self.player.update(&mut self.world, self.castle_map.map_handle);
            self.world.update();
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.2, 0.2, 0.2, 1.0].into());
        self.castle_map.draw(ctx)?;
        self.player.draw(ctx)?;
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
            event::KeyCode::Up if self.player.collision_ver != CollisionDirection::Up => self.player.direction.y = -1.0,
            event::KeyCode::Down if self.player.collision_ver != CollisionDirection::Down => self.player.direction.y = 1.0,
            event::KeyCode::Left if self.player.collision_hor != CollisionDirection::Left => self.player.direction.x = -1.0,
            event::KeyCode::Right if self.player.collision_hor != CollisionDirection::Right => self.player.direction.x = 1.0,
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

