use ggez::*;
use ggez::audio::SoundSource;

pub struct GameOver {
    text1_pos: mint::Point2<f32>,
    text2_pos: mint::Point2<f32>,
    stone_pos: mint::Point2<f32>,
    text1: graphics::Text,
    text2: graphics::Text,
    stone: graphics::Image,
    final_text1_pos: mint::Point2<f32>,
    final_text2_pos: mint::Point2<f32>,
    final_stone_pos: mint::Point2<f32>,

}

impl GameOver {
    pub fn new(ctx: &mut Context, score: i32) -> GameResult<Self> {
        let mut game_over_text = graphics::Text::new(format!("Game Over"));
        let mut high_score_text = graphics::Text::new(format!("Gold collected: {}", score));
        let font_celtknot = graphics::Font::new(ctx, "/fonts/Celtknot.ttf").unwrap();
        game_over_text.set_font(font_celtknot, graphics::Scale::uniform(40.0));
        high_score_text.set_font(font_celtknot, graphics::Scale::uniform(40.0));
        let mut stone_sound = audio::Source::new(ctx, "/sounds/stone.wav").unwrap();
        stone_sound.play_detached();

        Ok(GameOver {
            text1_pos: mint::Point2 { x: 200.0, y: 542.0 },
            text2_pos: mint::Point2 { x: 140.0, y: 592.0 },
            stone_pos: mint::Point2 { x: 40.0, y: 430.0 },
            text1: game_over_text,
            text2: high_score_text,
            stone: graphics::Image::new(ctx, "/images/user_interface.png").unwrap(),
            final_text1_pos: mint::Point2 { x: 200.0, y: 152.0 },
            final_text2_pos: mint::Point2 { x: 140.0, y: 202.0 },
            final_stone_pos: mint::Point2 { x: 40.0, y: 40.0 },
        })
    }

    pub fn update(&mut self) {
        if self.text1_pos != self.final_text1_pos {
            self.text1_pos.y -= 1.0;
        }
        if self.text2_pos != self.final_text2_pos {
            self.text2_pos.y -= 1.0;
        }
        if self.stone_pos != self.final_stone_pos {
            self.stone_pos.y -= 1.0;
        }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::draw(ctx, &self.stone, graphics::DrawParam::new().dest(self.stone_pos))?;
        graphics::draw(ctx, &self.text1, graphics::DrawParam::new().dest(self.text1_pos))?;
        graphics::draw(ctx, &self.text2, graphics::DrawParam::new().dest(self.text2_pos))?;
        Ok(())
    }
}
