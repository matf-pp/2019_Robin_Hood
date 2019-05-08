use ggez::*;
use ggez::audio::SoundSource;

pub struct GameOver {
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
    play_again: bool,
    stone_sound: audio::Source,

}

impl GameOver {
    pub fn new(ctx: &mut Context, score: i32, won: bool) -> GameResult<Self> {
        let game_over_str = match won {
            true => format!("You Won"),
            false => format!("Game Over"),
        };
        let mut game_over_text = graphics::Text::new(game_over_str);
        let mut high_score_text = graphics::Text::new(format!("Gold collected: {}", score));
        let mut play_again_text = graphics::Text::new("Play again");
        let font_celtknot = graphics::Font::new(ctx, "/fonts/Celtknot.ttf").unwrap();
        game_over_text.set_font(font_celtknot, graphics::Scale::uniform(40.0));
        high_score_text.set_font(font_celtknot, graphics::Scale::uniform(40.0));
        play_again_text.set_font(font_celtknot, graphics::Scale::uniform(30.0));
        let mut stone_sound = audio::Source::new(ctx, "/sounds/stone_short.mp3").unwrap();
        stone_sound.play_detached()?;

        Ok(GameOver {
            text1_pos: mint::Point2 { x: 220.0, y: 542.0 },
            text2_pos: mint::Point2 { x: 140.0, y: 642.0 },
            text3_pos: mint::Point2 { x: 240.0, y: 742.0 },
            stone_pos: mint::Point2 { x: 40.0, y: 430.0 },
            text1: game_over_text,
            text2: high_score_text,
            text3: play_again_text,
            stone: graphics::Image::new(ctx, "/images/user_interface.png").unwrap(),
            final_text1_pos: mint::Point2 { x: 220.0, y: 152.0 },
            final_text2_pos: mint::Point2 { x: 140.0, y: 252.0 },
            final_text3_pos: mint::Point2 { x: 240.0, y: 352.0 },
            final_stone_pos: mint::Point2 { x: 40.0, y: 40.0 },
            orig_text1_pos: mint::Point2 { x: 220.0, y: 542.0 },
            orig_text2_pos: mint::Point2 { x: 140.0, y: 642.0 },
            orig_text3_pos: mint::Point2 { x: 240.0, y: 742.0 },
            orig_stone_pos: mint::Point2 { x: 40.0, y: 430.0 },
            play_again: false,
            stone_sound: stone_sound,
        })
    }

    pub fn update(&mut self, ctx: &mut Context) -> bool {
        if self.play_again == false {
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
            let mouse_position = input::mouse::position(ctx);
            if (mouse_position.x >= self.text3_pos.x) && (mouse_position.x < self.text3_pos.x + (self.text3.dimensions(ctx).0 as f32)) &&
                (mouse_position.y >= self.text3_pos.y) && (mouse_position.y < self.text3_pos.y + (self.text3.dimensions(ctx).1 as f32)) {
                    for fragment in self.text3.fragments_mut() {
                        // tip Text se sastoji od fragmenata
                        // tj polja koji predstavljaju osobine Text-a
                        // jedan od fragmenata je boja, koju ovde menjamo
                        // ako se mis nalazi preko teksta "Play again"
                        fragment.color = Some((208, 198, 29, 255).into());
                    }
                    if input::mouse::button_pressed(ctx, input::mouse::MouseButton::Left) {
                        self.play_again = true;
                        self.stone_sound.play_detached().unwrap();
                    }
                } else {
                    for fragment in self.text3.fragments_mut() {
                        fragment.color = Some([1.0, 1.0, 1.0, 1.0].into());
                    }
                    //boja ostaje ista ako se mis ne nalazi preko
                    // teksta "Play again"
                }
        }
        else { // ako je pritisnuto "Play again" spustamo kamen
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
                return false // prestajemo da updateujemo GameOver i pocinjemo igricu ispocetka
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
