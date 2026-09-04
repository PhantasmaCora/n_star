
use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::BitOr;

use deterministic_default_hasher::DeterministicDefaultHasher;

use ndarray::Array2;

use rand::{Rng, RngExt};
use rand::rngs::ChaCha20Rng;

use bracket_lib::pathfinding::a_star_search;

use crate::mapgen::{CarverHandle, LightweightMap};


pub struct ConnCompLabeler {
    pub eight_connectivity: bool,
}

impl ConnCompLabeler {
    pub fn eight() -> ConnCompLabeler {
        return Self{
            eight_connectivity: true
        }
    }

    pub fn four() -> ConnCompLabeler {
        return Self{
            eight_connectivity: false
        }
    }

    pub fn label(&self, handle: &impl CarverHandle) -> Vec<HashSet<(usize, usize)>> {

        // list of connected components
        let mut ccs = HashMap::<usize, HashSet<(usize, usize)>, DeterministicDefaultHasher>::with_hasher(DeterministicDefaultHasher);

        // maps array labels to set indices
        let mut label_map = HashMap::<usize, usize>::new();

        // working array of labels, matches size of input
        let mut label_arr = Array2::<usize>::default(handle.dim());

        // working dimensions
        let (sx,sy) = handle.dim();
        let sx = sx as i32;
        let sy = sy as i32;

        let mut current_label = 0;

        let mut scans = vec![(-1, 0), (0, -1)];
        if self.eight_connectivity {
            scans.push((-1,-1));
            scans.push((1,-1));
        }

        for x in 0i32..sx {
            for y in 0i32..sy {
                if handle.inspect((x as usize, y as usize)).unwrap() {
                    continue;
                }

                let mut found = HashSet::<usize>::new(); // labels found on prior cells
                for s in scans.iter() {
                    if x+s.0 < 0 || x+s.0 >= sx || y+s.1 < 0 || y+s.1 >= sy {
                        continue; // out of bounds check
                    }
                    let adj = label_arr[[(x+s.0) as usize, (y+s.1) as usize]];
                    if adj > 0 {
                        found.insert(adj);
                    }
                }

                let fv: Vec<usize> = found.drain().collect();

                if fv.len() == 0 {
                    // new label added!
                    current_label += 1;
                    label_arr[[x as usize, y as usize]] = current_label;

                    label_map.insert(current_label, current_label);

                    let mut hs = HashSet::new();
                    hs.insert((x as usize, y as usize));
                    ccs.insert(current_label, hs);

                } else if fv.len() == 1 {
                    // add point to existing label

                    let label = fv[0];
                    label_arr[[x as usize, y as usize]] = label;
                    let idx = label_map[&label];
                    ccs.get_mut(&idx).unwrap().insert((x as usize, y as usize));

                } else {
                    //print!("{:?}\n", fv);

                    // we need to do a merge
                    let dominant_label = *fv.iter().max().unwrap();
                    let didx = label_map[&dominant_label];
                    let mut ds = ccs.remove(&didx).unwrap();

                    label_arr[[x as usize,y as usize]] = dominant_label;

                    for lab in fv.iter() {
                        if !( label_map[lab] == label_map[&dominant_label] ) {
                            //print!("merge {} to {}\n", lab, dominant_label);
                            //print!("indices: {}, {}\n", label_map[&lab], didx);

                            let idx = label_map[&lab];

                            for (k,v) in label_map.iter_mut() {
                                if *v == idx {
                                    *v = didx;
                                }
                            }

                            let set = ccs.remove(&idx).unwrap();
                            ds = ds.bitor(&set);
                        }/* else if *lab != dominant_label {
                        print!("skipping merge {} to {}\n", lab, dominant_label);
                    }*/
                    }

                    ccs.insert(didx, ds);
                }
            }
        }

        let cc: Vec<HashSet<(usize, usize)>> = ccs.into_values().collect();
        cc
    }

}

pub struct ConnCompTunneler {
    pub labeler: ConnCompLabeler
}

impl ConnCompTunneler {
    pub fn cull_small(&self, handle: &mut impl CarverHandle, min_size: usize) {
        let mut ccs = self.labeler.label(handle);

        for cc in ccs.iter() {
            if cc.len() < min_size {
                // fill in the small gap
                for p in cc.iter() {
                    handle.fill(*p);
                }
                handle.push_batch();
            }
        }
    }

    pub fn connect_all_astar(&self, handle: &mut impl CarverHandle, weights: (f32, f32, f32), rng: &mut ChaCha20Rng ) {
        let mut ccs = self.labeler.label(handle);

        // necessary for determinism; otherwise the hasher randomization will impact results
        ccs.sort_by_key( |cc| cc.len() );

        let mut max_iter = 256;

        let sz = handle.dim();

        while ccs.len() > 1 && max_iter > 0 {
            max_iter -= 1;

            let mut comp_a = ccs.pop().unwrap();

            let size = ccs.len();
            let mut comp_b = ccs.remove( rng.random_range(0..size) );

            // average out for centers.
            let sum_a = comp_a.iter().fold( (0,0), |a: (usize, usize), e: &(usize, usize)|{(a.0 + e.0, a.1 + e.1)} );
            let center_a = ( (sum_a.0 as f32 / comp_a.len() as f32) as usize, (sum_a.1 as f32 / comp_a.len() as f32) as usize );

            let sum_b = comp_b.iter().fold( (0,0), |a: (usize, usize), e: &(usize, usize)|{(a.0 + e.0, a.1 + e.1)} );
            let center_b = ( (sum_b.0 as f32 / comp_b.len() as f32) as usize, (sum_b.1 as f32 / comp_b.len() as f32) as usize );

            // find nearest point in set
            let mut collect_a: Vec<(usize, usize)> = comp_a.drain().collect();
            collect_a.sort_by_key( |pt| pt.0.abs_diff( center_b.0 ) + pt.1.abs_diff( center_b.1 ) );
            let target_a = collect_a[ 0 ];

            let mut collect_b: Vec<(usize, usize)> = comp_b.drain().collect();
            collect_b.sort_by_key( |pt| pt.0.abs_diff( center_a.0 ) + pt.1.abs_diff( center_a.1 ) );
            let target_b = collect_b[ 0 ];

            let mut work_arr = Array2::<f32>::default( sz );

            for x in 0..sz.0 {
                for y in 0..sz.1 {
                    if comp_a.contains(&(x,y)) || comp_b.contains(&(x,y)) {
                        continue; // keep the zero cost for space in either component
                    }

                    if !handle.inspect((x,y)).unwrap() {
                        work_arr[[x,y]] = weights.0; // low cost for empty space
                    } else {
                        let mut adj = false;
                        for dx in -1..=1 {
                            for dy in -1..=1 {
                                if (dx == 0 && dy == 0) || (dx < 0 && x == 0) || (dy < 0 && y == 0) { continue; }
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;

                                if nx >= sz.0 || ny >= sz.1 { continue; }

                                let ne = handle.inspect((nx, ny)).unwrap();
                                if ne {
                                    adj = true;
                                    break;
                                }
                            }
                            if adj {break;}
                        }
                        if adj {
                            work_arr[[x,y]] = weights.2; // high cost for room border walls
                        } else {
                            work_arr[[x,y]] = weights.1; // medium cost for walls
                        }
                    }
                }
            }

            let lwm = LightweightMap{ array: &work_arr };
            let astar = a_star_search( lwm.point_to_index(target_a), lwm.point_to_index(target_b), &lwm );

            let path = astar.steps;

            //let path = candidates.remove(min_idx as usize);
            for idx in path.iter() {
                let point = lwm.index_to_point(*idx);
                handle.carve(point);
            }

            handle.push_batch();

            // re-label since things got cut open
            ccs = self.labeler.label(handle);
            ccs.sort_by_key( |cc| cc.len() );
        }
    }

    // "Good Enough" implementation. needs a serious rework
    /*pub fn connect_all_dogleg<F: Fn(A)->bool, C: Fn(A)->f32>(&self, mut arr: ArrayViewMut<A, Ix2>, predicate: F, cost: C, opening: A, rng: &mut ChaCha20Rng ) {
     *        let mut ccs = self.labeler.label(arr.borrow(), &predicate);
     *
     *        // necessary for determinism; otherwise the hasher randomization will impact results
     *        ccs.sort_by_key( |cc| cc.len() );
     *
     *        let mut max_iter = 256;
     *
     *        while ccs.len() > 1 && max_iter > 0 {
     *            max_iter -= 1;
     *
     *            let mut comp_a = ccs.pop().unwrap();
     *
     *            let size = ccs.len();
     *            let comp_b = ccs.remove( rng.random_range(0..size) );
     *
     *            // average out for centers. change later???
     *            let sum_a = comp_a.iter().fold( (0,0), |a: (usize, usize), e: &(usize, usize)|{(a.0 + e.0, a.1 + e.1)} );
     *            let center_a = ( (sum_a.0 as f32 / comp_a.len() as f32) as usize, (sum_a.1 as f32 / comp_a.len() as f32) as usize );
     *
     *            let sum_b = comp_b.iter().fold( (0,0), |a: (usize, usize), e: &(usize, usize)|{(a.0 + e.0, a.1 + e.1)} );
     *            let center_b = ( (sum_b.0 as f32 / comp_b.len() as f32) as usize, (sum_b.1 as f32 / comp_b.len() as f32) as usize );
     *
     *            // distance to cover
     *            let diff = ( center_a.0 as i32 - center_b.0 as i32, center_a.1 as i32 - center_b.1 as i32 );
     *
     *            let min_x = usize::min(center_a.0, center_b.0);
     *            let min_y = usize::min(center_a.1, center_b.1);
     *            let diff = (i32::abs(diff.0) as usize, i32::abs(diff.1) as usize);
     *
     *            let mut candidates = Vec::<HashSet<[usize; 2]>>::new();
     *
     *            for _i in 0..32 {
     *                let mut hs = HashSet::<[usize; 2]>::new();
     *
     *                let choice = rng.random_bool(0.5);
     *
     *                if (choice || diff.1 == 0) && diff.0 > 0 {
     *                    let dl = rng.random_range( 0..diff.0 );
     *                    for x in 0..diff.0 {
     *                        let mut y = 0;
     *                        if x > dl {
     *                            y = diff.1
}
hs.insert( [ min_x + x, min_y + y ] );
}
for y in 0..diff.1 {
    hs.insert( [ min_x + dl, min_y + y ] );
}
} else {
    let dl = rng.random_range( 0..diff.1 );
    for y in 0..diff.1 {
        let mut x = 0;
        if y > dl {
            x = diff.0
}
hs.insert( [ min_x + x, min_y + y ] );
}
for x in 0..diff.0 {
    hs.insert( [ min_x + x, min_y + dl ] );
}
}

candidates.push(hs);
}

let mut min_cost = 1000000.0;
let mut min_idx: i32 = -1;
for (idx, item) in candidates.iter().enumerate() {
    let mut nc = 0.0;
    for point in item.iter() {
        let a = arr[*point];
        nc += cost(a);
}
if nc < min_cost {
    min_cost = nc;
    min_idx = idx as i32;
}
}

let path = candidates.remove(min_idx as usize);
for point in path.iter() {
    arr[*point] = opening;
}

// re-label since things got cut open
ccs = self.labeler.label(arr.borrow(), &predicate);
ccs.sort_by_key( |cc| cc.len() );
}
}*/
}

