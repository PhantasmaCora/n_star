use std::collections::HashMap;

use rand::prelude::*;
use rand::rngs::ChaCha20Rng;

use bracket_lib::prelude::{BaseMap, Algorithm2D};

use crate::actor::Actor;
use crate::map;

#[derive(PartialEq)]
pub enum Command {
    Wait(i32),
    MoveStep{x: i32, y: i32}
}

#[derive(PartialEq)]
pub enum TurnAttempt {
    Selected(Command),
    AwaitingInput{name: String},
    CallMeAgain
}

pub enum ActionResult {
    Succeeded(i32),
    TryAlternate(Box<dyn ActionResolver>),
    Failed
}


pub struct ActionResolutionContext<'a> {
    pub map: &'a mut map::Map,
    pub other_actors: &'a mut HashMap<String, Actor>,
    pub rng: &'a mut ChaCha20Rng

}

pub trait ActionResolver {
    fn resolve(&self, acting: &mut Actor, context: &mut ActionResolutionContext) -> ActionResult;
}

pub fn map_command(cmd: Command) -> Box<dyn ActionResolver> {
    match cmd {
        Command::Wait(breath) => { return Box::new(JustDepleteBreath{amount: breath}) },
        Command::MoveStep{x, y} => { return Box::new(MoveStep{x,y}) }
    }
}

pub struct JustDepleteBreath {
    amount: i32
}
impl ActionResolver for JustDepleteBreath {
    fn resolve(&self, acting: &mut Actor, context: &mut ActionResolutionContext) -> ActionResult {
        return ActionResult::Succeeded( self.amount );
    }
}

pub struct MoveStep {
    x: i32,
    y: i32
}
impl ActionResolver for MoveStep {
    fn resolve(&self, acting: &mut Actor, context: &mut ActionResolutionContext) -> ActionResult {
        let start_pos = acting.position;
        let end_pos = (acting.position.0 + self.x, acting.position.1 + self.y);

        if let Some( other_name ) = context.map.exclusive_occupancy.get( &end_pos ) {
            return ActionResult::TryAlternate( Box::new( MeleeAttack{target: other_name.clone()} ) );
        }

        let exits = context.map.get_available_exits( context.map.point2d_to_index( start_pos.into() ) );
        let (idxs, costs): (Vec<_>, Vec<_>) = exits.into_iter().unzip();

        if idxs.contains( &context.map.point2d_to_index( end_pos.into() ) ) {
            context.map.exclusive_occupancy.remove( &acting.position );
            acting.position = end_pos;
            context.map.exclusive_occupancy.insert(acting.position, acting.name.clone());

            let dsc = self.x.abs() + self.y.abs();
            let mut f: f32 = 1.0; // only accounts for ortho and diagonal steps
            if dsc >= 2 {
                f = 1.41421356;
            }

            // edit the 1024 to account for varying actor speeds later on!
            return ActionResult::Succeeded( (1024 as f32 * f) as i32 );
        }

        return ActionResult::Failed;
    }
}

// maybe this should work off an offset, like MoveStep, rather than a target declaration? would help if we get around to multitile entities
pub struct MeleeAttack {
    target: String
}
impl ActionResolver for MeleeAttack {
    fn resolve(&self, acting: &mut Actor, context: &mut ActionResolutionContext) -> ActionResult {
        if let Some(targeted) = context.other_actors.get_mut(&self.target) {
            let attacker_pos = acting.position;
            let target_pos = targeted.position;

            let exits = context.map.get_available_exits( context.map.point2d_to_index( attacker_pos.into() ) );
            let (idxs, costs): (Vec<_>, Vec<_>) = exits.into_iter().unzip();

            if idxs.contains( &context.map.point2d_to_index( target_pos.into() ) ) {
                if let Some(hc) = &mut targeted.health {
                    hc.take_damage( 1 );
                    return ActionResult::Succeeded( 1024 );
                }
            }
        }

        return ActionResult::Failed;
    }
}
