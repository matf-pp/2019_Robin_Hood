use std::path::Path;
use ggez::*;

#[derive(Debug)]
pub struct Animation {
    spritesheet: graphics::Image,
    width: f32,
    height: f32,
    frames_hor: f32,
    frames_ver: f32,
    curr_frame: f32,
    src_rect: graphics::Rect,
}

impl Animation {
    pub fn new<P>(ctx: &mut Context, filename: P) -> Self
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

    pub fn reset(&mut self) {
        self.curr_frame = 0.0;
        self.src_rect.x = 0.0;
    }

    pub fn next_frame(&mut self) {
        self.curr_frame = ((self.curr_frame as i32 + 1) % 7) as f32;
        self.src_rect.x = self.curr_frame/self.frames_hor;
        // y ne moramo da pomeramo jer je slika horizontalna
    }

    pub fn draw(&self,ctx: &mut Context, pos: mint::Point2<f32>) -> GameResult<()> {
        graphics::draw(ctx, &self.spritesheet, graphics::DrawParam::new().src(self.src_rect).dest(pos))?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Null,
}
