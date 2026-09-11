use std::cmp;
use std::collections::{VecDeque, HashSet};

use rand::{RngExt, rngs::ChaCha20Rng};

use ndarray::prelude::*;

use noise::{NoiseFn, Perlin};

use crate::mapgen::CarverHandle;

use crate::mapgen::RoomMaker;


pub struct URect {
    x: usize,
    y: usize,
    w: usize,
    h: usize
}



pub struct BinaryPartitioner {
    pub min_w: usize,
    pub max_w: usize,
    pub min_h: usize,
    pub max_h: usize,
    pub split_chance: f64
}

impl BinaryPartitioner {
    pub fn make_partition(&self, big_size: (usize, usize), rng: &mut ChaCha20Rng) -> Vec<URect> {
        let mut working = VecDeque::<URect>::new();
        let mut done = Vec::<URect>::new();

        working.push_back( URect{x: 0, y: 0, w: big_size.0, h: big_size.1} );

        while !working.is_empty() {
            let current = working.pop_front().unwrap();

            let mut do_split = true;
            if !( current.h > self.max_h || current.w > self.max_w ) {
                do_split = rng.random_bool(self.split_chance);
            }

            if do_split {

                let can_split_h = current.h >= 2 * self.min_h;
                let can_split_v = current.w >= 2 * self.min_w;

                let mut do_h = true;

                if can_split_h && can_split_v {
                    do_h = rng.random();
                } else if !can_split_h && !can_split_v {
                    done.push(current);
                    continue;
                } else if !can_split_h {
                    do_h = false;
                }

                if do_h {
                    let split_height = rng.random_range( self.min_h..=(current.h - self.min_h) );

                    working.push_back( URect{ x: current.x, y: current.y, w: current.w, h: split_height } );
                    working.push_back( URect{x: current.x, y: current.y + split_height, w: current.w, h: current.h - split_height} );

                } else {
                    let split_column = rng.random_range( self.min_w..=(current.w - self.min_w) );

                    working.push_back( URect{x: current.x, y: current.y, w: split_column, h: current.h} );
                    working.push_back( URect{x: current.x + split_column, y: current.y, w: current.w - split_column, h: current.h } );
                }
            } else {
                done.push(current);
            }
        }

        done
    }

}


pub struct BinaryPartitionRooms {
    pub makers: Vec<(f32, Box<dyn RoomMaker>)>,
    pub grid_x: usize,
    pub grid_y: usize,
    pub partitioner: BinaryPartitioner
}

impl BinaryPartitionRooms {
    pub fn make_rooms<'a>(&self, mut av: ArrayViewMut<'a, bool, Ix2>, rng: &mut ChaCha20Rng) {
        let size = av.dim();

        let part_size = (size.0 / self.grid_x, size.1 / self.grid_y);

        let part = self.partitioner.make_partition( part_size, rng );

        //let offset_x = rng.random_range(0..self.grid_x);
        //let offset_y = rng.random_range(0..self.grid_y);

        for rect in part {
            let x = rect.x * self.grid_x;// + offset_x;
            let y = rect.y * self.grid_y;// + offset_y;
            let w = rect.w * self.grid_x;
            let h = rect.h * self.grid_y;

            let mut rm_av = av.slice_mut(s![x..x+w, y..y+h]);

            let f: f32 = rng.random();

            for (chance, maker) in self.makers.iter() {
                if f < *chance {
                    maker.make_room_boolean(&mut rm_av, rng);
                    break;
                }
            }

        }

    }


}






pub struct BinaryPartitionGrid {
    pub grid_x: usize,
    pub grid_y: usize,
    pub partitioner: BinaryPartitioner
}

impl BinaryPartitionGrid {

    pub fn make_partition_grid(&self, handle: &mut impl CarverHandle, rng: &mut ChaCha20Rng) {
        let size = handle.dim();

        let part_size = (size.0 / self.grid_x, size.1 / self.grid_y);

        let part = self.partitioner.make_partition( part_size, rng );

        let offset_x = rng.random_range(0..self.grid_x);
        let offset_y = rng.random_range(0..self.grid_y);

        for rect in part {
            let x = rect.x * self.grid_x + offset_x;
            let y = rect.y * self.grid_y + offset_y;
            let w = rect.w * self.grid_x;
            let h = rect.h * self.grid_y;

            for ox in 0..w {
                let pt = ( x + ox, y + h );
                if let Some(b) = handle.inspect( pt ) {
                    if !b {
                        handle.fill(pt);
                    }

                }
            }

            for oy in 0..h {
                let pt = (x + w, y + oy);
                if let Some(b) = handle.inspect( pt ) {
                    if !b {
                        handle.fill(pt);
                    }

                }
            }
        }

        handle.push_batch();
    }
}


pub struct BinaryPartitionRuinGrid {
    pub grid_x: usize,
    pub grid_y: usize,
    pub types: Vec<(f64, usize)>,
    pub edge_perl: (f64, f64),
    pub edge_scale: f64,
    pub noise_scale: (f64, f64),
    pub corner_scale: (i32, i32),
    pub logistic_params: (f64, f64),
    pub dont_break: HashSet<usize>,
    pub partitioner: BinaryPartitioner
}

impl BinaryPartitionRuinGrid {

    pub fn make_partition_grid<'a>(&self, mut av: ArrayViewMut<'a, usize, Ix2>, rng: &mut ChaCha20Rng) {
        let size = av.dim();

        let part_size = (size.0 / self.grid_x, size.1 / self.grid_y);

        let part = self.partitioner.make_partition( part_size, rng );

        let offset_x = rng.random_range(0..self.grid_x);
        let offset_y = rng.random_range(0..self.grid_y);

        //let mut corners = HashSet::<(usize, usize)>::new();

        /*for rect in part {
            let x = rect.x * self.grid_x + offset_x;
            let y = rect.y * self.grid_y + offset_y;
            let w = rect.w * self.grid_x;
            let h = rect.h * self.grid_y;

            corners.insert( (x,y) );
            corners.insert( (x+w, y) );
            corners.insert( (x, y+h) );
            corners.insert( (x+w, y+h) );
        }*/

        let sz = av.dim();

        let mut work_arr = Array2::<f64>::default( sz );

        let perlin = Perlin::new( rng.random() );

        for x in 0..sz.0 {
            for y in 0..sz.1 {
                let edgeness = cmp::min( cmp::min(x, y), cmp::min( sz.0 - x, sz.1 - y ) );
                let mut edgeness = edgeness as f64;
                if self.edge_scale - edgeness > 0.0 {
                    edgeness = self.edge_scale - edgeness / self.edge_scale;
                } else {
                    edgeness = 0.0;
                }

                let perlness = perlin.get( [x as f64 * self.noise_scale.0, y as f64 * self.noise_scale.1] );

                let sum = self.edge_perl.0 * edgeness + self.edge_perl.1 + perlness;

                work_arr[[x,y]] = 1.0 / (1.0 + 1f64.exp().powf( - self.logistic_params.0 * (sum - self.logistic_params.1) ));
            }
        }

        for rect in part {
            let x = rect.x * self.grid_x + offset_x;
            let y = rect.y * self.grid_y + offset_y;
            let w = rect.w * self.grid_x;
            let h = rect.h * self.grid_y;

            for ox in 0..=w {
                let pt = ( x + ox, y + h );
                if pt.0 < sz.0 && pt.1 < sz.1 && !self.dont_break.contains( &av[[pt.0, pt.1]] ) {
                    let r: f64 = rng.random();
                    let factor = r + work_arr[[pt.0,pt.1]];
                    for (threshold, tile) in self.types.iter() {
                        if factor < *threshold {
                            av[[pt.0,pt.1]] = *tile;
                            break;
                        }
                    }
                }
            }

            for oy in 0..h {
                let pt = (x + w, y + oy);
                if pt.0 < sz.0 && pt.1 < sz.1 && !self.dont_break.contains( &av[[pt.0, pt.1]] ) {
                    let r: f64 = rng.random();
                    let factor = r + work_arr[[pt.0,pt.1]];
                    for (threshold, tile) in self.types.iter() {
                        if factor < *threshold {
                            av[[pt.0,pt.1]] = *tile;
                            break;
                        }
                    }
                }
            }
        }
    }
}
