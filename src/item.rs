

use textwrap::wrap;


#[derive(Clone, Debug)]
pub enum ItemSize {
    Volume(f32),
    Bulky,
    AttachOnly
}

#[derive(Clone, Debug)]
pub enum LickResponse {
    FlavorText(String, usize),
    LongText(Vec<String>, usize),
    PoisonRefusal,
    HeatRefusal,
    NonBioRefusal
}


#[derive(Clone, Debug)]
pub struct InvItem {
    pub display_name: String,
    pub display_ch: char,
    pub flavor_text: String,
    pub color: (u8, u8, u8),
    pub can_stack: i32, // unique identifier for this item's stackability type
    pub stack: usize,
    pub size: ItemSize,
    pub lick_result: LickResponse
}


impl InvItem {
    pub fn get_inspect_text(&self, width: usize) -> Vec<String> {
        let mut out = Vec::new();

        let txt = &self.flavor_text;

        // resolve flavor text context here ???

        out.append( &mut wrap(txt, width).iter().map(|cow| cow.to_string() ).collect() );

        out
    }

}
