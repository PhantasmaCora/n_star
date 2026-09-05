

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

pub struct Inventory{
    pub inventory: Vec<InvItem>,
    pub inv_volume: (f32, f32),
    pub inv_bulky: (usize, usize)
}

impl Inventory {
    pub fn len(&self) -> usize {
        self.inventory.len()
    }

    pub fn can_add_item(&self, it: &InvItem) -> bool {
        match it.size {
            ItemSize::Volume(v) => {
                return self.inv_volume.0 >= self.inv_volume.1 + v;
            },
            ItemSize::Bulky => {
                return self.inv_bulky.0 > self.inv_bulky.1;
            },
            ItemSize::AttachOnly => {
                return false;
            }
        }
    }

    pub fn add_item(&mut self, it: InvItem) -> Result<(), InvItem> {
        if !self.can_add_item(&it) {
            return Err(it);
        }

        match it.size {
            ItemSize::Volume(v) => {
                if it.can_stack > 0 {
                    let mut found = false;

                    self.inv_volume.1 += v * (it.stack as f32);

                    for other in self.inventory.iter_mut() {
                        if other.can_stack == it.can_stack {
                            other.stack += it.stack;
                            found = true;
                        }
                    }
                    if !found {
                        self.inventory.push(it);
                    }
                } else {
                    self.inventory.push(it);
                    self.inv_volume.1 += v;
                }
            },
            ItemSize::Bulky => {
                self.inv_bulky.1 += 1;
                self.inventory.push(it);
            },
            ItemSize::AttachOnly => {
                // unreachable
            }
        }
        Ok(())
    }

    pub fn remove_item(&mut self, idx: usize) -> Option<InvItem> {
        if idx >= self.inventory.len() {
            return None;
        }
        let mut it = self.inventory.remove(idx);

        if it.stack > 1 {
            let mut new_item = it.clone();
            new_item.stack -= 1;
            it.stack = 1;
            self.inventory.insert(idx, new_item);
        }

        match it.size {
            ItemSize::Volume(v) => {
                self.inv_volume.1 -= v;
            },
            ItemSize::Bulky => {
                self.inv_bulky.1 -= 1;
            },
            ItemSize::AttachOnly => {
                // unreachable
            }
        }

        Some(it)
    }



}
