use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};

use ndarray::{Array2};

use crate::mapgen::nodes::{MapNodeCalculation, MapNodeData, MapNodeDataType, CalcContext};


pub struct MakeGridI32 {}

impl MapNodeCalculation for MakeGridI32 {
    fn get_expected_input(&self) -> HashMap<String, MapNodeDataType> {
        let mut descriptor = HashMap::<String, MapNodeDataType>::new();

        descriptor.insert("Width".to_string(), MapNodeDataType::Int);
        descriptor.insert("Height".to_string(), MapNodeDataType::Int);
        descriptor.insert("Value".to_string(), MapNodeDataType::Int);

        descriptor
    }

    fn get_output_type(&self) -> Vec<MapNodeDataType> {
        let mut descriptor = Vec::<MapNodeDataType>::new();

        descriptor.push( MapNodeDataType::GridI32 );

        descriptor
    }

    fn calc_and_get_output(&self, input: &HashMap<String, MapNodeData>, context: CalcContext) -> Vec<MapNodeData> {
        let w = input["Width"].int();
        let h = input["Height"].int();
        let v = input["Value"].int();

        let arr = Array2::from_elem( (w as usize, h as usize), v );

        let mnd = MapNodeData::GridI32(arr);

        vec![mnd]
    }
}


pub struct MakeGridF32 {}

impl MapNodeCalculation for MakeGridF32 {
    fn get_expected_input(&self) -> HashMap<String, MapNodeDataType> {
        let mut descriptor = HashMap::<String, MapNodeDataType>::new();

        descriptor.insert("Width".to_string(), MapNodeDataType::Int);
        descriptor.insert("Height".to_string(), MapNodeDataType::Int);
        descriptor.insert("Value".to_string(), MapNodeDataType::Float);

        descriptor
    }

    fn get_output_type(&self) -> Vec<MapNodeDataType> {
        let mut descriptor = Vec::<MapNodeDataType>::new();

        descriptor.push( MapNodeDataType::GridF32 );

        descriptor
    }

    fn calc_and_get_output(&self, input: &HashMap<String, MapNodeData>, context: CalcContext) -> Vec<MapNodeData> {
        let w = input["Width"].int();
        let h = input["Height"].int();
        let v = input["Value"].float();

        let arr = Array2::from_elem( (w as usize, h as usize), v );

        vec![ MapNodeData::GridF32(arr) ]
    }
}


pub struct MakeFullSelection {}

impl MapNodeCalculation for MakeFullSelection {
    fn get_expected_input(&self) -> HashMap<String, MapNodeDataType> {
        let mut descriptor = HashMap::<String, MapNodeDataType>::new();

        descriptor.insert("Rect".to_string(), MapNodeDataType::Rect);

        descriptor
    }

    fn get_output_type(&self) -> Vec<MapNodeDataType> {
        let mut descriptor = Vec::<MapNodeDataType>::new();

        descriptor.push( MapNodeDataType::PointSet );

        descriptor
    }

    fn calc_and_get_output(&self, input: &HashMap<String, MapNodeData>, context: CalcContext) -> Vec<MapNodeData> {
        let mut ps = HashSet::<(usize, usize)>::new();

        let rc = input["Rect"].rect();

        for x in rc.x1..rc.x2 {
            for y in rc.y1..rc.y2 {
                ps.insert( (x as usize, y as usize) );
            }
        }

        vec![ MapNodeData::PointSet(ps) ]
    }

}
