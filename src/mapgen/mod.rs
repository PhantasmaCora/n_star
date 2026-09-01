use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};

use ndarray::prelude::*;

use rand::{Rng, RngExt, SeedableRng};
use rand::rngs::ChaCha20Rng;

use bracket_lib::geometry::{Point, Rect};

use crate::map::{Tile, Map};


mod lw_mapalgo;
pub use lw_mapalgo::LightweightMap;

mod carver_handle;
use carver_handle::WideChainCarverHandle;

mod cellauto;
use cellauto::{SimpleBooleanCellAuto, SOFT_CAVES, JAGGED_CAVES};

mod schism;
use schism::SchismCarver;

mod conncomp;
use conncomp::{ConnCompLabeler, ConnCompTunneler};


pub struct MapGenerator {
    pub w: usize,
    pub h: usize
}

impl MapGenerator {
    pub fn generate_map(&self, tileset: Vec<Tile>) -> Map {
        let mut rng = ChaCha20Rng::from_seed([12,9,3,4,5,6,7,8,255,2,3,4,5,6,7,8,1,2,3,12,5,6,7,8,9,2,3,4,5,6,7,8]);

        let mut barr = Array2::<bool>::from_elem( (self.w, self.h), true );

        {
            let view = barr.slice_mut(s![0..self.w-1, 2..self.h-1]);
            let mut ch = BoolViewBatchHandle{
                view,
                carve_batch: HashSet::<(usize, usize)>::new(),
                fill_batch: HashSet::<(usize, usize)>::new()
            };

            self.set_bool_arr(&mut ch, 0.45, &mut rng);

            let ca = SimpleBooleanCellAuto{
                r_params: (5,9,4,9),
                s_params: (5,9,3,9)
            };

            ca.run_cell_auto(&mut ch, 0, 3, &mut rng);

            let cct = ConnCompTunneler{
                labeler: ConnCompLabeler::four()
            };
            cct.cull_small( &mut ch, 3 );

            {
                let mut wide_ch = WideChainCarverHandle{
                    chain: Box::new(&mut ch),
                    radius: (1,1)
                };

                cct.connect_all_astar( &mut wide_ch, (0.1, 1.0, 1.2), &mut rng );
            }

        }

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

    pub fn set_bool_arr(&self, handle: &mut impl CarverHandle, frac: f64, rng: &mut ChaCha20Rng) {
        let sz = handle.dim();
        for x in 0..sz.0 {
            for y in 0..sz.1 {
                if rng.random_bool( frac ) {
                    handle.fill((x,y));
                } else {
                    handle.carve((x,y));
                }
            }
        }
        handle.push_batch();
    }

}



pub trait CarverHandle {
    fn dim(&self) -> (usize, usize);

    fn inspect(&self, point: (usize, usize)) -> Option<bool>;

    fn carve(&mut self, point: (usize, usize));

    fn fill(&mut self, point: (usize, usize));

    fn push_batch(&mut self);

    fn discard_batch(&mut self);
}


pub struct BoolViewBatchHandle<'a>{
    view: ArrayViewMut<'a, bool, Ix2>,
    carve_batch: HashSet<(usize, usize)>,
    fill_batch: HashSet<(usize, usize)>
}

impl<'a> CarverHandle for BoolViewBatchHandle<'a> {
    fn dim(&self) -> (usize, usize) {
        self.view.dim()
    }

    fn inspect(&self, point: (usize, usize)) -> Option<bool> {
        let sz = self.view.dim();
        if point.0 >= sz.0 || point.1 >= sz.1 {
            return None;
        } else {
            return Some( self.view[[point.0, point.1]] );
        }
    }

    fn carve(&mut self, point: (usize, usize)) {
        self.fill_batch.remove(&point);
        self.carve_batch.insert(point);
    }

    fn fill(&mut self, point: (usize, usize)) {
        self.carve_batch.remove(&point);
        self.fill_batch.insert(point);
    }

    fn push_batch(&mut self) {
        for p in self.carve_batch.drain() {
            self.view[p] = false;
        }
        for p in self.fill_batch.drain() {
            self.view[p] = true;
        }
    }

    fn discard_batch(&mut self) {
        self.carve_batch.clear();
        self.fill_batch.clear();
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
