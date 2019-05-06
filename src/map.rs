use std::io::Read;
use std::f32::consts::PI;
use std::path::Path;
use ggez::*;
use na::{Vector2, Isometry2, Point2};
use ncollide2d::shape::{Cuboid, Compound, ConvexPolygon, ShapeHandle};
use ncollide2d::world::{CollisionGroups, CollisionObjectHandle, CollisionWorld, GeometricQueryType};
use rand::{thread_rng, Rng};

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

pub struct Gold {
    pos: mint::Point2<f32>,
    image: graphics::Image,
    value: i32,
    handle: CollisionObjectHandle,
}
impl Gold {
    pub fn new (ctx: &mut Context, point1: mint::Point2<f32>, point2: mint::Point2<f32>, handle1: CollisionObjectHandle) -> Self {
// handle ce nam kasnije pomoci da odredimo da li je igrac dodirnuo zlato (pokupio zlato)
        let mut rng = thread_rng();
        let num : i32 = rng.gen_range (1,4);


        Gold {
            pos: mint::Point2 { x: rng.gen_range(point1.x, point2.x),
            y: rng.gen_range(point1.y, point2.y) },
            image: graphics::Image::new(ctx, format!("/images/gold{}.png", num)).unwrap(),
            value: match num {
                1 => 5,
                2 => 15,
                3 => 50,
                _ => 0,
            },
            handle: handle1,
        }
    }
    pub fn update(&mut self, world: &mut CollisionWorld<f32, ()>, player_handle: CollisionObjectHandle, map_vel: Vector2<f32>) -> i32 {
        self.pos.x += map_vel.x;
        self.pos.y += map_vel.y;
        world.set_position(self.handle, Isometry2::new(Vector2::new(self.pos.x, self.pos.y), 0.0));
        match world.contact_pair(self.handle, player_handle, true) {
            // contact_pair vraca uredjenu cetvorku koja opisuje da li se desio sudar
            None => 0,
            _ => {
                world.remove(&[self.handle]); // remove brise iz CollisionWorld ali ne brise ceo objekat
                self.value
            },
        }
    }
    pub fn draw (&self, ctx: &mut Context) -> GameResult<()> {
        graphics::draw(ctx, &self.image, graphics::DrawParam::new().dest(self.pos))?;
        Ok(())
    }
}

pub struct Map {
    map_size: mint::Point2<f32>,
    map_start: mint::Point2<f32>,
    map_vel: Vector2<f32>,
    map_spd: f32,
    map_tile_size: mint::Point2<f32>,
    map_corners: Vec<mint::Point2<f32>>,
    map_matrix: Vec<Vec<Tile>>,
    map_spritebatch: graphics::spritebatch::SpriteBatch,
    pub map_handle: CollisionObjectHandle,
    map_guards: Vec<Guard>,
    map_gold: Vec<Gold>,
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
            let query = GeometricQueryType::Contacts(0.0, 0.0); // definisemo sta znaci dodir igraca i zlata i igraca i zida
            let mut col_groups = CollisionGroups::new();
            col_groups.set_membership(&[1 as usize]); // kojim grupama pripada objekat
            col_groups.set_blacklist(&[1 as usize]); // sa kojim grupama ne moze da interaguje objekat
            col_groups.set_whitelist(&[0 as usize]); // sa kojim grupama objekat moze da interaguje
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
                                compound_shape_vec.push((Isometry2::new(Vector2::new(curr_x*tile_size.x, curr_y*tile_size.y), 0.0), shape_full.clone()));
                            } else {
                                compound_shape_vec.push((Isometry2::new(Vector2::new(curr_x*tile_size.x, curr_y*tile_size.y+16.0), 0.0), shape_quart.clone()));
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
            let mut gold_vec: Vec<Gold> = Vec::new();

            let shape_gold = ShapeHandle::new(Cuboid::new(Vector2::new(8.0, 8.0))); // "okvir" zlata, njegove granice da bismo mogli da definisemo "sudar" igraca i zlata


            let first_dir = Vector2::new(0.0, 1.0);
            let vec1 = Isometry2::new(Vector2::new(0.0,0.0), PI/6.0).transform_vector(&first_dir)*64.0;
            let vec2 = Isometry2::new(Vector2::new(0.0,0.0), -PI/6.0).transform_vector(&first_dir)*64.0;
            let origin_point = Point2::new(0.0, 0.0);
            let triangle_points: [Point2<f32>; 3] = [origin_point, origin_point+vec1, origin_point+vec2]; 

            let shape_triangle = ShapeHandle::new(ConvexPolygon::try_from_points(&triangle_points).unwrap());


            while let Some(guard_line) = map_lines.next() {
                // posle opisa mape u txt fajlu sledi nekoliko redova koji
                // imaju informacije o koordinatama soba, broju strazara i broju
                // novcica
                let split_space: Vec<&str> = guard_line.split(' ').collect();

                let point1_vec: Vec<&str> = split_space[0].split(',').collect();
                let point1_x: f32 = point1_vec[0].parse().unwrap();
                let point1_y: f32 = point1_vec[1].parse().unwrap();

                let point1: mint::Point2<f32> = mint::Point2 { x: startpos.x + point1_x*tile_size.x, y: startpos.y + point1_y*tile_size.y };

                let point2_vec: Vec<&str> = split_space[1].split(',').collect();
                let point2_x: f32 = point2_vec[0].parse().unwrap();
                let point2_y: f32 = point2_vec[1].parse().unwrap();

                let point2: mint::Point2<f32> = mint::Point2 { x: startpos.x + point2_x*tile_size.x, y: startpos.y + point2_y*tile_size.y };

                let number_of_guards: i32 = split_space[2].parse().unwrap();
                let number_of_points: i32 = split_space[3].parse().unwrap();
                let number_of_coins: i32 = split_space[4].parse().unwrap();

                for _i in 0..number_of_guards {
                    guards_vec.push(Guard::new(ctx, point1, point2, number_of_points,
                        world_mut.add(Isometry2::new(Vector2::new(startpos.x, startpos.y), 0.0), // world_mut je neophodan zbog sudaranja igraca i zlata
                         shape_triangle.clone(),
                         col_groups,
                         query,
                         ()).handle())); // do handle je poziv funkcije world_mut.add koja dodaje objekat u svet za koliziju (ne crta ga)

                }

                for _i in 0..number_of_coins { // pravimo vektor koji sadrzi svo zlato na mapi
                    gold_vec.push(Gold::new(ctx, point1, point2,
                        world_mut.add(Isometry2::new(Vector2::new(startpos.x, startpos.y), 0.0), // world_mut je neophodan zbog sudaranja igraca i zlata
                         shape_gold.clone(),
                         col_groups,
                         query,
                         ()).handle())); // do handle je poziv funkcije world_mut.add koja dodaje objekat u svet za koliziju (ne crta ga)
                }

            }

            Ok(Map {
                map_size: mint::Point2 { x: map_width, y: map_heigth }, // ovo je broj polja na mapi
                map_start: startpos,
                map_vel: Vector2::new(0.0, 0.0),
                map_spd: 4.0,
                map_tile_size: tile_size,
                map_corners: corner_points,
                map_matrix: matrix,
                map_spritebatch: graphics::spritebatch::SpriteBatch::new(spritesheet),
                map_handle: world_mut.add(Isometry2::new(Vector2::new(startpos.x, startpos.y), 0.0), ShapeHandle::new(Compound::new(compound_shape_vec)), col_groups, query, ()).handle(),
                map_guards: guards_vec,
                map_gold: gold_vec,
            })
        }

    pub fn update(&mut self, world: &mut CollisionWorld<f32, ()>, dir: Vector2<f32>) {
        let dir_norm = if dir.x != 0.0 || dir.y != 0.0 {
            (-dir).normalize()
        } else {
            dir
        };
        self.map_vel = dir_norm*self.map_spd;
        self.map_start.x += self.map_vel.x;
        self.map_start.y += self.map_vel.y;
        world.set_position(self.map_handle, Isometry2::new(Vector2::new(self.map_start.x, self.map_start.y), 0.0));
    }

    pub fn update_gold(&mut self, world: &mut CollisionWorld<f32, ()>, player_handle: CollisionObjectHandle) -> i32 {
        // argumenti su isti kao za update pojedinacnog golda
        let mut zbir: i32 = 0;
        let mut duzina = self.map_gold.len();
        let mut i: usize = 0;
        while i < duzina {
            let sudaren = self.map_gold[i].update(world, player_handle, self.map_vel);
            match sudaren { //vraca 0 ako se igrac nije sudario sa zlatom,i neku vrednost ako jeste
                0 => (),
                vrednost => {
                    self.map_gold.remove(i); // brise i-ti gold iz vektora pa se taj gold vise nece updateovati i crtati
                    zbir += vrednost;
                    // ovaj zbir prosledjujemo igracu da bi pri svakom
                    // updateu promenio score. Zato se zbiru dodeljuje nula pri svakom pozivu
                }
            }
            duzina = self.map_gold.len(); // updateujemo zbog brisanja elemenata
            i += 1;
        }
        zbir
    }

    pub fn update_guards(&mut self, world: &mut CollisionWorld<f32, ()>, player_handle: CollisionObjectHandle) -> bool {
        let mut res: bool = false;
        for i in 0..self.map_guards.len() {
            res = res || self.map_guards[i].update(world, player_handle, self.map_vel);
        }
        res
    }

    pub fn get_corners(&mut self) -> Vec<mint::Point2<f32>> {
        self.map_corners.clone().into_iter().map(|c| mint::Point2 { x: self.map_start.x + c.x*self.map_tile_size.x,
            y: self.map_start.y + c.y*self.map_tile_size.y }).collect()
    }

    pub fn draw_gold(&mut self, ctx: &mut Context) -> GameResult<()> {
        for gold in self.map_gold.iter() {
            gold.draw(ctx)?;
        }
        Ok(())
        // ova funkcija crta na ekran sve zlatnike tj pojedinacno poziva draw za svaki gold
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
