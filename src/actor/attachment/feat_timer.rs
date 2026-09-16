use serde::{Serialize, Deserialize};

use bracket_lib::prelude::*;

use crate::actor::Actor;
use crate::actor::attachment::{AttachmentFeature, AttachmentFeatureDescriptor};


#[derive(Debug, Deserialize, Serialize)]
pub struct HasTimer {
    pub TimerId: String,

}


#[typetag::serde]
impl AttachmentFeature for HasTimer {
    fn validate(&self, actor: &mut Actor) -> bool {
        true
    }

    fn apply(&self, actor: &mut Actor) {}

    fn remove(&self, actor: &mut Actor) {}

    fn get_descriptor(&self) -> Option<AttachmentFeatureDescriptor> {
        None
    }

    fn get_text(&self) -> Vec<String> {}
}
