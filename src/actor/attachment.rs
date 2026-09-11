use std::cell::{RefCell, Ref, RefMut, OnceCell};
use std::collections::{HashMap};
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use serde::Deserialize;

use rand::{prelude::*, rngs::ChaCha20Rng};

use bracket_lib::prelude::*;

use crate::Actor;
use crate::InvItem;


pub mod feat_attack;



#[derive(Debug, Deserialize)]
pub struct AttachmentType {
    pub integrated: bool,
    pub name: String,
    pub display_name: String,
    pub describe_types: (String, String),
    pub holder_kind: String,
    pub weight: f32,
    pub features: Vec<Box<dyn AttachmentFeature>>,
    pub provides_slots: Vec<(String, String)>,
    pub item_proto: Option<InvItem>
}

impl AttachmentType {
    pub fn get_descriptors(&self) -> Vec<AttachmentFeatureDescriptor> {
        let mut out = vec![];

        if self.integrated {
            out.push( AttachmentFeatureDescriptor::SingleChar{
                ch: 'I',
                fg: palette_color(&"inf_deep").unwrap(),
                bg: palette_color(&"inf_invl").unwrap()
            } );
        }

        // apply descriptors from features here too

        out
    }

    pub fn get_text_describe(&self) -> Vec<String> {
        let mut baseline = format!("{}, fits {} slot. {:.2} weight.", self.describe_types.0, self.describe_types.1, self.weight);

        if self.describe_types.1 == "" || self.describe_types.0 == self.describe_types.1 {
            baseline = format!("{} slot. {:.2} weight.", self.describe_types.0, self.weight);
        }

        let mut out = vec![baseline];

        if self.provides_slots.len() > 0 {
            let mut line = "Hardpoints: [".to_string();

            let mut i = self.provides_slots.iter();
            let mut v = i.next();
            while !v.is_none() {
                line += &v.unwrap().1;
                v = i.next();
                if !v.is_none() {
                    line += ", "
                }
            }
            line += "]";
            out.push(line);
        }

        for feat in self.features.iter() {
            out.append(&mut feat.get_text());
        }

        out
    }
}


#[derive(Clone, Debug)]
pub enum SlotContent {
    Empty,
    Attached(i64),
    Bracing(i64)
}

pub enum SlotBorrow<'a> {
    Empty,
    Attached(&'a Attachment),
    Bracing(&'a Attachment)
}

#[derive(Clone, Debug)]
pub struct Attachment {
    pub kind: Rc<AttachmentType>,
    pub slots: Vec<SlotContent>,
    pub parent: Option<i64>,
    pub brace: Option<i64>,
    pub id: Option<i64>
}

impl Attachment {
    pub fn new_from_type(t: Rc<AttachmentType>) -> Self {
        Self {
            slots: { let mut v = vec![]; for i in 0..t.provides_slots.len() {v.push(SlotContent::Empty);} v},
            kind: t,
            parent: None,
            brace: None,
            id: None
        }
    }
}

// return.0 = can_attach return.1 = needs_bracing
pub fn is_compat(att_fits: &str, slot_is: &str) -> (bool, bool) {
    if att_fits == slot_is {
        return (true, false);
    }

    if att_fits == "Weapon" && (slot_is == "Weapon1" || slot_is == "Weapon2" || slot_is == "Weapon3") {
        return (true, false);
    }

    if slot_is == "Weapon2" && att_fits == "HeavyWeapon" {
        return (true, true);
    } else if slot_is == "Weapon3" && att_fits == "HeavyWeapon" {
        return (true, true);
    }

    return (false, false);
}





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

    pub fn can_attach(&self, parent_id: i64, parent_slot: usize, child: &Attachment) -> bool {
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

    pub fn attach(&mut self, parent_id: i64, parent_slot: usize, mut child: Attachment) {
        if !self.can_attach(parent_id, parent_slot, &child) {
            return;
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
            if let Some(mut parent) = p {
                parent.slots[parent_slot] = SlotContent::Attached( *child.id.as_ref().unwrap() );
                child.parent = Some( *parent.id.as_ref().unwrap() );
            }
        }

        self.hm.insert(*child.id.as_ref().unwrap(), child);

    }
}







pub enum AttachmentFeatureDescriptor {
    SingleChar{ch: char, fg: RGBA, bg: RGBA},
    PrinterString(String, i32)
}

#[typetag::serde(tag = "type")]
pub trait AttachmentFeature: Debug {
    fn validate(&self, actor: &mut Actor) -> bool;

    fn apply(&self, actor: &mut Actor);

    fn remove(&self, actor: &mut Actor);

    fn get_descriptor(&self) -> AttachmentFeatureDescriptor;

    fn get_text(&self) -> Vec<String>;
}



/*pub fn make_test_att_comp(table: &HashMap<String, Rc<AttachmentType>>) -> AttachmentsComponent {
    let opt = Attachment::from_type( table.get("Optics").unwrap().clone() );
    let larm = Attachment::from_type( table.get("Arm2").unwrap().clone() );
    let rarm = Attachment::from_type( table.get("Arm2").unwrap().clone() );
    let lleg = Attachment::from_type( table.get("Leg2").unwrap().clone() );
    let rleg = Attachment::from_type( table.get("Leg2").unwrap().clone() );;

    let shell = Attachment{kind: table.get("Shell2").unwrap().clone(), slots: vec![
        SlotContent::Attached(larm), SlotContent::Attached(rarm), SlotContent::Attached(lleg), SlotContent::Attached(rleg), SlotContent::Attached(opt)
    ]};

    return AttachmentsComponent {
        root: shell
    };
}*/
