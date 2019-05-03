use std::f32::consts::PI;
use ggez::*;
use na::{Vector2, Isometry2, Rotation2};
use rand::{thread_rng, Rng};

use crate::anim::{Animation, Direction};

#[derive(Debug)]
pub struct Guard {
    pos: mint::Point2<f32>,
    direction: Vector2<f32>,
    final_direction: Vector2<f32>,
    run_left: Animation,
    run_right: Animation,
    run_down: Animation,
    run_up: Animation,
    animation_state: Direction,
    spd: f32,
    turn_spd: f32,
    rotation: Rotation2<f32>,
    next_point: mint::Point2<f32>,
    patrol_points: Vec<mint::Point2<f32>>,
    current_patrol: usize,
}

impl Guard {
    pub fn new(ctx: &mut Context, coor_1: mint::Point2<f32>, coor_2: mint::Point2<f32>, patrol_point_count: i32) -> Self {
        // konstruktoru saljemo tacke koje oznacavaju koordinate sobe
        let mut patrol: Vec<mint::Point2<f32>> = Vec::new();
        let mut rng = thread_rng();
        for _i in 0..patrol_point_count {
            patrol.push(mint::Point2 { x: rng.gen_range(coor_1.x, coor_2.x),
            y: rng.gen_range(coor_1.y, coor_2.y) });
        }
        Guard {
            pos: mint:: Point2 {x: patrol[0].x , y: patrol[0].y }, 
            direction: Vector2::new(0.0, 1.0),
            final_direction: Vector2::new(0.0, 0.0),
            run_left: Animation::new(ctx, "/images/guard_runleft.png"),
            run_right: Animation::new(ctx, "/images/guard_runright.png"),
            run_down: Animation::new(ctx, "/images/guard_rundown.png"),
            run_up: Animation::new(ctx, "/images/guard_runup.png"), 
            animation_state: Direction::Down,
            spd: 3.3,
            turn_spd: 8.0,
            rotation: Rotation2::new(0.0),
            next_point: mint:: Point2 {x: patrol[0].x , y: patrol[0].y },
            patrol_points: patrol,
            current_patrol: 0,
        }
    }
    fn direction_maker(&self, point_1: mint::Point2<f32>, point_2: mint::Point2<f32>) -> Vector2<f32> {
        let dist_x = point_2.x - point_1.x;
        let dist_y = point_2.y - point_1.y;
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
    pub fn update(&mut self) {
        if (self.pos.x.abs() - self.next_point.x.abs()).abs() > (self.spd + 0.2) &&
            (self.pos.y.abs() - self.next_point.y.abs()).abs() > (self.spd + 0.2) {
                if self.final_direction.relative_eq(&self.direction, 0.00006, 0.0006) == false {
                    let iso = Isometry2::new(Vector2::new(0.0, 0.0), self.rotation.angle());
                    self.direction = iso.transform_vector(&self.direction);
                } else {
                    self.pos = self.pos_from_move();
                }
            } else {
                self.next_rand_coor();
                self.final_direction = self.direction_maker(self.pos, self.next_point);
                self.rotation = Rotation2::scaled_rotation_between(&self.direction, &self.final_direction, 1.0/self.turn_spd);
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
    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
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
