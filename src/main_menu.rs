use ggez::*;
use ggez::audio::SoundSource;

pub struct MainMenu {
    text1_pos: mint::Point2<f32>,
    text2_pos: mint::Point2<f32>,
    text3_pos: mint::Point2<f32>,
    stone_pos: mint::Point2<f32>,
    text1: graphics::Text,
    text2: graphics::Text,
    text3: graphics::Text,
    stone: graphics::Image,
    final_text1_pos: mint::Point2<f32>,
    final_text2_pos: mint::Point2<f32>,
    final_text3_pos: mint::Point2<f32>,
    final_stone_pos: mint::Point2<f32>,
    orig_text1_pos: mint::Point2<f32>,
    orig_text2_pos: mint::Point2<f32>,
    orig_text3_pos: mint::Point2<f32>,
    orig_stone_pos: mint::Point2<f32>,
    stone_sound: audio::Source,
    play: bool,

}

impl MainMenu {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut title_text = graphics::Text::new("Robin Hood");
        let mut play_text = graphics::Text::new("Play");
        let mut quit_text = graphics::Text::new("Quit");
        let font_celtknot = graphics::Font::new(ctx, "/fonts/Celtknot.ttf").unwrap();
        title_text.set_font(font_celtknot, graphics::Scale::uniform(60.0));
        play_text.set_font(font_celtknot, graphics::Scale::uniform(40.0));
        quit_text.set_font(font_celtknot, graphics::Scale::uniform(40.0));
        let mut stone_sound = audio::Source::new(ctx, "/sounds/stone_short.mp3").unwrap();

        stone_sound.play_detached()?;

        Ok(MainMenu {
            text1_pos: mint::Point2 { x: 150.0, y: 542.0 },
            text2_pos: mint::Point2 { x: 260.0, y: 642.0 },
            text3_pos: mint::Point2 { x: 260.0, y: 702.0 },
            stone_pos: mint::Point2 { x: 40.0, y: 430.0 },
            text1: title_text,
            text2: play_text,
            text3: quit_text,
            stone: graphics::Image::new(ctx, "/images/user_interface.png").unwrap(),
            final_text1_pos: mint::Point2 { x: 150.0, y: 152.0 },
            final_text2_pos: mint::Point2 { x: 260.0, y: 252.0 },
            final_text3_pos: mint::Point2 { x: 260.0, y: 312.0 },
            final_stone_pos: mint::Point2 { x: 40.0, y: 40.0 },
            orig_text1_pos: mint::Point2 { x: 150.0, y: 542.0 },
            orig_text2_pos: mint::Point2 { x: 260.0, y: 642.0 },
            orig_text3_pos: mint::Point2 { x: 260.0, y: 702.0 },
            orig_stone_pos: mint::Point2 { x: 40.0, y: 430.0 },
            stone_sound: stone_sound,
            play: false,
        })
    }

    pub fn update(&mut self, ctx: &mut Context) -> bool {
        if self.play == false {
            if self.text1_pos != self.final_text1_pos {
                self.text1_pos.y -= 5.0;
            }
            if self.text2_pos != self.final_text2_pos {
                self.text2_pos.y -= 5.0;
            }
            if self.text3_pos != self.final_text3_pos {
                self.text3_pos.y -= 5.0;
            }
            if self.stone_pos != self.final_stone_pos {
                self.stone_pos.y -= 5.0;
            }
            let mouse_pos = input::mouse::position(ctx);
            if mouse_pos.x >= self.text2_pos.x && mouse_pos.x < self.text2_pos.x + (self.text2.dimensions(ctx).0 as f32) &&
                mouse_pos.y >= self.text2_pos.y && mouse_pos.y < self.text2_pos.y + (self.text2.dimensions(ctx).1 as f32) {
                // mis je preko play teksta
                for fragment in self.text2.fragments_mut() {
                    fragment.color = Some((208, 198, 29, 255).into());
                }
                if input::mouse::button_pressed(ctx, input::mouse::MouseButton::Left) {
                    self.play = true;
                    self.stone_sound.play_detached().unwrap();
                }
            } else {
                for fragment in self.text2.fragments_mut() {
                    fragment.color = Some([1.0, 1.0, 1.0, 1.0].into());
                }
            }
            if mouse_pos.x >= self.text3_pos.x && mouse_pos.x < self.text3_pos.x + (self.text3.dimensions(ctx).0 as f32) &&
                mouse_pos.y >= self.text3_pos.y && mouse_pos.y < self.text3_pos.y + (self.text3.dimensions(ctx).1 as f32) {
                // mis je preko play teksta
                for fragment in self.text3.fragments_mut() {
                    fragment.color = Some((208, 198, 29, 255).into());
                }
                if input::mouse::button_pressed(ctx, input::mouse::MouseButton::Left) {
                    quit(ctx);
                }
            } else {
                for fragment in self.text3.fragments_mut() {
                    fragment.color = Some([1.0, 1.0, 1.0, 1.0].into());
                }
            }
        } else {
            if self.text1_pos != self.orig_text1_pos {
                self.text1_pos.y += 5.0;
            }
            if self.text2_pos != self.orig_text2_pos {
                self.text2_pos.y += 5.0;
            }
            if self.text3_pos != self.orig_text3_pos {
                self.text3_pos.y += 5.0;
            }
            if self.stone_pos != self.orig_stone_pos {
                self.stone_pos.y += 5.0;
            }
            if self.stone_pos == self.orig_stone_pos {
                return false 
            }
        }
        true 
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::draw(ctx, &self.stone, graphics::DrawParam::new().dest(self.stone_pos))?;
        graphics::draw(ctx, &self.text1, graphics::DrawParam::new().dest(self.text1_pos))?;
        graphics::draw(ctx, &self.text2, graphics::DrawParam::new().dest(self.text2_pos))?;
        graphics::draw(ctx, &self.text3, graphics::DrawParam::new().dest(self.text3_pos))?;
        Ok(())
    }
}
