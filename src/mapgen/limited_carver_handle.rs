use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use ndarray::prelude::*;

use deterministic_default_hasher::DeterministicDefaultHasher;



pub trait LimitedCarverHandle {
    fn dim(&self) -> (usize, usize);

    fn inspect(&self, point: (usize, usize)) -> Option<(bool, bool, bool)>;

    fn carve(&mut self, point: (usize, usize)) -> bool;

    fn fill(&mut self, point: (usize, usize)) -> bool;

    fn push_batch(&mut self);

    fn discard_batch(&mut self);
}



pub struct MapBatchHandle<'a, T: Eq+Clone+Hash>{
    view: ArrayViewMut<'a, T, Ix2>,
    breaks: HashMap<T,T>,
    fills: HashMap<T,T>,
    solid: HashSet<T>,
    carve_batch: HashSet<(usize, usize), DeterministicDefaultHasher>,
    fill_batch: HashSet<(usize, usize), DeterministicDefaultHasher>
}

impl<'a, T: Eq+Clone+Hash> MapBatchHandle<'a, T> {
    pub fn new( av: ArrayViewMut<'a, T, Ix2>, breaks: HashMap<T,T>, fills: HashMap<T,T>, solid: HashSet<T> ) -> Self {
        MapBatchHandle{
            view: av,
            breaks,
            fills,
            solid,
            carve_batch: HashSet::<(usize, usize), DeterministicDefaultHasher>::with_hasher(DeterministicDefaultHasher),
            fill_batch: HashSet::<(usize, usize), DeterministicDefaultHasher>::with_hasher(DeterministicDefaultHasher)
        }
    }
}

impl<'a, T: Eq+Clone+Hash> LimitedCarverHandle for MapBatchHandle<'a, T> {
    fn dim(&self) -> (usize, usize) {
        self.view.dim()
    }

    fn inspect(&self, point: (usize, usize)) -> Option<(bool, bool, bool)> {
        let sz = self.view.dim();
        if point.0 >= sz.0 || point.1 >= sz.1 {
            return None;
        } else {
            let t = &self.view[[point.0, point.1]];
            return Some( (
                self.solid.contains( t ),
                self.breaks.get( t ).is_some(),
                self.fills.get( t ).is_some()
            ) );
        }
    }

    fn carve(&mut self, point: (usize, usize)) -> bool {
        if self.breaks.get( &self.view[[point.0, point.1]] ).is_some() {
            self.fill_batch.remove(&point);
            self.carve_batch.insert(point);
            return true;
        }
        false
    }

    fn fill(&mut self, point: (usize, usize)) -> bool {
        if self.fills.get( &self.view[[point.0, point.1]] ).is_some() {
            self.carve_batch.remove(&point);
            self.fill_batch.insert(point);
            return true;
        }
        false
    }

    fn push_batch(&mut self) {
        for p in self.carve_batch.drain() {
            let c = self.breaks.get( &self.view[[p.0, p.1]] ).unwrap();
            self.view[p] = c.clone();
        }
        for p in self.fill_batch.drain() {
            let f = self.fills.get( &self.view[[p.0, p.1]] ).unwrap();
            self.view[p] = f.clone();
        }
    }

    fn discard_batch(&mut self) {
        self.carve_batch.clear();
        self.fill_batch.clear();
    }
}
