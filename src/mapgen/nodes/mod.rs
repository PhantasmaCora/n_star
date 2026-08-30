use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};

use serde::{Serialize, Deserialize};
use ndarray::{Array2};

use rand::rngs::ChaCha20Rng;

use bracket_lib::geometry::{Point, Rect};

use crate::map::{Tile, Map};



pub mod biome_build;

pub mod make_grid_ops;



// sadly this mostly just Panics if something goes wrong for now, we'll add more validation later on

#[derive(Serialize, Deserialize, Clone)]
pub enum MapNodeLink {
    Linked { ancestor: String, socket: usize },
    StaticInput { config: MapNodeConfigItem }
}


#[derive(Serialize, Deserialize, Clone)]
pub enum MapNodeConfigItem {
    Int(i32),
    Float(f32),
    Rect(i32, i32, i32, i32)
}

impl MapNodeConfigItem {
    fn to_data(&self) -> MapNodeData {
        match self {
            MapNodeConfigItem::Int(i) => {
                return MapNodeData::Int(*i);
            }
            MapNodeConfigItem::Float(f) => {
                return MapNodeData::Float(*f);
            }
            MapNodeConfigItem::Rect(x,y,w,h) => {
                return MapNodeData::Rect( Rect{x1: *x, x2: *x+*w, y1: *y, y2: *y+*h} );
            }
        }

    }

}

#[derive(Clone)]
enum MapNodeData {
    GridI32(Array2<i32>),
    GridF32(Array2<f32>),
    Point((usize, usize)),
    PointSet(HashSet<(usize, usize)>),
    Rect(Rect),
    RectList(Vec<Rect>),
    Int(i32),
    Float(f32)
}

impl MapNodeData {
    fn int(&self) -> i32 {
        if let MapNodeData::Int(i) = &self {
            return *i;
        } else {
            panic!();
        }
    }

    fn float(&self) -> f32 {
        if let MapNodeData::Float(f) = &self {
            return *f;
        } else {
            panic!();
        }
    }

    fn grid_i(&self) -> Array2<i32> {
        if let MapNodeData::GridI32(g) = &self {
            return g.clone();
        } else {
            panic!();
        }
    }

    fn grid_f(&self) -> Array2<f32> {
        if let MapNodeData::GridF32(g) = &self {
            return g.clone();
        } else {
            panic!();
        }
    }

    fn point(&self) -> (usize, usize) {
        if let MapNodeData::Point(p) = &self {
            return *p;
        } else {
            panic!();
        }
    }

    fn point_set(&self) -> HashSet<(usize, usize)> {
        if let MapNodeData::PointSet(s) = &self {
            return s.clone();
        } else {
            panic!();
        }
    }

    fn rect(&self) -> Rect {
        if let MapNodeData::Rect(r) = &self {
            return r.clone();
        } else {
            panic!();
        }
    }

    fn rect_list(&self) -> Vec<Rect> {
        if let MapNodeData::RectList(l) = &self {
            return l.clone();
        } else {
            panic!();
        }
    }

}



#[derive(Clone, Copy, PartialEq)]
enum MapNodeDataType {
    GridI32,
    GridF32,
    Point,
    PointSet,
    Rect,
    RectList,
    Int,
    Float
}

impl MapNodeDataType {
    // Painful but it had to be done
    pub fn check(&self, data: &MapNodeData) -> bool {
        match self {
            MapNodeDataType::GridI32 => {
                if let MapNodeData::GridI32(_) = data {
                    true
                } else {
                    false
                }
            },
            MapNodeDataType::GridF32 => {
                if let MapNodeData::GridF32(_) = data {
                    true
                } else {
                    false
                }
            },
            MapNodeDataType::Point => {
                if let MapNodeData::Point(_) = data {
                    true
                } else {
                    false
                }
            },
            MapNodeDataType::PointSet => {
                if let MapNodeData::PointSet(_) = data {
                    true
                } else {
                    false
                }
            },
            MapNodeDataType::Rect => {
                if let MapNodeData::Rect(_) = data {
                    true
                } else {
                    false
                }
            },
            MapNodeDataType::RectList => {
                if let MapNodeData::RectList(_) = data {
                    true
                } else {
                    false
                }
            },
            MapNodeDataType::Int => {
                if let MapNodeData::Int(_) = data {
                    true
                } else {
                    false
                }
            },
            MapNodeDataType::Float => {
                if let MapNodeData::Float(_) = data {
                    true
                } else {
                    false
                }
            },
        }
    }

}

// dyn component for the actual work a MapNode does
pub trait MapNodeCalculation {
    fn get_expected_input(&self) -> HashMap<String, MapNodeDataType>;

    fn get_output_type(&self) -> Vec<MapNodeDataType>;

    fn calc_and_get_output(&self, input: &HashMap<String, MapNodeData>, context: CalcContext) -> Vec<MapNodeData>;
}

// wrapper that handles setup, validation, caching
pub struct MapNode {
    pub calc: Box<dyn MapNodeCalculation>,
    pub links: HashMap<String, MapNodeLink>,
    cache: OnceCell<Vec<MapNodeData>>
}

impl MapNode {
    pub fn get_output(&self, context: &mut MapNodeContext, idx: usize) -> MapNodeData {

        // get_or_init will only run the function if cache is uninitialized

        let dvec = self.cache.get_or_init(|| -> Vec<MapNodeData> {
            let mut input = HashMap::<String, MapNodeData>::new();

            for (k, v) in self.links.iter() {
                match v {
                    MapNodeLink::Linked { ancestor, socket} => {
                        let item = context.others.get( ancestor ).expect( &format!("Node {} not found in Biome {}, called from Node {}", ancestor, context.biome_name, context.my_name) );

                        input.insert( k.to_string(), item.get_output(context, *socket) );
                    }
                    MapNodeLink::StaticInput { config } => {
                        input.insert(k.to_string(), config.to_data());
                    }
                }
            }

            let cctx = CalcContext{
                my_name: context.my_name,
                biome_name: context.biome_name,
                rng: context.rng
            };

            self.calc.calc_and_get_output(&input, cctx)
        });

        let oopt = dvec.get(idx);
        if let Some(out) = oopt {
            return out.clone();
        } else {
            panic!("Unexpected index `{}` into outputs of Mapgen Node {} in Biome {}", idx, context.my_name, context.biome_name);
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache = OnceCell::<Vec<MapNodeData>>::new();
    }
}



pub struct MapNodeContext<'a> {
    pub my_name: &'a str,
    pub biome_name: &'a str,
    pub others: &'a HashMap<String, MapNode>,
    pub rng: &'a mut ChaCha20Rng
}

struct CalcContext<'a> {
    my_name: &'a str,
    biome_name: &'a str,
    rng: &'a mut ChaCha20Rng
}

pub struct Biome {
    pub name: String,
    pub valid_floors: Vec<i32>,
    pub seed_salt: i32,
    pub tileset: Vec<Tile>,
    pub nodes: HashMap<String, MapNode>
}




