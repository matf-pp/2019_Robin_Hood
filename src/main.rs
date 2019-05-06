extern crate ggez;
extern crate ncollide2d;
extern crate rand;
extern crate nalgebra as na;

mod map;
mod guard;
mod anim;
mod player;
mod score;
mod game_over;
mod main_menu;

use ggez::*;
use ggez::audio::SoundSource;
use na::{Vector2, Isometry2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use ncollide2d::world::{CollisionGroups, CollisionWorld, GeometricQueryType};

use crate::score::Score;
use crate::anim::Direction;
use crate::player::Player;
use crate::game_over::GameOver;
use crate::main_menu::MainMenu;

use std::time::{Duration, Instant};

const SCREEN_SIZE: (f32, f32) = (
    640.0, 480.0
    );

const UPDATES_PER_SECOND: f32 = 30.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64; // vreme koje treba da prodje izmedju dva updatea

struct GameState { // glavno stanje cele igre
    castle_map: map::Map,
    player: Player,
    world: CollisionWorld<f32, ()>,
    last_update: Instant, // vreme kad se desio poslednji update
    song: audio::Source,
    menu: MainMenu,
    in_menu: bool,
    end: Option<GameOver>,
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
            castle_map: map::Map::load(ctx, "/levels/level1.txt", "/images/castle_spritesheet.png", mint::Point2 { x:100.0, y:100.0 }, mint::Point2 { x:32.0, y:32.0 }, &mut world_mut).unwrap(),
            player: Player::new(ctx, world_mut.add(Isometry2::new(Vector2::new(64.0, 74.0), 0.0), shape.clone(), groups, query, ()).handle()),
            world: world_mut,
            last_update: Instant::now(),
            song: celtic_song,
            menu: MainMenu::new(ctx)?,
            in_menu: true,
            end: None,
        })
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // da kontrolisemo broj apdejta u sekundi, ili FPS
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            if !self.song.playing() {
                self.song.set_repeat(true);
                self.song.play()?;
            }
            if !self.in_menu {
                match &mut self.end {
                    None => {
                        if self.player.caught {
                            self.end = Some(GameOver::new(ctx, self.player.score).unwrap());
                        }
                        let map_move = self.player.update(ctx, &mut self.world, self.castle_map.map_handle, &mut self.castle_map.get_corners());
                        self.castle_map.update(&mut self.world, map_move);
                        self.world.update();
                        self.player.caught = self.castle_map.update_guards(&mut self.world, self.player.col_handle);
                        self.player.increase(self.castle_map.update_gold(&mut self.world, self.player.col_handle))?;
                    },
                    Some(g) => {
                        g.update();
                    }
                }
            } else {
                self.in_menu = self.menu.update(ctx);
            }
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult { // crta sve na mapu, bitan je redosled navodjenja pojedinacnih draw funkcija
        graphics::clear(ctx, (18, 15, 17, 255).into()); // brise prethodno stanje ekrana (posto se ono non stop updateuje)
        if !self.in_menu {
            match &self.end {
                None => {
                    self.castle_map.draw(ctx, 1, false)?; // crta prvi sloj mape (podovi, zidovi iza igraca)
                    self.castle_map.draw_gold(ctx)?; // prodje kroz ceo vektor i nacrta svaki element
                    self.player.draw(ctx, false)?;
                    self.castle_map.draw_guards(ctx)?; //
                    self.castle_map.draw(ctx, 2, false)?; // crta drugi sloj mape (donji zidovi)
                    self.castle_map.draw_guard_vision(ctx)?; // vidno polje strazara
                    self.player.draw_score(ctx)?;
                    // self.player.draw_visibility(ctx)?;
                },
                Some(g) => {
                    g.draw(ctx)?;
                }
            }
        } else {
            self.menu.draw(ctx)?;
        }
        graphics::present(ctx)?; // konacno sve nacrta na ekran
        timer::yield_now(); // ovo pisemo da bi crtanje sacekalo sledeci update
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

fn main() -> GameResult { //
    let (ctx, events_loop) = &mut ContextBuilder::new("robin_hood", "lkh01")
        .window_setup(conf::WindowSetup::default().title("Robin Hood"))
        .window_mode(conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    let state = &mut GameState::new(ctx)?;
    event::run(ctx, events_loop, state)
}
