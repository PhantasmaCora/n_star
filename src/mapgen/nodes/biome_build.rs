use std::cell::OnceCell;
use std::collections::{HashMap};

use serde::{Serialize, Deserialize};

use crate::map::{Tile};
use crate::mapgen::nodes::{MapNode, MapNodeCalculation, MapNodeLink, MapNodeData};




#[derive(Serialize, Deserialize)]
pub struct BiomeBuilder {
    pub name: String,
    pub valid_floors: Vec<i32>,
    pub seed_salt: i32,
    pub tileset: Vec<Tile>,
    pub nodes: HashMap<String, NodeBuilder>
}

#[derive(Serialize, Deserialize)]
pub struct NodeBuilder {
    pub calc_type: String,
    pub links: HashMap<String, MapNodeLink>
}

impl NodeBuilder {
    pub fn make_node(&self) -> MapNode {
        let calc: Box<dyn MapNodeCalculation> = match self.calc_type.as_str() {
            "MakeGridI" | "MakeGridI32" => { Box::new( crate::mapgen::make_grid_ops::MakeGridI32{} ) },
            "MakeGridF" | "MakeGridF32" => { Box::new( crate::mapgen::make_grid_ops::MakeGridF32{} ) },
            _=> { panic!("Invalid mapgen node calculation type: {}", self.calc_type) }
        };

        MapNode {
            calc,
            links: self.links.clone(),
            cache: OnceCell::<Vec<MapNodeData>>::new()
        }
    }
}





