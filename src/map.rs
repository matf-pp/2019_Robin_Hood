use std::io::Read;
use std::path::Path;
use ggez::*;
use na::{Vector2, Isometry2};
use ncollide2d::shape::{Cuboid, Compound, ShapeHandle};
use ncollide2d::world::{CollisionGroups, CollisionObjectHandle, CollisionWorld, GeometricQueryType};


#[derive(Debug, Clone)]
pub enum TileType {
    Floor(graphics::Rect, i32),
    Wall(graphics::Rect, i32),
}

#[derive(Debug, Clone)] // Clone nam treba da bi mogli da kopiramo vektore
pub struct Tile {
    tile_type: TileType,
    tile_src: graphics::DrawParam,
    pub tile_layer: i32,
    tile_pos: mint::Point2<f32>,
    tile_size: mint::Point2<f32>,
}

impl Tile {
    pub fn new(ttype: TileType, tpos: mint::Point2<f32>, tsize: mint::Point2<f32>) -> Self {
        let (tsrc, tlayer) = match ttype {
            TileType::Floor(d, l) => (graphics::DrawParam::new().src(d), l),
            TileType::Wall(d, l)  => (graphics::DrawParam::new().src(d), l),
        }; // od tipa polja nece zavisiti samo slika, 
        // vec i stvari kao sto su kolizija, osvetljenje itd.  
        Tile {
            tile_type: ttype,
            tile_src: tsrc,
            tile_layer: tlayer,
            tile_pos: tpos,
            tile_size: tsize,
        }
    }

    pub fn drawparam(&self, map_start: mint::Point2<f32>) -> graphics::DrawParam {
        /* Kada crtamo mapu, crtamo jednu sliku sa razlicitim parametrima vise puta.
         * Jedan bitan parametar je src (self.tile_src), gde odredjujemo koji deo slike ce se
         * crtati, a drugi je pozicija slike, sto ovde racunamo.
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
    pub map_handle: CollisionObjectHandle,
}

impl Map {
    pub fn load<P>(ctx: &mut Context, level_filename: P, image_filename: P, startpos: mint::Point2<f32>, tile_size: mint::Point2<f32>, world_mut: &mut CollisionWorld<f32, ()>) -> GameResult<Self> 
        where 
        P: AsRef<Path>,
        {
            let spritesheet = graphics::Image::new(ctx, image_filename)?;
            let swidth: f32 = spritesheet.width() as f32;
            let sheigth: f32 = spritesheet.height() as f32;
            let tfrac: mint::Point2<f32> = mint::Point2 { x: tile_size.x/swidth, y: tile_size.y/sheigth };
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

            let shape_full = ShapeHandle::new(Cuboid::new(Vector2::new(16.0, 16.0)));
            let shape_quart = ShapeHandle::new(Cuboid::new(Vector2::new(16.0, 8.0)));
            let query = GeometricQueryType::Contacts(0.0, 0.0);
            let mut col_groups = CollisionGroups::new();
            col_groups.set_membership(&[1 as usize]);
            col_groups.set_blacklist(&[1 as usize]);
            col_groups.set_whitelist(&[0 as usize]);
            let mut compound_shape_vec: Vec<(Isometry2<f32>, ShapeHandle<f32>)> = Vec::new(); 

            while let Some(line) = map_lines.next() {
                for c in line.chars() {
                    match c {
                        // treba pratiti x i y poziciju svakog polja, i na osnovu karaktera sa te
                        // pozicije dodati polje odgovarajuceg tipa u matricu mape
                        '1' => curr_row_vec.push(Tile::new(TileType::Wall([0.0, 0.0, tfrac.x, tfrac.y].into(), 2),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '2' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x, 0.0, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '3' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*2.0, 0.0, tfrac.x, tfrac.y].into(), 2),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '4' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*3.0, 0.0, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '5' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*4.0, 0.0, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '6' => curr_row_vec.push(Tile::new(TileType::Wall([0.0, tfrac.y*1.0, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '7' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x, tfrac.y, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '8' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*2.0, tfrac.y, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '9' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*3.0, tfrac.y, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        'A' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*4.0, tfrac.y, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        'B' => curr_row_vec.push(Tile::new(TileType::Wall([0.0, tfrac.y*2.0, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        'C' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x, tfrac.y*2.0, tfrac.x, tfrac.y].into(), 2),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        'D' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*2.0, tfrac.y*2.0, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        ' ' => curr_row_vec.push(Tile::new(TileType::Floor([tfrac.x*3.0, tfrac.y*2.0, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        _   => (),
                    }
                    match curr_row_vec[curr_x as usize].tile_type {
                        TileType::Wall(_, l) => {
                            if l != 2 {
                                compound_shape_vec.push((Isometry2::new(Vector2::new(startpos.x+curr_x*tile_size.x, startpos.y+curr_y*tile_size.y), 0.0), shape_full.clone()));
                            } else {
                                compound_shape_vec.push((Isometry2::new(Vector2::new(startpos.x+curr_x*tile_size.x, startpos.y+curr_y*tile_size.y+16.0), 0.0), shape_quart.clone()));
                            }
                            ()
                        },
                        _ => ()
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
                map_handle: world_mut.add(Isometry2::new(Vector2::new(startpos.x, startpos.y), 0.0), ShapeHandle::new(Compound::new(compound_shape_vec)), col_groups, query, ()).handle(),
            })
        }

    pub fn draw(&mut self, ctx: &mut Context, layer: i32, show_mesh: bool) -> GameResult<()> {
        for row in self.map_matrix.iter() {
            for tile in row.iter() {
                if tile.tile_layer == layer {
                    self.map_spritebatch.add(tile.drawparam(self.map_start));
                }
            }
        }
        self.map_spritebatch.set_filter(graphics::FilterMode::Nearest);
        graphics::draw(ctx, &self.map_spritebatch, graphics::DrawParam::new())?;
        self.map_spritebatch.clear();

        if show_mesh {
            let mut tile_mesh: graphics::Mesh;
            for row in self.map_matrix.iter() {
                for tile in row.iter() {
                    if tile.tile_layer == layer {
                        match tile.tile_type {
                            TileType::Wall(_, _) => {
                                tile_mesh = graphics::MeshBuilder::new().rectangle(graphics::DrawMode::stroke(3.0), [tile.drawparam(self.map_start).dest.x, tile.drawparam(self.map_start).dest.y, tile.tile_size.x, tile.tile_size.y].into(), [0.0, 1.0, 0.0, 1.0].into()).build(ctx)?;
                                graphics::draw(ctx, &tile_mesh, graphics::DrawParam::new())?;
                                ()
                            },
                            TileType::Floor(_, _) => (),
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
