
use ndarray::prelude::*;

use rand::{Rng, RngExt};
use rand::rngs::ChaCha20Rng;
use rand::distr::Uniform;

use bracket_lib::geometry::Rect;

// smin, smax, bmin, bmax
pub struct SimpleBooleanCellAuto {
    r_params: (u8,u8,u8,u8),
    s_params: (u8,u8,u8,u8)
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

impl SimpleBooleanCellAuto {
    pub fn run_cell_auto(&self, start_arr: &mut ArrayRef2<bool>, area: Rect, rand_iters: usize, smooth_iters: usize, rng: &mut ChaCha20Rng ) -> Array2<bool> {
        // initialize
        let mut buff_a = start_arr;

        let sz = buff_a.dim();

        // random iterations
        let xdist = Uniform::new( area.x1 as usize, area.x2 as usize ).unwrap();
        let ydist = Uniform::new( area.y1 as usize, area.y2 as usize ).unwrap();

        for i in 0..rand_iters {
            let x = rng.sample(xdist);
            let y = rng.sample(ydist);

            let st = buff_a[(x,y)];

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
                    if buff_a[( (x as i32 + dx) as usize, (y as i32 + dy) as usize )] {
                        count += 1;
                    }
                }
            }

            /*if i % 1000 == 0 {
             *           print!("{}", count);
        }*/

            if st && (count < self.r_params.0 || count > self.r_params.1) {
                buff_a[(x,y)] = false; // death
            } else if (!st) && ( count >= self.r_params.2 && count <= self.r_params.3 ) {
                buff_a[(x,y)] = true; // birth
            }
        }

        // smoothing passes
        let mut buff_b = buff_a.to_owned();

        for i in 0..smooth_iters {
            let mut src = buff_a.view_mut();
            let mut dest = buff_b.view_mut();

            if i % 2 == 1 {
                // swap buffers
                let swap = src;
                src = dest;
                dest = swap;
            }

            // test each
            for x in area.x1 as usize..=area.x2 as usize {
                for y in area.y1 as usize..=area.y2 as usize {

                    let st = src[(x,y)];

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
                            if src[( (x as i32 + dx) as usize, (y as i32 + dy) as usize )] {
                                count += 1;
                            }
                        }
                    }

                    if st && (count < self.s_params.0 || count > self.s_params.1) {
                        dest[(x,y)] = false; // death
                    } else if st {
                        dest[(x,y)] = true; // survival
                    } else if !st && ( count >= self.s_params.2 && count <= self.s_params.3 ) {
                        dest[(x,y)] = true; // birth
                    } else {
                        dest[(x,y)] = false // stayin dead
                    }
                }
            }
        }

        if smooth_iters % 2 == 1 {
            return buff_b;
        } else {
            return buff_a.to_owned();
        }
    }
}
