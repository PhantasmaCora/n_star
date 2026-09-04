use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use rand::prelude::*;
use rand::rngs::ChaCha20Rng;

use bracket_lib::prelude::Point;
use bracket_lib::pathfinding::field_of_view_set;

use crate::turn::{Command, TurnAttempt};
use crate::map::Map;
use crate::item::{InvItem, ItemSize};


pub mod attachment;
use attachment::{AttachmentsComponent};


pub struct ActorKind {
    pub name: String,
    pub class: char,
    pub color: (u8, u8, u8),
    pub breath_interest: i32, // debt increases by this factor at engine tick-up. expressed as a fixed fraction of 4096, describing the added portion rather than the total.
    pub max_stability: i32,
    pub sight_range: i32
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
    pub attachments: Option<AttachmentsComponent>,
    pub inventory: Vec<InvItem>,
    pub inv_volume: (f32, f32),
    pub inv_bulky: (usize, usize),
    pub overrides: HashMap<String, ActorOverrideTrait>, // unique traits
    pub bonus_breath: i32, // added on at end of turn or at engine tick-up
    pub fov: Option<HashSet<Point>>,
    pub memory: Option<HashSet<Point>>
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

    pub fn update_fov(&mut self, map: &Map) {
        let hs = field_of_view_set( Point{ x: self.position.0, y:self.position.1 }, self.kind.sight_range, map );
        if let Some(mem) = self.memory.take() {
            let nmem = mem.union( &hs ).map( |e| e.clone() ).collect();
            self.memory = Some(nmem);
        }
        self.fov = Some(hs);
    }

}


// organiation -- inventory system
impl Actor {
    pub fn can_add_item(&self, it: &InvItem) -> bool {
        match it.size {
            ItemSize::Volume(v) => {
                return self.inv_volume.0 > self.inv_volume.1 + v;
            },
            ItemSize::Bulky => {
                return self.inv_bulky.0 > self.inv_bulky.1;
            },
            ItemSize::AttachOnly => {
                return false;
            }
        }
    }

    pub fn add_item(&mut self, it: InvItem) -> Result<(), InvItem> {
        if !self.can_add_item(&it) {
            return Err(it);
        }

        match it.size {
            ItemSize::Volume(v) => {
                if it.can_stack > 0 {
                    let mut found = false;

                    self.inv_volume.1 += v * (it.stack as f32);

                    for other in self.inventory.iter_mut() {
                        if other.can_stack == it.can_stack {
                            other.stack += it.stack;
                            found = true;
                        }
                    }
                    if !found {
                        self.inventory.push(it);
                    }
                } else {
                    self.inventory.push(it);
                    self.inv_volume.1 += v;
                }
            },
            ItemSize::Bulky => {
                self.inv_bulky.1 += 1;
                self.inventory.push(it);
            },
            ItemSize::AttachOnly => {
                // unreachable
            }
        }
        Ok(())
    }

    pub fn remove_item(&mut self, idx: usize) -> Option<InvItem> {
        if idx >= self.inventory.len() {
            return None;
        }
        let mut it = self.inventory.remove(idx);

        if it.stack > 1 {
            let mut new_item = it.clone();
            new_item.stack -= 1;
            it.stack = 1;
            self.inventory.insert(idx, new_item);
        }

        match it.size {
            ItemSize::Volume(v) => {
                self.inv_volume.1 -= v;
            },
            ItemSize::Bulky => {
                self.inv_bulky.1 -= 1;
            },
            ItemSize::AttachOnly => {
                // unreachable
            }
        }

        Some(it)
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


pub struct ActionSelectionContext<'a> {
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
