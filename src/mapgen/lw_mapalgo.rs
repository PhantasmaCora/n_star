
use ndarray::prelude::*;

use bracket_lib::prelude::BaseMap;
use bracket_lib::pathfinding::{SmallVec};



pub struct LightweightMap<'a> {
    pub array: &'a ArrayRef<f32, Ix2>
}

impl<'a> LightweightMap<'a> {
    pub fn point_to_index(&self, pt: (usize, usize)) -> usize {
        let mapsize = self.array.dim();
        ( pt.0 + pt.1 * mapsize.0 ) as usize
    }

    pub fn index_to_point(&self, idx: usize) -> (usize, usize) {
        let mapsize = self.array.dim();
        let width = mapsize.0 as usize;
        let x: usize = idx as usize % width;
        let y: usize = (idx as usize - x) / width;
        (x,y)
    }
}

impl<'a> BaseMap for LightweightMap<'a> {
    fn get_pathing_distance(&self, idx1: usize, idx2: usize ) -> f32 {
        let pt1 = self.index_to_point(idx1);
        let pt2 = self.index_to_point(idx2);
        f32::sqrt( ( pt1.0.abs_diff(pt2.0).pow(2) + pt1.1.abs_diff(pt2.1).pow(2) ) as f32 )
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let coords = self.index_to_point(idx);
        let sz = self.array.dim();
        let sz = (sz.0 as i32, sz.1 as i32);
        let offsets = vec![(-1,0), (1,0), (0,-1), (0,1)];

        let mut exits = SmallVec::<[(usize, f32); 10]>::new();

        for o in offsets {
            let dp = ( coords.0 as i32 + o.0, coords.1 as i32 + o.1 );

            if dp.0 < 0 || dp.0 >= sz.0 || dp.1 < 0 || dp.1 >= sz.1 {
                continue;
            }

            let cost = self.array[[dp.0 as usize, dp.1 as usize]];
            let didx = self.point_to_index( (dp.0 as usize, dp.1 as usize) );
            exits.push( (didx, cost) );
        }
        exits
    }

}

