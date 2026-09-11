use std::cell::OnceCell;
use std::collections::{HashMap};
use std::fmt::Debug;
use std::rc::Rc;

use serde::Deserialize;

use rand::{prelude::*, rngs::ChaCha20Rng};



use crate::actor::attachment::{Attachment, SlotContent, SlotBorrow, is_compat};




pub struct AttachmentsComponent {
    pub hm: HashMap::<i64, Attachment>,
    pub root_id: i64,
    inner_rng: OnceCell<ChaCha20Rng>
}

impl AttachmentsComponent {
    pub fn new(mut root: Attachment) -> Self {
        root.id = Some(0);
        let mut hm = HashMap::new();
        hm.insert(0, root);
        Self {
            hm,
            root_id: 0,
            inner_rng: OnceCell::new()
        }
    }

    pub fn borrow_slot(&self, parent_id: i64, parent_slot: usize) -> SlotBorrow {
        let p = self.hm.get(&parent_id);
        if let Some(parent) = p {
            let ssize = parent.kind.provides_slots.len();
            if parent_slot >= ssize {
                return SlotBorrow::Empty;
            }
            let g = parent.slots.get(parent_slot);
            if let Some(sc) = g {
                match sc {
                    SlotContent::Attached(id) => {
                        return SlotBorrow::Attached( self.hm.get(id).unwrap() );
                    },
                    SlotContent::Bracing(id) => {
                        return SlotBorrow::Bracing( self.hm.get(id).unwrap() );
                    },
                    SlotContent::Empty => {
                        return SlotBorrow::Empty;
                    }
                }
            }
        }

        return SlotBorrow::Empty;
    }

    pub fn can_attach(&self, parent_id: i64, parent_slot: usize, child: &Attachment, has_tools: bool) -> bool {
        if child.kind.integrated && !has_tools {
            return false;
        }

        let p = self.hm.get(&parent_id);
        if let Some(parent) = p {
            let ssize = parent.kind.provides_slots.len();
            if parent_slot >= ssize {
                return false;
            }

            let s = parent.slots.get(parent_slot);
            if let Some(slot_content) = s {
                // check whether already full
                match slot_content {
                    SlotContent::Empty => {},
                    SlotContent::Attached(_) => {return false;}
                    SlotContent::Bracing(_) => {return false;}
                }
            }

            // check compatibility
            let slot_type = parent.kind.provides_slots.get(parent_slot).unwrap();

            let compat = is_compat( &child.kind.holder_kind, &slot_type.0 );

            if compat.0 && !compat.1 {
                return true;
            }
        }
        return false;
    }

    pub fn attach(&mut self, parent_id: i64, parent_slot: usize, mut child: Attachment, has_tools: bool) -> Result<(), Attachment> {
        if !self.can_attach(parent_id, parent_slot, &child, has_tools) {
            return Err(child);
        }

        self.inner_rng.get_or_init( || rand::make_rng::<ChaCha20Rng>() );

        if let None = child.id {
            child.id = Some( self.inner_rng.get_mut().unwrap().random() );
        }
        loop {
            if self.hm.contains_key( child.id.as_ref().unwrap() ) {
                child.id = Some( self.inner_rng.get_mut().unwrap().random() );
            } else {
                break;
            }
        }
        {
            let p = self.hm.get_mut(&parent_id);
            if let Some(parent) = p {
                parent.slots[parent_slot] = SlotContent::Attached( *child.id.as_ref().unwrap() );
                child.parent = Some( *parent.id.as_ref().unwrap() );
            }
        }

        self.hm.insert(*child.id.as_ref().unwrap(), child);

        return Ok(());
    }

    pub fn can_remove(&self, parent_id: i64, parent_slot: usize, has_tools: bool) -> bool {
        if parent_id == self.root_id {
            return false;
        }

        let p = self.hm.get(&parent_id);
        if let Some(parent) = p {
            let ssize = parent.kind.provides_slots.len();
            if parent_slot >= ssize {
                return false;
            }

            let s = parent.slots.get(parent_slot);
            if let Some(slot_content) = s {
                // check whether properly occupied
                match slot_content {
                    SlotContent::Attached(cid) => {
                        let c = self.hm.get(cid);
                        if let Some(child) = c {
                            let integ = child.kind.integrated;

                            if has_tools || !integ {
                                return true;
                            }
                        }
                    }
                    SlotContent::Empty => {return false;},
                    SlotContent::Bracing(_) => {return false;}
                }
            }

        }
        return false;
    }


}
