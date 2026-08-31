use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
use std::ops::BitOr;

use ndarray::prelude::*;

use rand::{Rng, RngExt, SeedableRng};
use rand::rngs::ChaCha20Rng;

use bracket_lib::geometry::{Point, Rect};
use bracket_lib::pathfinding::a_star_search;

use crate::map::{Tile, Map};


mod lw_mapalgo;
pub use lw_mapalgo::LightweightMap;

mod cellauto;
use cellauto::{SimpleBooleanCellAuto, SOFT_CAVES, JAGGED_CAVES};

mod schism;
use schism::SchismCarver;


pub struct MapGenerator {
    pub w: usize,
    pub h: usize
}

impl MapGenerator {
    pub fn generate_map(&self, tileset: Vec<Tile>) -> Map {
        let mut rng = ChaCha20Rng::from_seed([1,2,3,4,5,6,7,8,1,2,3,4,5,6,7,8,1,2,3,12,5,6,7,8,1,2,3,4,5,6,7,8]);

        let mut barr = Array2::<bool>::from_elem( (self.w, self.h), true );

        let carver = SchismCarver{
            param_a: 0.95,
            param_b: 0.25,
            scale_min: 7.5,
            scale_max: 24.0,
            angle_min: 2.6,
            angle_max: 2.8
        };
        carver.carve_schisms( barr.slice_mut(s![2..self.w-2, 2..self.h-2]), 32, &mut rng );

        //barr = JAGGED_CAVES.run_cell_auto( &mut barr, Rect{ x1: 1, x2: self.w as i32 - 1, y1: 1, y2: self.h as i32 - 1 }, 20000, 2, &mut rng );

        let cct = ConnCompTunneler{
            labeler: ConnCompLabeler::<bool>::eight()
        };

        cct.cull_small( barr.view_mut(), |a|{!a}, true, 12 );
        cct.connect_all_astar( barr.view_mut(), |a|{!a}, false, ( 0.1, 1.0, 1.2 ), &mut rng );

        //let mut barr = self.bool_cell_auto( barr, 512, 1, (5, 9, 3, 9), (1, 9, 7, 9), &mut rng);

        //cct.connect_all_astar( barr.view_mut(), |a|{!a}, false, &mut rng );

        let mut uarr = Array2::<usize>::default( (self.w, self.h) );
        let tsm = TileSetMapper{};

        let uarr = tsm.map_tile(barr.borrow());

        return Map{
            tileset,
            tiles: uarr,
            exclusive_occupancy: HashMap::<(i32, i32), String>::new()
        }
    }


    pub fn init_bool_arr(&self, init_frac: f64, area: (usize, usize, usize, usize), rng: &mut ChaCha20Rng) -> Array2<bool> {
        Array2::<bool>::from_shape_fn((self.w, self.h), | (i, j) | -> bool {
            if i < area.0 || i > area.1 || j < area.2 || j > area.3 { return true; }
            return rng.random_bool( init_frac as f64 );
        })
    }

    pub fn set_bool_arr(&self, mut arr: ArrayViewMut<bool, Ix2>, frac: f64, area: (usize, usize, usize, usize), rng: &mut ChaCha20Rng) {
        for x in area.0..area.1 {
            for y in area.2..area.3 {
                arr[[x,y]] = rng.random_bool( frac as f64 );
            }
        }
    }

}


pub struct ConnCompLabeler<A: Copy> {
    pub eight_connectivity: bool,
    ghost: PhantomData<A>
}

impl<A: Copy> ConnCompLabeler<A> {
    pub fn eight() -> ConnCompLabeler<A> {
        return Self{
            eight_connectivity: true,
            ghost: PhantomData
        }
    }

    pub fn four() -> ConnCompLabeler<A> {
        return Self{
            eight_connectivity: false,
            ghost: PhantomData
        }
    }

    pub fn label<F: Fn(A)->bool>(&self, arr: &ArrayRef<A, Ix2>, predicate: F) -> Vec<HashSet<(usize, usize)>> {

        // list of connected components
        let mut ccs = HashMap::<usize, HashSet<(usize, usize)>>::new();

        // maps array labels to set indices
        let mut label_map = HashMap::<usize, usize>::new();

        // working array of labels, matches size of input
        let mut label_arr = Array2::<usize>::default(arr.dim());

        // working dimensions
        let (sx,sy) = arr.dim();
        let sx = sx as i32;
        let sy = sy as i32;

        let mut current_label = 0;

        let mut scans = vec![(-1, 0), (0, -1)];
        if self.eight_connectivity {
            scans.push((-1,-1));
            scans.push((1,-1));
        }

        for x in 0i32..sx {
            for y in 0i32..sy {
                if !predicate( arr[(x as usize, y as usize)] ) {
                    continue;
                }

                let mut found = HashSet::<usize>::new(); // labels found on prior cells
                for s in scans.iter() {
                    if x+s.0 < 0 || x+s.0 >= sx || y+s.1 < 0 || y+s.1 >= sy {
                        continue; // out of bounds check
                    }
                    let adj = label_arr[[(x+s.0) as usize, (y+s.1) as usize]];
                    if adj > 0 {
                        found.insert(adj);
                    }
                }

                let fv: Vec<usize> = found.drain().collect();

                if fv.len() == 0 {
                    // new label added!
                    current_label += 1;
                    label_arr[[x as usize, y as usize]] = current_label;

                    label_map.insert(current_label, current_label);

                    let mut hs = HashSet::new();
                    hs.insert((x as usize, y as usize));
                    ccs.insert(current_label, hs);

                } else if fv.len() == 1 {
                    // add point to existing label

                    let label = fv[0];
                    label_arr[[x as usize, y as usize]] = label;
                    let idx = label_map[&label];
                    ccs.get_mut(&idx).unwrap().insert((x as usize, y as usize));

                } else {
                    //print!("{:?}\n", fv);

                    // we need to do a merge
                    let dominant_label = *fv.iter().max().unwrap();
                    let didx = label_map[&dominant_label];
                    let mut ds = ccs.remove(&didx).unwrap();

                    label_arr[[x as usize,y as usize]] = dominant_label;

                    for lab in fv.iter() {
                        if !( label_map[lab] == label_map[&dominant_label] ) {
                            //print!("merge {} to {}\n", lab, dominant_label);
                            //print!("indices: {}, {}\n", label_map[&lab], didx);

                            let idx = label_map[&lab];

                            for (k,v) in label_map.iter_mut() {
                                if *v == idx {
                                    *v = didx;
                                }
                            }

                            let set = ccs.remove(&idx).unwrap();
                            ds = ds.bitor(&set);
                        }/* else if *lab != dominant_label {
                            print!("skipping merge {} to {}\n", lab, dominant_label);
                        }*/
                    }

                    ccs.insert(didx, ds);
                }
            }
        }

        let cc: Vec<HashSet<(usize, usize)>> = ccs.into_values().collect();
        cc
    }

}

pub struct ConnCompTunneler<A: Copy> {
    labeler: ConnCompLabeler<A>
}

impl<A: Copy> ConnCompTunneler<A> {
    pub fn cull_small<F: Fn(A)->bool>(&self, mut arr: ArrayViewMut<A, Ix2>, predicate: F, fill: A, min_size: usize) {
        let mut ccs = self.labeler.label(arr.borrow(), predicate);

        for cc in ccs.iter() {
            if cc.len() < min_size {
                // fill in the small gap
                for (x,y) in cc.iter() {
                    arr[[*x,*y]] = fill;
                }
            }
        }
    }

    pub fn connect_all_astar<F: Fn(A)->bool>(&self, mut arr: ArrayViewMut<A, Ix2>, predicate: F, opening: A, weights: (f32, f32, f32), rng: &mut ChaCha20Rng ) {
        let mut ccs = self.labeler.label(arr.borrow(), &predicate);

        // necessary for determinism; otherwise the hasher randomization will impact results
        ccs.sort_by_key( |cc| cc.len() );

        let mut max_iter = 256;

        while ccs.len() > 1 && max_iter > 0 {
            max_iter -= 1;

            let mut comp_a = ccs.pop().unwrap();

            let size = ccs.len();
            let mut comp_b = ccs.remove( rng.random_range(0..size) );

            // average out for centers.
            let sum_a = comp_a.iter().fold( (0,0), |a: (usize, usize), e: &(usize, usize)|{(a.0 + e.0, a.1 + e.1)} );
            let center_a = ( (sum_a.0 as f32 / comp_a.len() as f32) as usize, (sum_a.1 as f32 / comp_a.len() as f32) as usize );

            let sum_b = comp_b.iter().fold( (0,0), |a: (usize, usize), e: &(usize, usize)|{(a.0 + e.0, a.1 + e.1)} );
            let center_b = ( (sum_b.0 as f32 / comp_b.len() as f32) as usize, (sum_b.1 as f32 / comp_b.len() as f32) as usize );

            // find nearest point in set
            let mut collect_a: Vec<(usize, usize)> = comp_a.drain().collect();
            collect_a.sort_by_key( |pt| pt.0.abs_diff( center_b.0 ) + pt.1.abs_diff( center_b.1 ) );
            let target_a = collect_a[ 0 ];

            let mut collect_b: Vec<(usize, usize)> = comp_b.drain().collect();
            collect_b.sort_by_key( |pt| pt.0.abs_diff( center_a.0 ) + pt.1.abs_diff( center_a.1 ) );
            let target_b = collect_b[ 0 ];

            let asz = arr.dim();
            let mut work_arr = Array2::<f32>::default( asz );

            for x in 0..asz.0 {
                for y in 0..asz.1 {
                    let e = arr[[x,y]];

                    if comp_a.contains(&(x,y)) || comp_b.contains(&(x,y)) {
                        continue; // keep the zero cost for space in either component
                    }

                    if predicate(e) {
                        work_arr[[x,y]] = weights.0; // low cost for empty space
                    } else {
                        let mut adj = false;
                        for dx in -1..=1 {
                            for dy in -1..=1 {
                                if (dx == 0 && dy == 0) || (dx < 0 && x == 0) || (dy < 0 && y == 0) { continue; }
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;

                                if nx >= asz.0 || ny >= asz.1 { continue; }

                                let ne = arr[[nx, ny]];
                                if predicate(ne) {
                                    adj = true;
                                    break;
                                }
                            }
                            if adj {break;}
                        }
                        if adj {
                            work_arr[[x,y]] = weights.2; // high cost for room border walls
                        } else {
                            work_arr[[x,y]] = weights.1; // medium cost for walls
                        }
                    }
                }
            }

            let lwm = LightweightMap{ array: &work_arr };
            let astar = a_star_search( lwm.point_to_index(target_a), lwm.point_to_index(target_b), &lwm );
            let path = astar.steps;

            //let path = candidates.remove(min_idx as usize);
            for idx in path.iter() {
                let point = lwm.index_to_point(*idx);
                arr[[point.0, point.1]] = opening;
            }

            // re-label since things got cut open
            ccs = self.labeler.label(arr.borrow(), &predicate);
            ccs.sort_by_key( |cc| cc.len() );
        }
    }

    // "Good Enough" implementation
    pub fn connect_all_dogleg<F: Fn(A)->bool, C: Fn(A)->f32>(&self, mut arr: ArrayViewMut<A, Ix2>, predicate: F, cost: C, opening: A, rng: &mut ChaCha20Rng ) {
        let mut ccs = self.labeler.label(arr.borrow(), &predicate);

        // necessary for determinism; otherwise the hasher randomization will impact results
        ccs.sort_by_key( |cc| cc.len() );

        let mut max_iter = 256;

        while ccs.len() > 1 && max_iter > 0 {
            max_iter -= 1;

            let mut comp_a = ccs.pop().unwrap();

            let size = ccs.len();
            let comp_b = ccs.remove( rng.random_range(0..size) );

            // average out for centers. change later???
            let sum_a = comp_a.iter().fold( (0,0), |a: (usize, usize), e: &(usize, usize)|{(a.0 + e.0, a.1 + e.1)} );
            let center_a = ( (sum_a.0 as f32 / comp_a.len() as f32) as usize, (sum_a.1 as f32 / comp_a.len() as f32) as usize );

            let sum_b = comp_b.iter().fold( (0,0), |a: (usize, usize), e: &(usize, usize)|{(a.0 + e.0, a.1 + e.1)} );
            let center_b = ( (sum_b.0 as f32 / comp_b.len() as f32) as usize, (sum_b.1 as f32 / comp_b.len() as f32) as usize );

            // distance to cover
            let diff = ( center_a.0 as i32 - center_b.0 as i32, center_a.1 as i32 - center_b.1 as i32 );

            let min_x = usize::min(center_a.0, center_b.0);
            let min_y = usize::min(center_a.1, center_b.1);
            let diff = (i32::abs(diff.0) as usize, i32::abs(diff.1) as usize);

            let mut candidates = Vec::<HashSet<[usize; 2]>>::new();

            for _i in 0..32 {
                let mut hs = HashSet::<[usize; 2]>::new();

                let choice = rng.random_bool(0.5);

                if (choice || diff.1 == 0) && diff.0 > 0 {
                    let dl = rng.random_range( 0..diff.0 );
                    for x in 0..diff.0 {
                        let mut y = 0;
                        if x > dl {
                            y = diff.1
                        }
                        hs.insert( [ min_x + x, min_y + y ] );
                    }
                    for y in 0..diff.1 {
                        hs.insert( [ min_x + dl, min_y + y ] );
                    }
                } else {
                    let dl = rng.random_range( 0..diff.1 );
                    for y in 0..diff.1 {
                        let mut x = 0;
                        if y > dl {
                            x = diff.0
                        }
                        hs.insert( [ min_x + x, min_y + y ] );
                    }
                    for x in 0..diff.0 {
                        hs.insert( [ min_x + x, min_y + dl ] );
                    }
                }

                candidates.push(hs);
            }

            let mut min_cost = 1000000.0;
            let mut min_idx: i32 = -1;
            for (idx, item) in candidates.iter().enumerate() {
                let mut nc = 0.0;
                for point in item.iter() {
                    let a = arr[*point];
                    nc += cost(a);
                }
                if nc < min_cost {
                    min_cost = nc;
                    min_idx = idx as i32;
                }
            }

            let path = candidates.remove(min_idx as usize);
            for point in path.iter() {
                arr[*point] = opening;
            }

            // re-label since things got cut open
            ccs = self.labeler.label(arr.borrow(), &predicate);
            ccs.sort_by_key( |cc| cc.len() );
        }
    }
}

pub struct TileSetMapper {}

impl TileSetMapper {
    fn map_tile(&self, src_arr: &ArrayRef<bool, Ix2>) -> Array2<usize> {
        let mut out = Array2::<usize>::default( src_arr.dim() );

        let offs = vec![(-1,0), (1,0), (0,-1), (0,1)];

        for x in 0..src_arr.dim().0 {
            for y in 0..src_arr.dim().1 {
                if !src_arr[[x,y]] {
                    continue;
                }

                let mut determinant = 0;

                for (idx, o) in offs.iter().enumerate() {
                    let ox = (x as i32 + o.0) as usize;
                    let oy = (y as i32 + o.1) as usize;

                    if (o.0 < 0 && x == 0) || ox >= src_arr.dim().0 || ( o.1 < 0 && y == 0 ) || oy >= src_arr.dim().1 || src_arr[[ox, oy]] {
                        determinant += 2_i32.pow(idx as u32);
                    }
                }

                out[[x,y]] = match determinant {
                    15 => 6,
                    7 => 2,
                    11 => 3,
                    13 => 5,
                    14 => 4,
                    _ => 1
                }
            }
        }

        out
    }

}
