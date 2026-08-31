use std::f32::consts::PI;

use ndarray::prelude::*;

use rand::{Rng, RngExt};
use rand::rngs::ChaCha20Rng;
use rand::distr::Uniform;

use bracket_lib::geometry::Rect;



pub struct SchismCarver {
    pub param_a: f32,
    pub param_b: f32,
    pub scale_min: f32,
    pub scale_max: f32,
    pub angle_min: f32,
    pub angle_max: f32
}


impl SchismCarver {
    pub fn carve_schisms(&self, mut arr: ArrayViewMut<bool, Ix2>, n: usize, rng: &mut ChaCha20Rng) {

        let sz = arr.dim();

        for i in 0..n {
            let scale : f32 = rng.random::<f32>() * (self.scale_max - self.scale_min) + self.scale_min;
            let isc = scale.ceil() as i32;

            let mut inner = ( isc, sz.0 as i32 - isc, isc, sz.1 as i32 - isc );

            if inner.0 > inner.1 {
                inner.0 = (inner.0 + inner.1) / 2;
                inner.1 = inner.0;
            }
            if inner.2 > inner.3 {
                inner.2 = (inner.2 + inner.3) / 2;
                inner.3 = inner.2;
            }

            let center_x = rng.random_range( inner.0..=inner.1 );
            let center_y = rng.random_range( inner.2..=inner.3 );

            let angle = rng.random::<f32>() * (self.angle_max - self.angle_min) + self.angle_min;

            for dx in -isc..isc {
                for dy in -isc..isc {
                    let x = center_x + dx;
                    let y = center_y + dy;

                    if x < 0 || x >= sz.0 as i32 || y < 0 || y >= sz.1 as i32 { continue; }

                    let diff = (dx as f32, dy as f32);

                    let rad = ( diff.0.powi(2) + diff.1.powi(2) ).sqrt();

                    let mut theta = diff.0.atan2( diff.1 ) + angle;

                    if theta > PI {
                        theta -= PI;
                    } else if theta < -PI {
                        theta += PI;
                    }

                    if theta > PI/2.0 {

                    } else if theta > 0.0 {
                        theta = PI - theta;
                    } else if theta > -PI/2.0 {
                        theta += PI;
                    } else {
                        theta = 2.0*PI + theta;
                    }

                    let thresh_r = (theta/2.0 - PI/4.0).sin() / ( (1.0 / theta.cos().powi(2) ) - self.param_a );

                    let thresh_r = thresh_r * self.param_b + 1.0;

                    let thresh_r = thresh_r * scale * self.param_b;

                    if rad < thresh_r {
                        arr[[x as usize, y as usize]] = false;
                    }
                }
            }
        }

    }


}
