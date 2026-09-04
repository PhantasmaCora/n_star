


#[derive(Clone, Debug)]
pub enum ItemSize {
    Volume(f32),
    Bulky,
    AttachOnly
}


#[derive(Clone, Debug)]
pub struct InvItem {
    pub display_name: String,
    pub display_ch: char,
    pub color: (u8, u8, u8),
    pub can_stack: i32, // unique identifier for this item's stackability type
    pub stack: usize,
    pub size: ItemSize
}
