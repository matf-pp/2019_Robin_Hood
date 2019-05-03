use std::io::Read;
use std::path::Path;
use ggez::*;
use na::{Vector2, Isometry2};
use ncollide2d::shape::{Cuboid, Compound, ShapeHandle};
use ncollide2d::world::{CollisionGroups, CollisionObjectHandle, CollisionWorld, GeometricQueryType};

use crate::guard::Guard;


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
    map_corners: Vec<mint::Point2<f32>>,
    map_matrix: Vec<Vec<Tile>>,
    map_spritebatch: graphics::spritebatch::SpriteBatch,
    pub map_handle: CollisionObjectHandle,
    map_guards: Vec<Guard>,
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
            let mut corner_points: Vec<mint::Point2<f32>> = Vec::new();

            while curr_y < map_heigth-1.0 {
                let line = map_lines.next().unwrap();
                for c in line.chars() {
                    match c {
                        // treba pratiti x i y poziciju svakog polja, i na osnovu karaktera sa te
                        // pozicije dodati polje odgovarajuceg tipa u matricu mape
                        '1' => {
                            corner_points.push(mint::Point2 { x: curr_x+12.0/32.0, y: curr_y+12.0/32.0});
                            curr_row_vec.push(Tile::new(TileType::Wall([0.0, 0.0, tfrac.x, tfrac.y].into(), 2),
                                                        mint::Point2 { x:curr_x, y:curr_y },
                                                        tile_size))
                        },
                        '2' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x, 0.0, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '3' => {
                            corner_points.push(mint::Point2 { x: curr_x+20.0/32.0, y: curr_y+12.0/32.0 });
                            curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*2.0, 0.0, tfrac.x, tfrac.y].into(), 2),
                                                        mint::Point2 { x:curr_x, y:curr_y },
                                                        tile_size))
                        },
                        '4' => {
                            corner_points.push(mint::Point2 { x: curr_x+1.0, y: curr_y+12.0/32.0 });
                            curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*3.0, 0.0, tfrac.x, tfrac.y].into(), 1),
                                                        mint::Point2 { x:curr_x, y:curr_y },
                                                        tile_size))
                        },
                        '5' => {
                            corner_points.push(mint::Point2 { x: curr_x, y: curr_y+12.0/32.0 });
                            curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*4.0, 0.0, tfrac.x, tfrac.y].into(), 1),
                                                        mint::Point2 { x:curr_x, y:curr_y },
                                                        tile_size))
                        },
                        '6' => curr_row_vec.push(Tile::new(TileType::Wall([0.0, tfrac.y*1.0, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '7' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x, tfrac.y, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '8' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*2.0, tfrac.y, tfrac.x, tfrac.y].into(), 1),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        '9' => {
                            corner_points.push(mint::Point2 { x: curr_x+1.0, y: curr_y });
                            curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*3.0, tfrac.y, tfrac.x, tfrac.y].into(), 1),
                                                        mint::Point2 { x:curr_x, y:curr_y },
                                                        tile_size))
                        },
                        'A' => {
                            corner_points.push(mint::Point2 { x: curr_x, y: curr_y });
                            curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*4.0, tfrac.y, tfrac.x, tfrac.y].into(), 1),
                                                        mint::Point2 { x:curr_x, y:curr_y },
                                                        tile_size))
                        },
                        'B' => {
                            corner_points.push(mint::Point2 { x: curr_x+12.0/32.0, y: curr_y });
                            curr_row_vec.push(Tile::new(TileType::Wall([0.0, tfrac.y*2.0, tfrac.x, tfrac.y].into(), 1),
                                                        mint::Point2 { x:curr_x, y:curr_y },
                                                        tile_size))
                        },
                        'C' => curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x, tfrac.y*2.0, tfrac.x, tfrac.y].into(), 2),
                                                           mint::Point2 { x:curr_x, y:curr_y },
                                                           tile_size)),
                        'D' => {
                            corner_points.push(mint::Point2 { x: curr_x+20.0/32.0, y: curr_y });
                            curr_row_vec.push(Tile::new(TileType::Wall([tfrac.x*2.0, tfrac.y*2.0, tfrac.x, tfrac.y].into(), 1),
                                                        mint::Point2 { x:curr_x, y:curr_y },
                                                        tile_size))
                        },
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
            let mut guards_vec: Vec<Guard> = Vec::new();
            while let Some(guard_line) = map_lines.next() {
                let split_space: Vec<&str> = guard_line.split(' ').collect();
                let point1_vec: Vec<&str> = split_space[0].split(',').collect();
                let point1_x: f32 = point1_vec[0].parse().unwrap();
                let point1_y: f32 = point1_vec[1].parse().unwrap();
                let point1: mint::Point2<f32> = mint::Point2 { x: point1_x*tile_size.x, y: point1_y*tile_size.y };
                let point2_vec: Vec<&str> = split_space[1].split(',').collect();
                let point2_x: f32 = point2_vec[0].parse().unwrap();
                let point2_y: f32 = point2_vec[1].parse().unwrap();
                let point2: mint::Point2<f32> = mint::Point2 { x: point2_x*tile_size.x, y: point2_y*tile_size.y };
                let number_of_guards: i32 = split_space[2].parse().unwrap();
                let number_of_points: i32 = split_space[3].parse().unwrap();
                for _i in 0..number_of_guards {
                    guards_vec.push(Guard::new(ctx, point1, point2, number_of_points));
                }
            }

            Ok(Map {
                map_size: mint::Point2 { x: map_width, y: map_heigth }, // ovo je broj polja na mapi
                map_start: startpos,
                map_tile_size: tile_size,
                map_corners: corner_points,
                map_matrix: matrix,
                map_spritebatch: graphics::spritebatch::SpriteBatch::new(spritesheet),
                map_handle: world_mut.add(Isometry2::new(Vector2::new(startpos.x, startpos.y), 0.0), ShapeHandle::new(Compound::new(compound_shape_vec)), col_groups, query, ()).handle(),
                map_guards: guards_vec,
            })
        }

    pub fn update_guards(&mut self) {
        for i in 0..self.map_guards.len() {
            self.map_guards[i].update();
        }
    }

    pub fn get_corners(&mut self) -> Vec<mint::Point2<f32>> {
        self.map_corners.clone().into_iter().map(|c| mint::Point2 { x: self.map_start.x + c.x*self.map_tile_size.x,
            y: self.map_start.y + c.y*self.map_tile_size.y }).collect()
    }

    pub fn draw_guards(&mut self, ctx: &mut Context) -> GameResult<()> {
        for guard in self.map_guards.iter() {
            guard.draw(ctx)?;
        }
        Ok(())
    }

    pub fn draw_guard_vision(&mut self, ctx: &mut Context) -> GameResult<()> {
        for guard in self.map_guards.iter() {
            guard.draw_vision(ctx)?;
        }
        Ok(())

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
            let mut tile_mesh = graphics::MeshBuilder::new();
            for row in self.map_matrix.iter() {
                for tile in row.iter() {
                    if tile.tile_layer == layer {
                        match tile.tile_type {
                            TileType::Wall(_, _) => {
                                tile_mesh.rectangle(graphics::DrawMode::stroke(3.0), [tile.drawparam(self.map_start).dest.x, tile.drawparam(self.map_start).dest.y, tile.tile_size.x, tile.tile_size.y].into(), [0.0, 1.0, 0.0, 1.0].into());
                                ()
                            },
                            TileType::Floor(_, _) => (),
                        }
                    }
                }
            }
            let built_mesh = tile_mesh.build(ctx)?;
            graphics::draw(ctx, &built_mesh, graphics::DrawParam::new())?;
        }
        Ok(())
    }
}
