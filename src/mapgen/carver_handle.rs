




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
