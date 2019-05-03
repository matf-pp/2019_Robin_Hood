use ggez::*;
use ncollide2d::shape::{Compound};
use ncollide2d::world::{CollisionObjectHandle, CollisionWorld};
use ncollide2d::query::{Ray, RayCast};
use na::{Vector2, Isometry2, Rotation2, Point2};


use crate::anim::{Animation, Direction};
use crate::score::Score;

#[derive(Debug)]
pub struct Player {
    pos: mint::Point2<f32>,
    pub direction: Vector2<f32>,
    pub collision_ver: Direction,
    pub collision_hor: Direction,
    walking: bool,
    run_left: Animation,
    run_right: Animation,
    run_down: Animation,
    run_up: Animation,
    idle: graphics::Image,
    pub animation_state: Direction,
    spd: f32,
    pub col_handle: CollisionObjectHandle,
    visibility: Vec<mint::Point2<f32>>,
    score: i32,
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
            visibility: Vec::new(),
            score: 0,

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

    pub fn update(&mut self, ctx: &mut Context, world: &mut CollisionWorld<f32, ()>, map_handle: CollisionObjectHandle, corners: &mut Vec<mint::Point2<f32>>) {
        /* self.walking je korisno za animaciju
         * npr. if self.walking {
         *          curr_animation = walk_animation;
         *      } else {
         *          curr_animation = idle_animation;
         *      }
         */
        self.visibility.clear();
        // corners.sort_by(|a, b| Rotation2::rotation_between(&Vector2::x(), &Vector2::new(a.x, a.y)).angle().partial_cmp(&Rotation2::rotation_between(&Vector2::x(), &Vector2::new(b.x, b.y)).angle()).unwrap());

        let origin_point = mint::Point2 { x: self.pos.x+5.0, y: self.pos.y+4.0 };
        for corner in corners.iter() {
            let ray = Ray::new(Point2::new(origin_point.x, origin_point.y), Vector2::new(corner.x-origin_point.x, corner.y-origin_point.y).normalize());
            let map_shape: &Compound<f32> = world.collision_object(map_handle).unwrap().shape().as_shape().unwrap();
            let intersect = (*map_shape).toi_with_ray(&Isometry2::identity(), &ray, true).unwrap();
            let ipoint = ray.point_at(intersect);
            self.visibility.push(mint::Point2 { x: ipoint.x, y: ipoint.y });
        }
        self.visibility.sort_by(|a, b| Rotation2::rotation_between(&Vector2::x(), &Vector2::new(origin_point.x - a.x, origin_point.y - a.y)).angle().partial_cmp(&Rotation2::rotation_between(&Vector2::x(), &Vector2::new(origin_point.x - b.x, origin_point.y - b.y)).angle()).unwrap());


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

    pub fn draw_visibility(&self, ctx: &mut Context) -> GameResult<()> {
        if self.visibility.len() > 0 && self.visibility[0] != self.visibility[1] {
            let vis_clone = self.visibility.clone();

            let vis_mesh_test: graphics::Mesh = graphics::Mesh::new_polygon(ctx, graphics::DrawMode::fill(), &vis_clone, [0.0, 1.0, 0.0, 0.5].into())?;
            // let mut whole_screen: graphics::Mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), [0.0, 0.0, SCREEN_SIZE.0, SCREEN_SIZE.1].into(), [0.0, 0.0, 0.0, 1.0].into())?;
            // whole_screen.set_blend_mode(Some(graphics::BlendMode::Subtract));
            // vis_mesh_test.set_blend_mode(Some(graphics::BlendMode::Invert));
            // let vis_mesh = vis_mesh_builder.build(ctx)?;
            // graphics::draw(ctx, &whole_screen, graphics::DrawParam::new())?;
            graphics::draw(ctx, &vis_mesh_test, graphics::DrawParam::new())?;
        }
        Ok(())
    }

    pub fn draw(&self, ctx: &mut Context, show_mesh: bool) -> GameResult<()> {
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

impl Score for Player {
    fn increase (&mut self, coin: i32) {
        self.score = self.score + coin;
    }
    fn draw_score (&self, ctx: &mut Context ) -> GameResult<()> {
        let high_score = format!("Level 1     Gold collected: {}", self.score);
        let mut tekst = graphics::Text::new (high_score);
        let font_stonecross = graphics::Font::new(ctx, "/fonts/MeathFLF.ttf")?;
        let interface_stone = graphics::Image::new (ctx, "/images/user_interface.png")?;
        tekst.set_font(font_stonecross, graphics::Scale::uniform(20.0));
        graphics::draw (ctx, &interface_stone, graphics::DrawParam::new().dest(mint::Point2{x: 40.0 , y: 432.0}))?;
        graphics::draw (ctx, &tekst, graphics::DrawParam::new().dest(mint::Point2{x: 200.0 , y: 455.0}))?;
        Ok(())
    }
}
// zlato razlicite velicine nosi razlicit Score
// mozemo uz score da ispisemo nivo na ekranu
