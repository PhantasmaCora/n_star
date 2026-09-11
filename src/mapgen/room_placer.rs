use std::collections::{HashSet};


use ndarray::prelude::*;
use ndarray_conv::{ConvExt, ConvMode, PaddingMode};

use imageproc::image::{RgbImage, Rgb};

use rand::rngs::ChaCha20Rng;
use rand::RngExt;

use crate::mapgen::{RoomMaker, DesiredRoomSize};


pub struct RoomPlacer {
    pub makers: Vec<(f32, Box<dyn RoomMaker>)>,
    pub max_size: usize,
    pub decrement_size: (usize, usize),
    pub wall_scale: usize
}


impl RoomPlacer {
    pub fn make_rooms<'a>(&self, av: &mut ArrayViewMut<'a, usize, Ix2>, max_tries: usize, max_rooms: usize, rng: &mut ChaCha20Rng) {
        let mut max_size = self.max_size;
        let mut rooms = 0;

        for i in 0..max_tries {
            rooms += 1;
            if rooms > max_rooms {
                break;
            }

            if self.decrement_size.0 > 0 && i % self.decrement_size.0 == 0 && i != 0 {
                max_size -= self.decrement_size.1;
            }

            let f: f32 = rng.random();
            let mut maker = &self.makers.get(self.makers.len() - 1).as_ref().unwrap().1;

            for (chance, m) in self.makers.iter() {
                if f < *chance {
                    maker = m;
                    break;
                }
            }

            let mut max_s = max_size;
            let mut min_s = max_s / 2;
            let ds = maker.desired_sizes();

            match ds {
                DesiredRoomSize::AnySize => {},
                DesiredRoomSize::Min(u) => { min_s = min_s.max(u); },
                DesiredRoomSize::Max(u) => { max_s = max_s.min(u); min_s = max_s / 2; },
                DesiredRoomSize::MinMax(a,b) => {
                    min_s = min_s.max(a);
                    max_s = max_s.min(b);
                }
            }
            if min_s > max_s {
                continue;
            }

            let w = rng.random_range(min_s..=max_s);
            let h = rng.random_range(min_s..=max_s);

            let mut arr = Array2::<bool>::from_elem( (w,h), true );

            {
                maker.make_room_boolean( &mut arr.view_mut(), rng );
            }

            let mut kernel = arr.map( |b| {if *b {0} else {1}} );

            if self.wall_scale > 0 {
                let expand = Array2::<usize>::from_elem( (2*self.wall_scale+1, 2*self.wall_scale+1), 1);
                kernel = kernel.conv(&expand, ConvMode::Full, PaddingMode::Zeros ).unwrap();
            }

            let conv = av.conv( &kernel, ConvMode::Full, PaddingMode::Zeros ).unwrap();

            let mut possible: Vec<(usize, usize)> = conv.slice( s![self.wall_scale+w..self.wall_scale+av.dim().0, self.wall_scale+h..self.wall_scale+av.dim().1] ).into_indexed_iter().filter_map( |(p, v)| if *v == 0 {Some(p)} else {None} ).collect();

            if possible.len() == 0 {
                continue;
            }

            let (x, y) = possible[ rng.random_range(0..possible.len()) ];

            let mut paste_slice = av.slice_mut(s![ x..x+w, y..y+h ]);

            azip!( (c in &arr, p in &mut paste_slice) if !c { *p = 1usize } );
        }

    }
}
