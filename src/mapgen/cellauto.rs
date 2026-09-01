
use ndarray::prelude::*;

use rand::{Rng, RngExt};
use rand::rngs::ChaCha20Rng;
use rand::distr::Uniform;

use bracket_lib::geometry::Rect;

use crate::mapgen::CarverHandle;

// smin, smax, bmin, bmax
pub struct SimpleBooleanCellAuto {
    pub r_params: (u8,u8,u8,u8),
    pub s_params: (u8,u8,u8,u8)
}

// use with 0.45 to 5 init and ~ 40000 rand, 6 smooth
pub const SOFT_CAVES: SimpleBooleanCellAuto = SimpleBooleanCellAuto{
    r_params: (4,9,5,9),
    s_params: (3,9,5,9)
};

// use with 0.2 to 0.35 init and ~ 15000 rand, 3 smooth
pub const JAGGED_CAVES: SimpleBooleanCellAuto = SimpleBooleanCellAuto{
    r_params: (4,9,3,9),
    s_params: (4,9,3,9)
};

// 0.37 init with 15000, 3, (4,9,4,7), (4,9,3,9) for somewhat smaller jagged caves
// 0.26 init with 16000, 4, (3,5,2,4), (4,9,3,9) makes cramped fragments, that get connected by long corridors.

// requires a batched op type handle!
impl SimpleBooleanCellAuto {
    pub fn run_cell_auto(&self, handle: &mut impl CarverHandle, rand_iters: usize, smooth_iters: usize, rng: &mut ChaCha20Rng ) {
        // initialize

        let sz = handle.dim();

        // random iterations
        let xdist = Uniform::new( 0, sz.0 ).unwrap();
        let ydist = Uniform::new( 0, sz.1 ).unwrap();

        for i in 0..rand_iters {
            let x = rng.sample(xdist);
            let y = rng.sample(ydist);

            let st = handle.inspect((x,y)).unwrap();

            let mut count = 0;
            for dx in -1i32..=1 {
                for dy in -1i32..=1 {
                    if x as i32 + dx < 0 || x as i32 + dx >= sz.0 as i32 || y as i32 + dy < 0 || y as i32 + dy >= sz.1 as i32 {
                        count += 1;
                        continue;
                    }
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    if handle.inspect( ( (x as i32 + dx) as usize, (y as i32 + dy) as usize ) ).unwrap() {
                        count += 1;
                    }
                }
            }

            if st && (count < self.r_params.0 || count > self.r_params.1) {
                handle.carve( (x,y) ); // death
            } else if (!st) && ( count >= self.r_params.2 && count <= self.r_params.3 ) {
                handle.fill( (x,y) ); // birth
            }

            handle.push_batch();
        }

        // smoothing passes
        for i in 0..smooth_iters {

            // test each
            for x in 0..sz.0 {
                for y in 0..sz.1 {

                    let st = handle.inspect( (x,y) ).unwrap();

                    let mut count = 0;
                    for dx in -1i32..=1 {
                        for dy in -1i32..=1 {
                            if x as i32 + dx < 0 || x as i32 + dx >= sz.0 as i32 || y as i32 + dy < 0 || y as i32 + dy >= sz.1 as i32 {
                                count += 1;
                                continue;
                            }
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            if handle.inspect( ( (x as i32 + dx) as usize, (y as i32 + dy) as usize ) ).unwrap() {
                                count += 1;
                            }
                        }
                    }

                    if st && (count < self.s_params.0 || count > self.s_params.1) {
                        handle.carve((x,y)); // death
                    } else if !st && ( count >= self.s_params.2 && count <= self.s_params.3 ) {
                        handle.fill((x,y)); // birth
                    }
                }
            }

            handle.push_batch();
        }

    }
}
