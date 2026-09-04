use std::collections::{HashMap};



pub struct AttachmentsComponent {
    pub slots: HashMap<String, Attachment>

}


pub struct Attachment {
    pub integrated: bool,
    pub held_by: String,
    //pub provides: Vec<dyn AttachmentFeature>
}
