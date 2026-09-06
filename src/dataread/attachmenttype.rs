

use serde::{Serialize, Deserialize};
use figment::{Figment, providers::{Format, Toml, Serialized}};

use crate::item::InvItem;

#[derive(Debug, Deserialize)]
pub struct AttachmentTypeData {
    #[serde(default = "falsehood")]
    pub integrated: bool,

    pub name: String,

    pub display_name: String,

    pub holder_kind: String,

    pub weight: f32,

    //pub features: Vec<AttachmentFeatureData>,

    pub provides_slots: Vec<(String, String)>, // kind, label

    pub item_proto: Option<InvItem>
}


fn falsehood() -> bool {
    false
}
