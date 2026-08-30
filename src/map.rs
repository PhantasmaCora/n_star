
use std::collections::HashMap;
use std::marker::Copy;

use serde::{Serialize, Deserialize};

use ndarray::{Array2};

use bracket_lib::prelude::*;
use bracket_lib::algorithm_traits::{Algorithm2D, BaseMap};



pub struct Map {
    pub tileset: Vec<Tile>,
    pub tiles: Array2<usize>,
    pub exclusive_occupancy: HashMap<(i32, i32), String>
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub fg: (u8,u8,u8),
    pub bg: (u8,u8,u8),
    pub ch: char,
    pub passable: bool,
    pub opaque: bool,
}


impl Map {
    pub fn empty_map( size: (usize, usize), blank: Tile ) -> Map {
        let tiles = Array2::zeros(size);
        let tileset = vec![blank];
        let exclusive_occupancy = HashMap::new();
        Map {tileset, tiles, exclusive_occupancy}
    }

    pub fn is_passable( &self, idx: usize ) -> bool {
        let pt = self.index_to_point2d(idx);
        self.tileset.get( self.tiles[ ( pt.x as usize, pt.y as usize ) ] ).unwrap().passable
    }
}


impl BaseMap for Map {
    fn is_opaque( &self, idx: usize ) -> bool {
        let pt = self.index_to_point2d(idx);
        self.tileset.get( self.tiles[ ( pt.x as usize, pt.y as usize ) ] ).unwrap().opaque
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let pt = self.index_to_point2d(idx);
        let offsets = [ Point{ x: -1, y: 0 }, Point{ x: 1, y: 0 }, Point{ x: 0, y: -1 }, Point{ x: 0, y: 1 },
        Point{ x: -1, y: -1 }, Point{ x: -1, y: 1 }, Point{ x: 1, y: -1 }, Point{ x: 1, y: 1 }
        ];
        let mut exits = SmallVec::<[(usize, f32); 10]>::new();

        for o in offsets {
            let npt = pt + o;
            if !self.in_bounds(npt) {
                continue;
            }
            let pass = self.tileset.get( self.tiles[ ( npt.x as usize, npt.y as usize ) ] ).unwrap().passable;
            if pass {
                let cost = f32::sqrt( (o.x * o.x + o.y * o.y) as f32 );
                exits.insert( 0, ( self.point2d_to_index(npt), cost ) );
            }
        }
        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize ) -> f32 {
        let pt1 = self.index_to_point2d(idx1);
        let pt2 = self.index_to_point2d(idx2);
        let o = pt2 - pt1;
        f32::sqrt( (o.x * o.x + o.y * o.y) as f32 )
    }

}


impl Algorithm2D for Map {
    fn point2d_to_index(&self, pt: Point) -> usize {
        let mapsize = self.tiles.dim();
        ( pt.x + pt.y * mapsize.0 as i32 ) as usize
    }

    fn index_to_point2d(&self, idx: usize) -> Point {
        let mapsize = self.tiles.dim();
        let width = mapsize.0 as i32;
        let x: i32 = idx as i32 % width;
        let y: i32 = (idx as i32 - x) / width;
        Point{x, y}
    }

    fn dimensions( &self ) -> Point {
        let mapsize = self.tiles.dim();
        Point{ x: mapsize.0 as i32, y: mapsize.1 as i32 }
    }

    fn in_bounds( &self, pos: Point ) -> bool {
        let dim = self.dimensions();
        pos.x >= 0 && pos.x < dim.x && pos.y >= 0 && pos.y < dim.y
    }
}
