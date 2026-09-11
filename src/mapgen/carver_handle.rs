use std::collections::HashSet;
use std::vec::Drain;

use ndarray::prelude::*;

use deterministic_default_hasher::DeterministicDefaultHasher;

use crate::mapgen::{CarverHandle};



pub struct WideChainCarverHandle<'a> {
    pub chain: Box<&'a mut dyn CarverHandle>,
    pub radius: (i32, i32)
}

impl<'a> CarverHandle for WideChainCarverHandle<'a> {
    fn dim(&self) -> (usize, usize) {
        self.chain.dim()
    }

    fn inspect(&self, point: (usize, usize)) -> Option<bool> {
        self.chain.inspect(point)
    }

    fn carve(&mut self, point: (usize, usize)) {
        for dx in -self.radius.0..=self.radius.0 {
            for dy in -self.radius.1..=self.radius.1{
                let dpt = ( point.0 as i32 + dx, point.1 as i32 + dy );
                self.chain.carve(( dpt.0 as usize, dpt.1 as usize ));
            }
        }
    }

    fn fill(&mut self, point: (usize, usize)) {
        for dx in -self.radius.0..=self.radius.0 {
            for dy in -self.radius.1..=self.radius.1{
                let dpt = ( point.0 as i32 + dx, point.1 as i32 + dy );
                self.chain.fill(( dpt.0 as usize, dpt.1 as usize ));
            }
        }
    }

    fn push_batch(&mut self) {
        self.chain.push_batch();
    }

    fn discard_batch(&mut self) {
        self.chain.discard_batch();
    }
}






pub struct IndependentPointSetCarverHandle<'a> {
    base_view: ArrayViewMut<'a, bool, Ix2>,
    do_batches: bool,
    fill: bool,
    results: Vec<HashSet<(usize, usize)>>
}

impl<'a> IndependentPointSetCarverHandle<'a> {
    pub fn from_view(base_view: ArrayViewMut<'a, bool, Ix2>, do_batches: bool, fill: bool) -> Self {
        Self{
            base_view,
            do_batches,
            fill,
            results: vec![HashSet::new()]
        }
    }

    pub fn drain_results<'b>(&'b mut self) -> Drain<'b, HashSet<(usize, usize)>> {
        self.results.drain(..)
    }
}


impl<'a> CarverHandle for IndependentPointSetCarverHandle<'a> {
    fn dim(&self) -> (usize, usize) {
        self.base_view.dim()
    }

    fn inspect(&self, point: (usize, usize)) -> Option<bool> {
        let sz = self.base_view.dim();
        if point.0 >= sz.0 || point.1 >= sz.1 {
            return None;
        } else {
            return Some( self.base_view[[point.0, point.1]] );
        }
    }

    fn carve(&mut self, point: (usize, usize)) {
        let l = self.results.len() - 1;
        if self.fill{
            self.results.get_mut( l ).unwrap().remove(&point);
        } else {
            self.results.get_mut( l ).unwrap().insert(point);
        }
    }

    fn fill(&mut self, point: (usize, usize)) {
        let l = self.results.len() - 1;
        if self.fill{
            self.results.get_mut( l ).unwrap().insert(point);
        } else {
            self.results.get_mut( l ).unwrap().remove(&point);
        }
    }

    fn push_batch(&mut self) {
        if self.do_batches {
            self.results.push(HashSet::new());
        }
    }

    fn discard_batch(&mut self) {
        if self.do_batches && self.results.len() > 1 {
            self.results.pop();
        }
    }
}



pub struct BoolViewBatchHandle<'a>{
    view: ArrayViewMut<'a, bool, Ix2>,
    carve_batch: HashSet<(usize, usize), DeterministicDefaultHasher>,
    fill_batch: HashSet<(usize, usize), DeterministicDefaultHasher>
}

impl<'a> BoolViewBatchHandle<'a> {
    pub fn new( av: ArrayViewMut<'a, bool, Ix2> ) -> Self {
        BoolViewBatchHandle{
            view: av,
            carve_batch: HashSet::<(usize, usize), DeterministicDefaultHasher>::with_hasher(DeterministicDefaultHasher),
            fill_batch: HashSet::<(usize, usize), DeterministicDefaultHasher>::with_hasher(DeterministicDefaultHasher)
        }
    }
}

impl<'a> CarverHandle for BoolViewBatchHandle<'a> {
    fn dim(&self) -> (usize, usize) {
        self.view.dim()
    }

    fn inspect(&self, point: (usize, usize)) -> Option<bool> {
        let sz = self.view.dim();
        if point.0 >= sz.0 || point.1 >= sz.1 {
            return None;
        } else {
            return Some( self.view[[point.0, point.1]] );
        }
    }

    fn carve(&mut self, point: (usize, usize)) {
        self.fill_batch.remove(&point);
        self.carve_batch.insert(point);
    }

    fn fill(&mut self, point: (usize, usize)) {
        self.carve_batch.remove(&point);
        self.fill_batch.insert(point);
    }

    fn push_batch(&mut self) {
        for p in self.carve_batch.drain() {
            self.view[p] = false;
        }
        for p in self.fill_batch.drain() {
            self.view[p] = true;
        }
    }

    fn discard_batch(&mut self) {
        self.carve_batch.clear();
        self.fill_batch.clear();
    }
}




pub struct EqViewBatchHandle<'a, T: Eq+Clone>{
    view: ArrayViewMut<'a, T, Ix2>,
    matches: Vec<T>,
    set_carve: T,
    set_fill: T,
    carve_batch: HashSet<(usize, usize), DeterministicDefaultHasher>,
    fill_batch: HashSet<(usize, usize), DeterministicDefaultHasher>
}

impl<'a, T: Eq+Clone> EqViewBatchHandle<'a, T> {
    pub fn new( av: ArrayViewMut<'a, T, Ix2>, matches: Vec<T>, set_carve: T, set_fill: T ) -> Self {
        EqViewBatchHandle{
            view: av,
            matches,
            set_carve,
            set_fill,
            carve_batch: HashSet::<(usize, usize), DeterministicDefaultHasher>::with_hasher(DeterministicDefaultHasher),
            fill_batch: HashSet::<(usize, usize), DeterministicDefaultHasher>::with_hasher(DeterministicDefaultHasher)
        }
    }
}

impl<'a, T: Eq+Clone> CarverHandle for EqViewBatchHandle<'a, T> {
    fn dim(&self) -> (usize, usize) {
        self.view.dim()
    }

    fn inspect(&self, point: (usize, usize)) -> Option<bool> {
        let sz = self.view.dim();
        if point.0 >= sz.0 || point.1 >= sz.1 {
            return None;
        } else {
            return Some( self.matches.contains( &self.view[[point.0, point.1]] ) );
        }
    }

    fn carve(&mut self, point: (usize, usize)) {
        if point.0 >= self.dim().0 || point.1 >= self.dim().1 {
            return;
        }
        self.fill_batch.remove(&point);
        self.carve_batch.insert(point);
    }

    fn fill(&mut self, point: (usize, usize)) {
        if point.0 >= self.dim().0 || point.1 >= self.dim().1 {
            return;
        }
        self.carve_batch.remove(&point);
        self.fill_batch.insert(point);
    }

    fn push_batch(&mut self) {
        for p in self.carve_batch.drain() {
            self.view[p] = self.set_carve.clone();
        }
        for p in self.fill_batch.drain() {
            self.view[p] = self.set_fill.clone();
        }
    }

    fn discard_batch(&mut self) {
        self.carve_batch.clear();
        self.fill_batch.clear();
    }
}

