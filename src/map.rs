extern crate ggez;

use std::io::Read;
use std::path::Path;
use ggez::*;

#[derive(Debug, Clone)]
pub enum TileType {
    Floor(graphics::DrawParam),
    Wall(graphics::DrawParam),
}

#[derive(Debug, Clone)] // Clone nam treba da bi mogli da kopiramo vektore
pub struct Tile {
    tile_type: TileType,
    tile_src: graphics::DrawParam,
    tile_pos: mint::Point2<f32>,
    tile_size: mint::Point2<f32>,
}

impl Tile {
    pub fn new(ctx: &mut Context, ttype: TileType, tpos: mint::Point2<f32>, tsize: mint::Point2<f32>) -> Self {
        let tsrc = match ttype {
            TileType::Floor(d) => d,
            TileType::Wall(d)  => d,
        }; // od tipa polja nece zavisiti samo slika, 
           // vec i stvari kao sto su kolizija, osvetljenje itd.  
        Tile {
            tile_type: ttype,
            tile_src: tsrc,
            tile_pos: tpos,
            tile_size: tsize,
        }
    }

    pub fn drawparam(&self, map_start: mint::Point2<f32>) -> graphics::DrawParam {
        /* Mozda je bolje crtanje prepustiti samoj mapi, da bi mogli da pomeramo celu mapu
         * ili eventualno napravimo i minimapu. U tom slucaju bi tile_pos bila pozicija u matrici
         * mape, a ne na samom ekranu, a ova draw funkcija bi mogla da uzima tacku levog gornjeg
         * ugla mape kao argument.
         */
        self.tile_src.dest(mint::Point2 { x: map_start.x+self.tile_pos.x*self.tile_size.x,
                                          y: map_start.y+self.tile_pos.y*self.tile_size.y,
        })
    }
}

pub struct Map {
    map_size: mint::Point2<f32>,
    map_start: mint::Point2<f32>,
    map_tile_size: mint::Point2<f32>,
    map_matrix: Vec<Vec<Tile>>,
    map_spritebatch: graphics::spritebatch::SpriteBatch,
}

impl Map {
    pub fn load<P>(ctx: &mut Context, level_filename: P, image_filename: P, startpos: mint::Point2<f32>, tile_size: mint::Point2<f32>) -> GameResult<Self> 
    where 
        P: AsRef<Path>,
    {
        let spritesheet = graphics::Image::new(ctx, image_filename)?;
        let mut map_file = filesystem::open(ctx, level_filename)?;
        let mut map_string: String = "".to_string();
        map_file.read_to_string(&mut map_string)?;
        let mut map_lines = map_string.lines();
        
        // U prvoj liniji treba da bude 2 broja - 
        // map_width map_heigth
        let first_line: Vec<&str> = map_lines.next().unwrap().split(' ').collect();
        let map_width: f32 = first_line[0].parse().unwrap();
        let map_heigth: f32 = first_line[1].parse().unwrap();

        let mut matrix: Vec<Vec<Tile>> = Vec::with_capacity(map_heigth as usize);

        let mut curr_row_vec: Vec<Tile> = Vec::with_capacity(map_width as usize);
        let mut curr_x = 0.0;
        let mut curr_y = 0.0;

        while let Some(line) = map_lines.next() {
            for c in line.chars() {
                match c {
                    // treba pratiti x i y poziciju svakog polja, i na osnovu karaktera sa te
                    // pozicije dodati polje odgovarajuceg tipa u matricu mape
                    '#' => curr_row_vec.push(Tile::new(ctx,
                                                       TileType::Wall(graphics::DrawParam::new().src(graphics::Rect::new(tile_size.x/(spritesheet.width() as f32), 0.0, 0.5, 1.0))), 
                                                       mint::Point2 { x:curr_x, y:curr_y }, 
                                                       tile_size)),
                    ' ' => curr_row_vec.push(Tile::new(ctx,
                                                       TileType::Floor(graphics::DrawParam::new().src(graphics::Rect::new(0.0, 0.0, 0.5, 1.0))), 
                                                       mint::Point2 { x:curr_x, y:curr_y}, 
                                                       tile_size)),
                    _   => (),
                }
                curr_x += 1.0;
            }
            matrix.push(curr_row_vec.clone());
            curr_row_vec.clear();
            curr_x = 0.0;
            curr_y += 1.0;
        }

        Ok(Map {
            map_size: mint::Point2 { x: map_width, y: map_heigth }, // ovo je broj polja na mapi
            map_start: startpos,
            map_tile_size: tile_size,
            map_matrix: matrix,
            map_spritebatch: graphics::spritebatch::SpriteBatch::new(spritesheet),
          })
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for row in self.map_matrix.iter() {
            for tile in row.iter() {
                self.map_spritebatch.add(tile.drawparam(self.map_start));
            }
        }
        graphics::draw(ctx, &self.map_spritebatch, graphics::DrawParam::new())?;
        self.map_spritebatch.clear();
        Ok(())
    }
}
