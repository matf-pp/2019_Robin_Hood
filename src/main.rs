extern crate ggez;
extern crate ncollide2d;
extern crate rand;
extern crate nalgebra as na;

mod map;
mod guard;
mod anim;
mod player;
mod score;

use ggez::*;
use ggez::graphics::Drawable;
use ggez::audio::SoundSource;
use na::{Vector2, Isometry2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use ncollide2d::world::{CollisionGroups, CollisionWorld, GeometricQueryType};

use crate::score::Score;
use crate::anim::Direction;
use crate::guard::Guard;
use crate::player::Player;

use std::time::{Duration, Instant};

const SCREEN_SIZE: (f32, f32) = (
    640.0, 480.0
    );

const UPDATES_PER_SECOND: f32 = 30.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

struct GameState {
    castle_map: map::Map,
    player: Player,
    world: CollisionWorld<f32, ()>,
    last_update: Instant,
    song: audio::Source,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut world_mut = CollisionWorld::new(0.02);
        let shape = ShapeHandle::new(Cuboid::new(Vector2::new(12.0, 8.0)));
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[0 as usize]);
        groups.set_blacklist(&[0 as usize]);
        groups.set_whitelist(&[1 as usize]);
        let query = GeometricQueryType::Contacts(0.0, 0.0);
        let celtic_song = audio::Source::new(ctx, "/music/a_celtic_lore.mp3")?;
        

        Ok(GameState {
            castle_map: map::Map::load(ctx, "/levels/level1.txt", "/images/castle_spritesheet.png", mint::Point2 { x:0.0, y:0.0 }, mint::Point2 { x:32.0, y:32.0 }, &mut world_mut).unwrap(),
            player: Player::new(ctx, world_mut.add(Isometry2::new(Vector2::new(64.0, 74.0), 0.0), shape.clone(), groups, query, ()).handle()),
            world: world_mut,
            last_update: Instant::now(),
            song: celtic_song,
        })
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // da kontrolisemo broj apdejta u sekundi, ili FPS
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            if !self.song.playing() {
                self.song.play()?;
            }
            self.player.update(ctx, &mut self.world, self.castle_map.map_handle, &mut self.castle_map.get_corners());
            self.world.update();
            self.castle_map.update_guards();
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.2, 0.2, 0.2, 1.0].into());
        self.castle_map.draw(ctx, 1, false)?;
        self.player.draw(ctx, false)?;
        self.castle_map.draw_guards(ctx)?;
        self.castle_map.draw(ctx, 2, false)?;
        self.castle_map.draw_guard_vision(ctx)?;
        self.player.draw_score(ctx)?;
        // self.player.draw_visibility(ctx)?;
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

    let state = &mut GameState::new(ctx)?;
    event::run(ctx, events_loop, state)
}
