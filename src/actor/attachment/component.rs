use std::cell::OnceCell;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::rc::Rc;

use serde::Deserialize;

use rand::{prelude::*, rngs::ChaCha20Rng};



use crate::actor::attachment::{Attachment, AttachmentType, SlotContent, SlotBorrow, is_compat, PackedAttachment};




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

    pub fn attach(&mut self, parent_id: i64, parent_slot: usize, mut child: Attachment, has_tools: bool) -> Result<i64, Attachment> {
        if !self.can_attach(parent_id, parent_slot, &child, has_tools) {
            return Err(child);
        }

        self.inner_rng.get_or_init( || rand::make_rng::<ChaCha20Rng>() );

        if let None = child.id {
            child.id = Some( self.inner_rng.get_mut().unwrap().random() );
        }
        loop {
            if self.hm.contains_key( child.id.as_ref().unwrap() ) || *child.id.as_ref().unwrap() == -1 {
                child.id = Some( self.inner_rng.get_mut().unwrap().random() );
            } else {
                break;
            }
        }

        //print!("{:?}\n", child.id);

        {
            let p = self.hm.get_mut(&parent_id);
            if let Some(parent) = p {
                parent.slots[parent_slot] = SlotContent::Attached( *child.id.as_ref().unwrap() );
                child.parent = Some( *parent.id.as_ref().unwrap() );
            }
        }

        let id = child.id.unwrap();

        self.hm.insert(*child.id.as_ref().unwrap(), child);

        return Ok( id );
    }

    pub fn can_swap_attach(&self, parent_id: i64, parent_slot: usize, child: &Attachment, has_tools: bool) -> bool {
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
                    SlotContent::Attached(att_id) => {
                        let att_opt = self.hm.get(att_id);
                        if let Some( att ) = att_opt {
                            if att.kind.integrated && !has_tools {
                                return false;
                            }
                        }
                    }
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

    pub fn swap_attach(&mut self, parent_id: i64, parent_slot: usize, mut child: Attachment, has_tools: bool) -> Result<(i64, Vec<PackedAttachment>), Attachment> {
        if !self.can_swap_attach(parent_id, parent_slot, &child, has_tools) {
            return Err(child);
        }

        self.inner_rng.get_or_init( || rand::make_rng::<ChaCha20Rng>() );

        if let None = child.id {
            child.id = Some( self.inner_rng.get_mut().unwrap().random() );
        }
        loop {
            if self.hm.contains_key( child.id.as_ref().unwrap() ) || *child.id.as_ref().unwrap() == -1 {
                child.id = Some( self.inner_rng.get_mut().unwrap().random() );
            } else {
                break;
            }
        }

        //print!("{:?}\n", child.id);

        let mut swapped_out = vec![];
        if let Some(v) = self.remove_and_pack( parent_id, parent_slot, has_tools ).ok() {
            swapped_out = v;
        }

        {
            let p = self.hm.get_mut(&parent_id);
            if let Some(parent) = p {
                parent.slots.insert( parent_slot, SlotContent::Attached( *child.id.as_ref().unwrap() ) );
                child.parent = Some( *parent.id.as_ref().unwrap() );
            }
        }

        let id = child.id.unwrap();

        self.hm.insert(*child.id.as_ref().unwrap(), child);

        return Ok( (id, swapped_out) );
    }

    pub fn attach_packed(&mut self, at: &HashMap<String, Rc<AttachmentType>>, parent_id: i64, parent_slot: usize, child: PackedAttachment, has_tools: bool) -> Result<Vec<PackedAttachment>, PackedAttachment> {
        let mut stack: Vec<(i64, usize, PackedAttachment)> = vec![];

        let backup = child.clone();

        stack.push( (parent_id, parent_slot, child) );

        let mut first = true;

        let mut old = vec![];

        while !stack.is_empty() {
            let (pid, ps, mut current) = stack.pop().unwrap();

            let mut children = vec![];
            children.append( &mut current.children );

            let k = at.get( &current.kind );

            let att = Attachment::from_type( k.unwrap().clone() );

            let mut good_id = -1;

            if first {
                let res = self.swap_attach( pid, ps, att, has_tools );
                if let Err(_) = res {
                    self.remove_and_pack( parent_id, parent_slot, has_tools );
                    return Err(backup);
                } else if let Ok( (id, oatt) ) = res {
                    good_id = id;
                    old = oatt;
                }
            } else {
                let res = self.attach( pid, ps, att, true );
                if let Err(_) = res {
                    self.remove_and_pack( parent_id, parent_slot, has_tools );
                    return Err(backup);
                }
            }

            for (idx, ch) in current.children.drain(..).enumerate().rev() {
                if let Some(patt) = ch {
                    stack.push( (good_id, idx, patt) );
                }
            }

            first = false;
        }

        return Ok(old);
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

    pub fn remove_and_pack(&mut self, parent_id: i64, parent_slot: usize, has_tools: bool) -> Result<Vec<PackedAttachment>, ()> {
        if !self.can_remove(parent_id, parent_slot, has_tools) {
            return Err(());
        }
        let mut pack_root = SlotContent::Empty;
        {
            let parent = self.hm.get_mut(&parent_id).unwrap();
            pack_root = parent.slots.remove( parent_slot );
            parent.slots.insert( parent_slot, SlotContent::Empty );
        }

        if let SlotContent::Attached(pack_root_id) = pack_root {
            if let Some(att) = self.hm.remove( &pack_root_id ) {
                let mut stack: Vec<(Vec<usize>, Attachment)> = vec![];
                let mut results = vec![];

                stack.push( (vec![], att) );
                while !stack.is_empty() {
                    let Some((mut addr, mut current)) = stack.pop() else {break;};

                    let slotcount = current.slots.len();

                    for (idx, s) in current.slots.drain(..).enumerate() {
                        match s {
                            SlotContent::Attached(cid) => {
                                let opt = self.hm.remove(&cid);
                                if let Some(catt) = opt {
                                    if catt.kind.integrated {
                                        let mut caddr = addr.clone();
                                        caddr.push( idx );
                                        stack.push( (caddr, catt) );
                                    } else {
                                        stack.push( (vec![], catt) );
                                    }
                                }
                            },
                            SlotContent::Bracing(cid) => {
                                let opt = self.hm.remove(&cid);
                                if let Some(catt) = opt {
                                    stack.push( (vec![], catt) );
                                }
                            },
                            SlotContent::Empty => {}
                        }
                    }

                    let packed = current.to_packed();

                    if addr.is_empty() {
                        results.push( packed );
                    } else {
                        let len = results.len();
                        let mut base = results.pop().unwrap();
                        let last = addr.pop().unwrap();

                        {
                            let c = addr.iter().fold( &mut base, | ca, idx | { ca.children.get_mut(*idx).unwrap().as_mut().unwrap() } );
                            c.children[last] = Some(packed);
                        }

                        results.push(base);
                    }
                }

                return Ok(results);
            }
        }

        return Err(());
    }

}
