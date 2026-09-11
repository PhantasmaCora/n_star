

use serde::{Serialize, Deserialize};

use bracket_lib::prelude::*;

use crate::actor::Actor;
use crate::actor::attachment::{AttachmentFeature, AttachmentFeatureDescriptor};


#[derive(Debug, Deserialize, Serialize)]
pub struct ProvidesMeleeAttack {
    pub pen_rating: i32,
    pub damage_die: String,
    pub technique_rating: i32
}

#[typetag::serde]
impl AttachmentFeature for ProvidesMeleeAttack {
    fn validate(&self, actor: &mut Actor) -> bool {
        true
    }

    fn apply(&self, actor: &mut Actor) {}

    fn remove(&self, actor: &mut Actor) {}

    fn get_descriptor(&self) -> AttachmentFeatureDescriptor {
        AttachmentFeatureDescriptor::SingleChar{ch: 'M', fg: palette_color(&"inf_attc").unwrap(), bg: palette_color(&"inf_deep").unwrap()}
    }

    fn get_text(&self) -> Vec<String> {
        vec![
            format!("Melee ♠{}, dmg {}", &self.pen_rating, &self.damage_die),
            format!(" > Technique Rating {} ({})", &self.technique_rating, desc_sr(self.technique_rating) )
        ]
    }
}

pub fn desc_sr(sr: i32) -> String {
    if sr > 72 {
        "Very Fast".to_string()
    } else if sr > 64 {
        "Fast".to_string()
    } else if sr > 48 {
        "Quick".to_string()
    } else if sr > 32 {
        "Average".to_string()
    } else if sr > 16 {
        "Slow".to_string()
    } else {
        "Very Slow".to_string()
    }
}


#[derive(Debug, Default, Deserialize, Serialize)]
pub enum BurstType {
    #[default]
    None,
    Scatter(usize),
    Rapid(usize)
}


#[derive(Debug, Deserialize, Serialize)]
pub struct ProvidesRangedAttack {
    #[serde(default)]
    pub burst: BurstType,

    pub spread: f32,
    pub pen_rating: i32,
    pub damage_die: String
}

#[typetag::serde]
impl AttachmentFeature for ProvidesRangedAttack {
    fn validate(&self, actor: &mut Actor) -> bool {
        true
    }

    fn apply(&self, actor: &mut Actor) {}

    fn remove(&self, actor: &mut Actor) {}

    fn get_descriptor(&self) -> AttachmentFeatureDescriptor {
        AttachmentFeatureDescriptor::SingleChar{ch: 'R', fg: palette_color(&"inf_attc").unwrap(), bg: palette_color(&"inf_deep").unwrap()}
    }

    fn get_text(&self) -> Vec<String> {
        let mut out = vec![];

        let mut txt = format!("Ranged ♠{}, dmg {}", &self.pen_rating, &self.damage_die);
        if let BurstType::Scatter(n) = self.burst {
            txt = format!("Ranged {}x ♠{}, dmg {}", n, &self.pen_rating, &self.damage_die);
        }
        out.push(txt);

        if let BurstType::Rapid(n) = self.burst {
            out.push( format!(" > {}-round burst", n) );
        }

        if self.spread > 0.05 {
            out.push( format!( " > Across {:.1} deg spread", &self.spread.to_degrees() ) );
        }

        out
    }
}
