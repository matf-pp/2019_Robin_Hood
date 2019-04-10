extern crate ggez;

mod map;

use ggez::*;

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

#[derive(Debug)]
struct Player {
    pos: mint::Point2<f32>,
    ver: f32,
    hor: f32,
    walking: bool,
    img: graphics::Image, // TODO: zameniti graphics::Image sa mnogo efikasnijom varijantom graphics::spritebatch
    spd: f32,
}

impl Player {
    pub fn new(ctx: &mut Context) -> Self {
        Player {
            pos: mint::Point2 {x: 0.0, y: 0.0},
            ver: 0.0,
            hor: 0.0,
            walking: false,
            img: graphics::Image::new(ctx, "/images/robin_rundown.png").unwrap(),
            spd: 8.0,
        }
    }

    fn pos_from_move(&self) -> mint::Point2<f32> {
        // ova f-ja se poziva pri svakom apdejtu
        // na osnovu trenutne pozicije i pravca kretanja
        // vraca "sledecu" poziciju igraca
        let mov = nalgebra::Vector2::new(self.hor, self.ver);
        let mov_norm = mov.normalize();
        mint::Point2 { x: self.pos.x + mov_norm.x * self.spd, y: self.pos.y + mov_norm.y * self.spd }
    }

    fn update(&mut self) {
        /* self.walking je korisno za animaciju
         * npr. if self.walking {
         *          curr_animation = walk_animation;
         *      } else {
         *          curr_animation = idle_animation;
         *      }
         */
        if self.ver == 0.0 && self.hor == 0.0 {
            self.walking = false;
        } else {
            self.walking = true;
        }
        if self.walking {
            self.pos = self.pos_from_move();
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::draw(ctx, &self.img, graphics::DrawParam::new()
                       .src(graphics::Rect::new(0.0, 0.0, 1.0/7.0, 1.0))
                       .scale(mint::Vector2 { x: 1.3, y: 1.3 })
                       .dest(self.pos))?;
        Ok(())
    }
}

struct GameState {
    castle_map: map::Map,
    player: Player,
    last_update: Instant,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> Self {
        GameState {
            castle_map: map::Map::load(ctx, "/levels/level1.txt").unwrap(),
            player: Player::new(ctx),
            last_update: Instant::now(),
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // da kontrolisemo broj apdejta u sekundi, ili FPS
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            self.player.update();
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
            event::KeyCode::Up => self.player.ver = -1.0,
            event::KeyCode::Down => self.player.ver = 1.0,
            event::KeyCode::Left => self.player.hor = -1.0,
            event::KeyCode::Right => self.player.hor = 1.0,
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
            event::KeyCode::Up if self.player.ver == -1.0 => self.player.ver = 0.0,
            event::KeyCode::Down if self.player.ver == 1.0 => self.player.ver = 0.0,
            event::KeyCode::Left if self.player.hor == -1.0 => self.player.hor = 0.0,
            event::KeyCode::Right if self.player.hor == 1.0 => self.player.hor = 0.0,
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

