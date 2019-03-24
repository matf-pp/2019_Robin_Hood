extern crate ggez;

use ggez::*;

#[derive(Debug)]
pub enum TileType {
    Floor,
    Wall,
}

#[derive(Debug)]
pub struct Tile {
    tile_type: TileType,
    tile_image: graphics::Image,
    tile_pos: mint::Point2<f32>,
    tile_size: mint::Point2<f32>,
}

impl Tile {
    pub fn new(ctx: &mut Context, ttype: TileType, tpos: mint::Point2<f32>, tsize: mint::Point2<f32>) -> Self {
        let timage = match ttype {
            TileType::Floor => graphics::Image::new(ctx, "/images/castle_floor.png"),
            TileType::Wall => graphics::Image::new(ctx, "/images/castle_wall.png"),
        };
        Tile {
            tile_type: ttype,
            tile_image: timage.unwrap(),
            tile_pos: tpos,
            tile_size: tsize,
        }
    }
}
