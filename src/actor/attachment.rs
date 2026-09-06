use std::collections::{HashMap};

use std::rc::Rc;


use crate::InvItem;
use crate::dataread::AttachmentTypeData;


pub enum SlotContent {
    Empty,
    Attached(Attachment),
    Bracing
}


pub struct AttachmentType {
    pub integrated: bool,
    pub name: String,
    pub display_name: String,
    pub holder_kind: String,
    pub weight: f32,
    pub features: Vec<Box<dyn AttachmentFeature>>,
    pub provides_slots: Vec<(String, String)>,
    pub item_proto: Option<InvItem>
}

impl AttachmentType {
    pub fn from_data(dat: AttachmentTypeData) -> Self {
        Self {
            integrated: dat.integrated,
            name: dat.name,
            display_name: dat.display_name,
            holder_kind: dat.holder_kind,
            weight: dat.weight,
            features: vec![],
            provides_slots: dat.provides_slots,
            item_proto: dat.item_proto
        }
    }
}




pub struct Attachment {
    pub kind: Rc<AttachmentType>,
    pub slots: Vec<SlotContent>
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


pub trait AttachmentFeature{}





pub fn make_test_att_types() -> HashMap<String, Rc<AttachmentType>> {
    let mut v = vec![];

    v.push(AttachmentType{
        integrated: true,
        name: "Shell2".to_string(),
        display_name: "RUNKO-127 Shell".to_string(),
        holder_kind: "".to_string(),
        weight: 96.0,
        features: vec![],
        provides_slots: vec![ ("Arm2".to_string(), "Left Arm".to_string()), ("Arm2".to_string(), "Right Arm".to_string()), ("Leg2".to_string(), "Left Leg".to_string()), ("Leg2".to_string(), "Right Leg".to_string()), ("Optics2".to_string(), "Optics".to_string()) ],
        item_proto: None
    });

    v.push(AttachmentType{
        integrated: true,
        name: "Arm2".to_string(),
        display_name: "KSVS-63 Arm typ-D".to_string(),
        holder_kind: "Arm2".to_string(),
        weight: 24.0,
        features: vec![],
        provides_slots: vec![ ("Weapon2".to_string(), "Weapon".to_string()) ],
        item_proto: None
    });

    v.push(AttachmentType{
        integrated: true,
        name: "Leg2".to_string(),
        display_name: "JLK-63 Leg typ-C".to_string(),
        holder_kind: "Leg2".to_string(),
        weight: 24.0,
        features: vec![],
        provides_slots: vec![],
        item_proto: None
    });

    v.push(AttachmentType{
        integrated: true,
        name: "Optics2".to_string(),
        display_name: "SILMA Optics typ-A".to_string(),
        holder_kind: "Optics".to_string(),
        weight: 64.0,
        features: vec![],
        provides_slots: vec![],
        item_proto: None
    });

    let mut out = HashMap::new();

    for kind in v.drain(..) {
        out.insert( kind.name.clone(), Rc::new(kind) );
    }

    out
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
