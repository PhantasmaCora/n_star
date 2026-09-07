use std::borrow::Borrow;
use std::collections::{HashMap};
use std::fmt::Debug;
use std::rc::Rc;

use serde::Deserialize;

use bracket_lib::prelude::*;

use crate::Actor;
use crate::InvItem;


pub mod feat_attack;




#[derive(Clone, Debug)]
pub enum SlotContent {
    Empty,
    Attached(Attachment),
    Bracing
}

pub enum SlotBorrow<'a> {
    Empty,
    Attached(&'a Attachment),
    Bracing
}

impl SlotContent {
    pub fn get_ref<'a>(&'a self) -> SlotBorrow<'a> {
        match self {
            SlotContent::Empty => {SlotBorrow::Empty},
            SlotContent::Attached(att) => {SlotBorrow::Attached(&att)},
            SlotContent::Bracing => {SlotBorrow::Bracing}
        }
    }

}


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
        let mut baseline = format!("{}, fits {} slot.", self.describe_types.0, self.describe_types.1);

        if self.describe_types.1 == "" || self.describe_types.0 == self.describe_types.1 {
            baseline = format!("{} slot.", self.describe_types.0);
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
            out.push(line);
        }

        for feat in self.features.iter() {
            out.append(&mut feat.get_text());
        }

        out
    }
}



#[derive(Clone, Debug)]
pub struct Attachment {
    pub kind: Rc<AttachmentType>,
    pub slots: Vec<SlotContent>
}

impl Attachment {
    pub fn as_item(self) -> InvItem {
        let mut clon = self.kind.item_proto.clone().unwrap();
        clon.attaches_as = Some(self);

        // apply any other distinguishing traits

        clon
    }
}


pub struct AttachmentsComponent {
    pub root: Attachment
}

impl AttachmentsComponent {
    pub fn get_att(&self, addr: Vec<usize>) -> Option<&Attachment> {
        let mut current = &self.root;
        for idx in addr.iter() {
            let get = current.slots.get(*idx);
            if let Some(attopt) = get {
                if let SlotContent::Attached(att) = attopt {
                    current = att;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        return Some(current);
    }

    pub fn get_att_mut(&mut self, addr: Vec<usize>) -> Option<&mut Attachment> {
        let mut current = &mut self.root;
        for idx in addr.iter() {
            let get = current.slots.get_mut(*idx);
            if let Some(attopt) = get {
                if let SlotContent::Attached(att) = attopt {
                    current = att;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        return Some(current);
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



pub fn make_test_att_comp(table: &HashMap<String, Rc<AttachmentType>>) -> AttachmentsComponent {
    let opt = Attachment{kind: table.get("Optics").unwrap().clone(), slots: vec![]};
    let larm = Attachment{kind: table.get("Arm2").unwrap().clone(), slots: vec![SlotContent::Empty]};
    let rarm = Attachment{kind: table.get("Arm2").unwrap().clone(), slots: vec![SlotContent::Empty]};
    let lleg = Attachment{kind: table.get("Leg2").unwrap().clone(), slots: vec![]};
    let rleg = Attachment{kind: table.get("Leg2").unwrap().clone(), slots: vec![]};

    let shell = Attachment{kind: table.get("Shell2").unwrap().clone(), slots: vec![
        SlotContent::Attached(larm), SlotContent::Attached(rarm), SlotContent::Attached(lleg), SlotContent::Attached(rleg), SlotContent::Attached(opt)
    ]};

    return AttachmentsComponent {
        root: shell
    };
}
