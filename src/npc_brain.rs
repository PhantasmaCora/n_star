use rand::RngExt;

use bracket_lib::prelude::{BaseMap, Algorithm2D};
use bracket_lib::pathfinding::{a_star_search, NavigationPath};

use crate::actor::{Actor, ActionSelectionContext, Brain};
use crate::turn::{TurnAttempt, Command};


pub struct StandardMonsterBrain {
    pub courage: f32,
    pub wander_distance: f32,
    pub packmates: Vec<String>,
    pub pack_center_distance: f32,
    pub pack_loyalty_frac: f32,
    pub priority: MonsterPriority,
    pub sleep_time: i32
}

impl StandardMonsterBrain {
    fn get_wander_point(&self, actor: &Actor, context: &mut ActionSelectionContext) -> Result<(i32,i32),()> {
        // search nearby points for valid destinations
        let mut candidates = Vec::<(i32, i32)>::new();

        let size = self.wander_distance as i32 / 2;

        for x in -size..=size {
            for y in -size..=size {
                if x == 0 && y == 0 {
                    continue;
                }

                let dest = ( actor.position.0 + x * 2, actor.position.1 + y * 2 );

                if context.map.in_bounds(dest.into()) && context.map.get_pathing_distance( context.map.point2d_to_index(actor.position.into()), context.map.point2d_to_index(dest.into()) ) <= self.wander_distance && context.map.is_passable( context.map.point2d_to_index(dest.into()) ) {
                    candidates.push(dest);
                }
            }
        }

        if candidates.is_empty() {
            return Err(());
        }

        // select a random candidate
        let idx = context.rng.random_range( ..candidates.len() );
        let selected = candidates[idx];

        return Ok(selected);
    }

    fn get_pack_center(&mut self, actor: &Actor, context: &mut ActionSelectionContext) -> (i32, i32) {
        // find the pack center
        let mut sum = actor.position;
        let mut divisor = 1.0f32;
        let mut to_rm = Vec::<String>::new();

        for (idx, pm_name) in self.packmates.iter().enumerate() {
            let pm_search = context.other_actors.get( pm_name );

            if let Some(pm) = pm_search {
                sum = ( sum.0 + pm.position.0, sum.1 + pm.position.1 );
                divisor += 1.0;
            } else {
                to_rm.push(pm_name.to_string());
            }
        }
        // cleanup phase - delete refs to missing packmates
        self.packmates.retain( | name: &String | -> bool { !to_rm.contains(name) } );

        // divide out the average
        ( (sum.0 as f32 / divisor) as i32, (sum.1 as f32 / divisor) as i32 )
    }

}

impl Brain for StandardMonsterBrain {
    fn get_action(&mut self, actor: &Actor, mut context: &mut ActionSelectionContext) -> TurnAttempt {
        match self.priority {
            MonsterPriority::Sleep => {
                if self.sleep_time <= 0 {
                    let wp = self.get_wander_point(actor, context).expect("failed to get wander point");

                    self.priority = MonsterPriority::Wander{x: wp.0, y: wp.1};
                    self.sleep_time = 16;
                    return TurnAttempt::CallMeAgain;
                }
                self.sleep_time -= 1;
                return TurnAttempt::Selected( Command::Wait(1024) );
            },
            MonsterPriority::Wander{mut x, mut y} => {
                self.sleep_time -= 1;

                // check for interrupts first

                // if enemy sighted, evaluate

                // else if its been too long, try something new
                if self.sleep_time <= 0 {
                    let wp = self.get_wander_point(actor, context).expect("failed to get wander point");

                    self.sleep_time = 16;

                    x = wp.0;
                    y = wp.1;
                    self.priority = MonsterPriority::Wander{x: wp.0, y: wp.1};
                }

                // else if too far from pack center, try get back
                {
                    let center = self.get_pack_center(actor, context);
                    let dst = context.map.get_pathing_distance(
                        context.map.point2d_to_index( actor.position.into() ),
                        context.map.point2d_to_index( center.into() )
                    );

                    if dst > self.pack_center_distance {
                        let chance = (dst - self.pack_center_distance) / ( self.pack_loyalty_frac * self.pack_center_distance );

                        if context.rng.random::<f32>() < chance {
                            self.priority = MonsterPriority::PackReunite;
                            return TurnAttempt::CallMeAgain;
                        }
                    }
                }

                // get a navigation path
                let mut path = a_star_search( context.map.point2d_to_index(actor.position.into()), context.map.point2d_to_index((x,y).into()), context.map );

                let mut max_misses = 8;

                while !path.success || (path.steps.len() <= 1 ) {
                    max_misses -= 1;
                    if max_misses <= 0 { break; }
                    let potential = self.get_wander_point(actor, &mut context);
                    if let Ok(point) = potential {
                        path = a_star_search( context.map.point2d_to_index(actor.position.into()), context.map.point2d_to_index(point.into()), context.map );
                    } else {
                        max_misses -= 1;
                        if max_misses <= 0 { break; }
                    }
                }

                if max_misses > 0 && path.steps.len() > 1 {
                    let fin = context.map.index_to_point2d( path.destination );
                    self.priority = MonsterPriority::Wander{x: fin.x, y:fin.y};

                    let step_dest = context.map.index_to_point2d( path.steps[1] );

                    return TurnAttempt::Selected( Command::MoveStep{ x: step_dest.x - actor.position.0, y: step_dest.y - actor.position.1} );
                } else {
                    self.sleep_time = 8;
                    self.priority = MonsterPriority::Sleep;
                    return TurnAttempt::Selected(Command::Wait(1024));
                }
            },
            MonsterPriority::PackReunite => {
                // check for interrupts first, later on!

                let center = self.get_pack_center(actor, context);

                // if close enough to pack center, consider wandering again
                if context.map.get_pathing_distance( context.map.point2d_to_index(center.into()), context.map.point2d_to_index(actor.position.into()) ) < 0.5 * self.pack_center_distance {

                    let wp = self.get_wander_point(actor, context).expect("failed to get wander point");

                    self.priority = MonsterPriority::Wander{x: wp.0, y: wp.1};
                    self.sleep_time = 16;
                    return TurnAttempt::CallMeAgain;
                }

                // pathfind.
                let path = a_star_search( context.map.point2d_to_index(actor.position.into()), context.map.point2d_to_index(center.into()), context.map );

                if path.steps.len() > 1 {
                    let step_dest = context.map.index_to_point2d( path.steps[1] );

                    return TurnAttempt::Selected( Command::MoveStep{ x: step_dest.x - actor.position.0, y: step_dest.y - actor.position.1} );
                } else {
                    return TurnAttempt::Selected( Command::Wait(1024) );
                }
            }
        }
    }
}

pub enum MonsterPriority {
    Sleep,
    Wander{ x: i32, y: i32 },
    PackReunite
}
