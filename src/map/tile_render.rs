use std::collections::{HashMap, HashSet};

use crate::map::Tile;

pub struct TileRenderContext {
    pub me: usize,
    pub neighbors: [Option<usize>; 8]
}

pub enum TileDrawType {
    Dont,
    Regular(char, bool),
    Override{ch: char, fg: (u8,u8,u8), bg: (u8,u8,u8)}
}

pub trait TileRender {
    // char for character, bool for swap colors
    fn get_draw(&self, ctx: TileRenderContext, ts: &Vec<Tile>) -> TileDrawType;
}

#[derive(Clone, Copy)]
pub struct FixedTileRender {
    pub ch: char
}

impl TileRender for FixedTileRender {
    fn get_draw(&self, ctx: TileRenderContext, ts: &Vec<Tile>) -> TileDrawType {
        TileDrawType::Regular(self.ch, false)
    }
}

#[derive(Clone)]
pub struct Wall4WayTileRender {
    pub connects: HashSet<usize>,
    pub lower: (char, bool),
    pub upper: (char, bool),
    pub left: (char, bool),
    pub right: (char, bool),
    pub vertical: (char, bool),
    pub horizontal: (char, bool),
    pub misc: (char, bool)
}

impl TileRender for Wall4WayTileRender {
    fn get_draw(&self, ctx: TileRenderContext, ts: &Vec<Tile>) -> TileDrawType {
        let mapping: [usize;4] = [1, 3, 4, 6];

        let mut determinant = 0;
        let mut look = false;

        for (idx, shift_index) in mapping.iter().enumerate() {
            if let Some(tidx) = ctx.neighbors[*shift_index] {
                if self.connects.contains(&tidx) {
                    determinant += 2_i32.pow(idx as u32);
                }
                if !ts[tidx].opaque {
                    look = true;
                }
            } else {
                determinant += 2_i32.pow(idx as u32);
            }
        }

        if !look {
            return TileDrawType::Dont;
        }

        return match determinant {
            6 => TileDrawType::Regular( self.horizontal.0, self.horizontal.1 ),
            9 => TileDrawType::Regular( self.vertical.0, self.vertical.1 ),
            7 => TileDrawType::Regular( self.lower.0, self.lower.1 ),
            11 => TileDrawType::Regular( self.right.0, self.right.1 ),
            14 => TileDrawType::Regular( self.upper.0, self.upper.1 ),
            13 => TileDrawType::Regular( self.left.0, self.left.1 ),
            _ => TileDrawType::Regular( self.misc.0, self.misc.1 ),
        };
    }
}
