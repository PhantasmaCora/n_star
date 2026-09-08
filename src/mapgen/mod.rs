use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};

use ndarray::prelude::*;

use rand::{RngExt, SeedableRng};
use rand::rngs::ChaCha20Rng;


use crate::map::{Tile, Map};



mod lw_mapalgo;
pub use lw_mapalgo::LightweightMap;

mod carver_handle;
use carver_handle::{BoolViewBatchHandle, WideChainCarverHandle};

mod limited_carver_handle;
use limited_carver_handle::{LimitedCarverHandle, MapBatchHandle};

mod cellauto;
use cellauto::JAGGED_CAVES;

mod schism;
use schism::SchismCarver;

mod conncomp;
use conncomp::{ConnCompLabeler, ConnCompTunneler};

mod binary_partition;
use binary_partition::{BinaryPartitioner, BinaryPartitionRuinGrid};


pub struct MapGenerator {
    pub w: usize,
    pub h: usize
}

impl MapGenerator {
    pub fn generate_map(&self, tileset: Vec<Tile>) -> Map {
        let mut rng = ChaCha20Rng::from_seed([12,9,3,4,5,6,7,8,255,24,3,4,5,6,7,8,1,2,3,12,5,6,7,8,9,2,3,4,5,6,7,8]);

        let mut barr = Array2::<bool>::from_elem((self.w, self.h), true);//self.init_bool_arr(0.32, (1, self.w-1, 1, self.h-1), &mut rng);

        {
            let view = barr.slice_mut(s![1..self.w-1, 1..self.h-1]);
            let mut ch = BoolViewBatchHandle::new(view);

            let sc = SchismCarver{
                param_a: 0.95,
                param_b: 0.24,
                scale_min: 12.0,
                scale_max: 36.0,
                angle_min: 2.0,
                angle_max: 3.0
            };

            sc.carve_schisms( &mut ch, 24, &mut rng );

            let cct = ConnCompTunneler{
                labeler: ConnCompLabeler::four()
            };

            {
                let mut wide_ch = WideChainCarverHandle{
                    chain: Box::new(&mut ch),
                    radius: (2,2)
                };
                cct.connect_all_astar(&mut wide_ch, (0.05, 1.0, 1.5), &mut rng);
            }

            JAGGED_CAVES.run_cell_auto(&mut ch, 20000, 2, &mut rng );

            cct.connect_all_astar(&mut ch, (0.1, 1.0, 1.0), &mut rng);
        }

        let uarr = Array2::<usize>::default( (self.w, self.h) );
        let tsm = TileSetMapper{};

        let mut uarr = tsm.map_tile(barr.borrow());

        {
            let view = uarr.slice_mut(s![self.w/2..3*self.w/4, 2*self.h/7..self.h/2]);
            //let mut ich = IndependentPointSetCarverHandle::from_view(view, false, true);

            let bpg = BinaryPartitionRuinGrid {
                grid_x: 4,
                grid_y: 4,
                partitioner: BinaryPartitioner {
                    min_w: 2,
                    max_w: 5,
                    min_h: 2,
                    max_h: 5,
                    split_chance: 0.3
                },
                types: vec![(0.5, 0), (0.7, 3), (1000.0, 2)],
                edge_perl: (-0.8, 0.5),
                edge_scale: 12.0,
                noise_scale: (0.5, 0.5),
                logistic_params: (1.0, 0.5),
                dont_break: { let mut hs = HashSet::new(); hs.insert(1); hs },
                corner_scale: (1, 1)
            };

            bpg.make_partition_grid( view, &mut rng );

        }

        /*{ // Catacombs preset
            let view = barr.slice_mut(s![1..self.w-1, 1..self.h-1]);
            let mut ch = BoolViewBatchHandle::new(view);

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

        }*/



        {
            let view = uarr.slice_mut(s![1..self.w-1, 1..self.h-1]);

            let mut breaks = HashMap::new();
            breaks.insert(2,3);

            let mut fills = HashMap::new();
            fills.insert(0,1);
            fills.insert(3,2);

            let mut solid = HashSet::new();
            solid.insert(1);
            solid.insert(2);

            let mut lch = MapBatchHandle::<usize>::new(view, breaks, fills, solid);

            let cct = ConnCompTunneler{
                labeler: ConnCompLabeler::four()
            };

            cct.limited_cull_small(&mut lch, 6);
            cct.connect_limited_astar( &mut lch, (0.1, 1.0, 0.96), &mut rng );
        }



        let map = Map {
            tileset,
            tiles: uarr,
            exclusive_occupancy: HashMap::new(),
            non_exclusive_occupancy: HashMap::new()
        };

        // add test items
        /*let proto = InvItem {
            display_name: "Strange Rock".to_string(),
            display_ch: 'º',
            color: (128, 208, 208),
            can_stack: 1,
            stack: 1,
            size: ItemSize::Volume(1.5),
            attaches_as: None,
            flavor_text: "Small, you'd almost call it a pebble. Not a type of stone you've seen before...".to_string(),
            lick_result: LickResponse::NonBioRefusal
        };

        for _i in 0..64 {
            let mut rx = rng.random_range(1..self.w-1);
            let mut ry = rng.random_range(1..self.h-1);

            while !map.is_coord_passable( (rx, ry) ) {
                rx = rng.random_range(1..self.w-1);
                ry = rng.random_range(1..self.h-1);
            }

            map.add_neo( NonExclusiveOccupant::Item( proto.clone() ), (rx as i32, ry as i32) );
        }

        let proto = InvItem {
            display_name: "Canister".to_string(),
            display_ch: 'Ü',
            color: (128, 128, 128),
            can_stack: 0,
            stack: 1,
            size: ItemSize::Bulky,
            attaches_as: None,
            flavor_text: "Some manner of storage vesssel. It's not clear what's inside or how to get it open.".to_string(),
            lick_result: LickResponse::LongText(vec!["#[]Alloy, notes of tungsten.".to_string(), "#[]Can probably store some harsh stuff in here...".to_string()], 45)
        };

        for _i in 0..16 {
            let mut rx = rng.random_range(1..self.w-1);
            let mut ry = rng.random_range(1..self.h-1);

            while !map.is_coord_passable( (rx, ry) ) {
                rx = rng.random_range(1..self.w-1);
                ry = rng.random_range(1..self.h-1);
            }

            map.add_neo( NonExclusiveOccupant::Item( proto.clone() ), (rx as i32, ry as i32) );
        }*/

        map
    }


    pub fn init_bool_arr(&self, init_frac: f64, area: (usize, usize, usize, usize), rng: &mut ChaCha20Rng) -> Array2<bool> {
        Array2::<bool>::from_shape_fn((self.w, self.h), | (i, j) | -> bool {
            if i < area.0 || i >= area.1 || j < area.2 || j >= area.3 { return true; }
            return rng.random_bool( init_frac as f64 );
        })
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


pub struct TileSetMapper {}

impl TileSetMapper {
    fn map_tile( &self, src_arr: &ArrayRef<bool, Ix2> ) -> Array2<usize> {
        let mut out = Array2::<usize>::default( src_arr.dim() );

        for x in 0..src_arr.dim().0 {
            for y in 0..src_arr.dim().1 {
                if !src_arr[[x,y]] {
                    continue;
                }
                out[[x,y]] = 1;
            }
        }

        out
    }

}
