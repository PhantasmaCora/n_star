use std::f64::consts::PI;

use ndarray::prelude::*;

use rand::RngExt;
use rand::rngs::ChaCha20Rng;


use crate::mapgen::CarverHandle;
use crate::mapgen::{RoomMaker, DesiredRoomSize};
use crate::mapgen::carver_handle::BoolViewBatchHandle;


pub struct SuperFormulaParameter {
    pub fa: u32,
    pub fb: u32,
    pub a: f64,
    pub b: f64,
    pub na: f64,
    pub nb: f64,
    pub n: f64,
    pub innate_scale: f64,
    pub use_min: bool,
    pub use_first_only: bool
}

impl SuperFormulaParameter {
    pub fn draw( &self, handle: &mut impl CarverHandle, angle: f64, scale: f64 ) {
        let sz = handle.dim();

        let mut arr = [0.0f64; 32];

        for a in 0..arr.len() {
            let theta = a as f64 * 2.0 * PI / arr.len() as f64;

            let mut thr = self.thresh( theta );

            if self.use_first_only {

            } else if self.use_min {
                thr = thr.min( self.thresh( theta + 2.0 * PI ) );
            } else {
                thr = thr.max( self.thresh( theta + 2.0 * PI ) );
            }

            arr[a] = thr;
        }

        let center = (sz.0 / 2, sz.1 / 2);

        for x in 0..sz.0 {
            for y in 0..sz.1 {
                let diff = (x as f64 - center.0 as f64, y as f64 - center.1 as f64);

                let rad = ( diff.0.powi(2) + diff.1.powi(2) ).sqrt();

                let mut theta = diff.0.atan2( diff.1 ) + angle + 2.0 * PI;

                theta %= 2.0 * PI;

                let idx = theta / 2.0 / PI * arr.len() as f64;
                let u_idx = idx.floor() as usize;

                let s1 = arr[u_idx];
                let mut s2 = arr[0];
                if idx < (arr.len() - 1) as f64 {
                    s2 = arr[u_idx + 1];
                }

                let inter = idx - idx.floor();

                let thr = s2 * inter + s1 * (1.0 - inter);

                if rad <= thr * scale * self.innate_scale {
                    handle.carve( (x as usize, y as usize) );
                }
            }
        }

        handle.push_batch();
    }

    fn thresh(&self, angle: f64) -> f64 {
        let ta = angle * self.fa as f64 / 4.0;
        let tb = angle * self.fb as f64 / 4.0;

        let diva = ta.cos().abs() / self.a;
        let divb = tb.sin().abs() / self.b;

        let sum = diva.powf( self.na ) + divb.powf( self.nb );

        sum.powf( -1.0 / self.n )
    }
}

impl RoomMaker for SuperFormulaParameter {
    fn make_room_boolean<'a>(&self, av: &mut ArrayViewMut<'a, bool, Ix2>, rng: &mut ChaCha20Rng) {
        let sz = av.dim();

        let square_side = sz.0.min(sz.1);

        let square_x = rng.random_range(0..sz.0 - square_side + 1);
        let square_y = rng.random_range(0..sz.1 - square_side + 1);

        let square_view = av.slice_mut(s![square_x..square_x+square_side, square_y..square_y+square_side]);
        let mut ch = BoolViewBatchHandle::new(square_view);

        self.draw( &mut ch, rng.random::<f64>() * 2.0f64 * PI, (square_side as f64 / 2.0) - 0.75 );
    }

    fn desired_sizes(&self) -> DesiredRoomSize {
        DesiredRoomSize::Min(16)
    }
}

