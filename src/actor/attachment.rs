use std::cell::OnceCell;
use std::collections::{HashMap};
use std::fmt::Debug;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use bracket_lib::prelude::*;

use crate::Actor;
use crate::InvItem;

pub mod component;
pub use component::AttachmentsComponent;

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
    pub fn from_type(t: Rc<AttachmentType>) -> Self {
        Self {
            slots: { let mut v = vec![]; for i in 0..t.provides_slots.len() {v.push(SlotContent::Empty);} v},
            kind: t,
            parent: None,
            brace: None,
            id: None
        }
    }
}



#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackedAttachment {
    pub kind: String,

    #[serde(default="empty_vec")]
    pub children: Vec<Option<PackedAttachment>>
}

fn empty_vec() -> Vec<Option<PackedAttachment>> {
    vec![]
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









pub fn make_test_att_comp(table: &HashMap<String, Rc<AttachmentType>>) -> AttachmentsComponent {
    let opt = Attachment::from_type( table.get("Optics").unwrap().clone() );
    let larm = Attachment::from_type( table.get("Arm2").unwrap().clone() );
    let rarm = Attachment::from_type( table.get("Arm2").unwrap().clone() );
    let lleg = Attachment::from_type( table.get("Leg2").unwrap().clone() );
    let rleg = Attachment::from_type( table.get("Leg2").unwrap().clone() );;

    let shell = Attachment::from_type( table.get("Shell2").unwrap().clone() );

    let mut attcom = AttachmentsComponent::new(shell);

    let _ = attcom.attach( 0, 0, larm, true );
    let _ = attcom.attach( 0, 1, rarm, true );
    let _ = attcom.attach( 0, 2, lleg, true );
    let _ = attcom.attach( 0, 3, rleg, true );
    let _ = attcom.attach( 0, 4, opt, true );

    return attcom;
}
