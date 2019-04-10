extern crate ggez;

use std::io::Read;
use std::path::Path;
use ggez::*;

#[derive(Debug, Clone)]
pub enum TileType {
    Floor,
    Wall,
}

#[derive(Debug, Clone)] // Clone nam treba da bi mogli da kopiramo vektore
pub struct Tile {
    tile_type: TileType,
    tile_image: graphics::Image, // TODO: zameniti sa graphics::spritebatch
    tile_pos: mint::Point2<f32>,
    tile_size: mint::Point2<f32>,
}

impl Tile {
    pub fn new(ctx: &mut Context, ttype: TileType, tpos: mint::Point2<f32>, tsize: mint::Point2<f32>) -> Self {
        let timage = match ttype {
            TileType::Floor => graphics::Image::new(ctx, "/images/castle_floor.png"),
            TileType::Wall => graphics::Image::new(ctx, "/images/castle_wall.png"),
        }; // od tipa polja nece zavisiti samo slika, 
           // vec i stvari kao sto su kolizija, osvetljenje itd.  
        Tile {
            tile_type: ttype,
            tile_image: timage.unwrap(),
            tile_pos: tpos,
            tile_size: tsize,
        }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        /* Mozda je bolje crtanje prepustiti samoj mapi, da bi mogli da pomeramo celu mapu
         * ili eventualno napravimo i minimapu. U tom slucaju bi tile_pos bila pozicija u matrici
         * mape, a ne na samom ekranu, a ova draw funkcija bi mogla da uzima tacku levog gornjeg
         * ugla mape kao argument.
         */
        graphics::draw(ctx, &self.tile_image, graphics::DrawParam::new()
                       .dest(self.tile_pos))?;
        Ok(())
    }
}

pub struct Map {
    map_size: mint::Point2<f32>,
    map_start: mint::Point2<f32>,
    map_tile_size: mint::Point2<f32>,
    map_matrix: Vec<Vec<Tile>>,
}

impl Map {
    pub fn load<P>(ctx: &mut Context, filename: P) -> GameResult<Self> 
    where 
        P: AsRef<Path>,
    {
        let mut map_file = filesystem::open(ctx, filename)?;
        let mut map_string: String = "".to_string();
        map_file.read_to_string(&mut map_string)?;
        let mut map_lines = map_string.lines();
        
        // U prvoj liniji treba da bude 4 broja - 
        // map_width map_heigth tile_width tile_heigth
        let first_line: Vec<&str> = map_lines.next().unwrap().split(' ').collect();
        let map_width: f32 = first_line[0].parse().unwrap();
        let map_heigth: f32 = first_line[1].parse().unwrap();
        let tile_width: f32 = first_line[2].parse().unwrap();
        let tile_heigth: f32 = first_line[3].parse().unwrap();

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
                                                       TileType::Wall, 
                                                       mint::Point2 { x: curr_x, y: curr_y }, 
                                                       mint::Point2 { x: tile_width, y: tile_heigth})),
                    ' ' => curr_row_vec.push(Tile::new(ctx,
                                                       TileType::Floor, 
                                                       mint::Point2 { x: curr_x, y: curr_y }, 
                                                       mint::Point2 { x: tile_width, y: tile_heigth})),
                    _   => (),
                }
                curr_x += tile_width;
            }
            matrix.push(curr_row_vec.clone());
            curr_row_vec.clear();
            curr_x = 0.0;
            curr_y += tile_heigth;
        }

        Ok(Map {
            map_size: mint::Point2 { x: map_width, y: map_heigth }, // ovo je broj polja na mapi
            map_start: mint::Point2 { x: 0.0, y: 0.0 },
            map_tile_size: mint::Point2 { x: tile_width, y: tile_heigth },
            map_matrix: matrix,
          })
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for row in self.map_matrix.iter() {
            for tile in row.iter() {
                tile.draw(ctx)?;
            }
        }
        Ok(())
    }
}
