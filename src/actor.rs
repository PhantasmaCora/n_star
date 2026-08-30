use std::collections::HashMap;
use std::rc::Rc;

use rand::prelude::*;
use rand::rngs::ChaCha20Rng;

use crate::turn::{Command, TurnAttempt};
use crate::map::Map;

pub struct ActorKind {
    pub name: String,
    pub class: char,
    pub color: (u8, u8, u8),
    pub breath_interest: i32, // debt increases by this factor at engine tick-up. expressed as a fixed fraction of 4096, describing the added portion rather than the total.
    pub max_stability: i32
}

pub struct ActorOverrideTrait {}

pub struct ActorRegister {
    pub name: String,
    pub breath: i32 // action points
}



pub struct Actor {
    pub is_player: bool,
    pub kind: Rc<ActorKind>,
    pub brain: Option<Box<dyn Brain>>,
    pub name: String,
    pub position: (i32, i32),
    pub health: Option<HealthComponent>,
    pub overrides: HashMap<String, ActorOverrideTrait>, // unique traits
    pub bonus_breath: i32, // added on at end of turn or at engine tick-up
}

// organization -- gameplay functions
impl Actor {
    pub fn get_action(&mut self, context: &mut ActionSelectionContext) -> TurnAttempt {
        // get action command from brain (return AwaitingInput if applicable)
        let br = self.brain.take();
        let mut br = br.unwrap();
        let ta = br.get_action(self, context);
        self.brain = Some(br);

        ta
    }
}

// organization -- utilities
impl Actor {
    pub fn generate_register(&self) -> ActorRegister {
        ActorRegister { name: self.name.clone(), breath: 0 }
    }
}

// organization -- rendering related methods
impl Actor {
    pub fn get_draw_info(&self) -> ((u8, u8, u8), char) {
        // account for overrides later on!
        let k = &self.kind;
        ( k.color, k.class )
    }
}


pub struct HealthComponent {
    pub is_alive: bool,
    pub stability: i32, // "micro" HP
    pub wounds: i32, // "macro" HP
    pub max_stability: i32,
    pub max_wounds: i32
}

impl HealthComponent {
    pub fn take_damage(&mut self, amount: i32) {
        self.stability -= amount;
        if self.stability <= 0 {
            self.wounds -= 1;
            self.stability = self.max_stability;
        }
        if self.wounds <= 0 {
            self.is_alive = false;
        }
    }

}


pub struct ActionSelectionContext <'a> {
    pub player_orders: &'a mut Option::<Command>,
    pub map: &'a Map,
    pub other_actors: &'a HashMap<String, Actor>,
    pub rng: &'a mut ChaCha20Rng
}

pub trait Brain {
    fn get_action(&mut self, acting: &Actor, context: &mut ActionSelectionContext) -> TurnAttempt;
}

pub struct PlayerControlBrain {}

impl Brain for PlayerControlBrain {
    fn get_action(&mut self, acting: &Actor, context: &mut ActionSelectionContext) -> TurnAttempt {
        if let Some(cmd) = context.player_orders.take() {
            return TurnAttempt::Selected(cmd);
        } else {
            return TurnAttempt::AwaitingInput{ name: acting.name.clone() };
        }

    }

}
