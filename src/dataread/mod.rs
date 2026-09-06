use std::collections::HashMap;
use std::rc::Rc;

use std::path::PathBuf;
use std::fs::*;


use serde::{Deserialize};
use figment::{Figment, providers::{Format, Toml}};


mod attachmenttype;
pub use attachmenttype::AttachmentTypeData;

use crate::item::{InvItem};
use crate::AttachmentType;


fn no_attachments() -> Vec<AttachmentTypeData> {
    vec![]
}

fn no_items() -> Vec<InvItem> {
    vec![]
}

#[derive(Debug, Deserialize)]
pub struct AssetData {

    #[serde(default="no_attachments")]
    attachment: Vec<AttachmentTypeData>,

    #[serde(default="no_items")]
    item: Vec<InvItem>
}

impl AssetData {
    pub fn get_attachment_table(&mut self) -> HashMap<String, Rc<AttachmentType>> {
        let mut out = HashMap::new();

        for dat in self.attachment.drain(..) {
            let att = AttachmentType::from_data(dat);
            out.insert( att.name.clone(), Rc::new(att) );
        }

        out
    }
}



pub fn get_data(path: PathBuf) -> Result<AssetData, figment::Error> {
    let mut fig = Figment::new();

    let mut rdvec = vec![ read_dir(path).expect("invalid path") ];

    while !rdvec.is_empty() {
        let rditer = rdvec.pop().unwrap();

        for res_file in rditer {

            if let Ok(entry) = res_file {
                if entry.file_type().expect("invalid file").is_file() && entry.file_name().to_string_lossy().ends_with(".toml") {
                    fig = fig.admerge( Toml::file( entry.path() ) );
                } else if entry.file_type().expect("invalid file").is_dir() {
                    rdvec.push( read_dir( entry.path() ).expect("invalid recursive path") );
                }
            }
        }

    }



    return fig.extract();

}
