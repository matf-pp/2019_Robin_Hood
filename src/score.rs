use ggez::*;

pub trait Score {
    fn increase (&mut self) -> ();    // da povecavamo score
    fn draw_score (&self, ctx: &mut Context)-> GameResult<()>;  // da ispise score na ekran

}

// u struct Player sam dodala polje score
// Mozda i nije potreban zaseban interfejs posto ima malo metoda
