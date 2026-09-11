
use bracket_lib::prelude::*;

use crate::menu::{OverlayMenu, OverlayReturn, MenuContext};

use crate::actor::Actor;
use crate::turn::Command;
use crate::item::{ItemSize, LickResponse, ItemInspectContext};


enum Input {
    None,
    Close,
    DropItem,
    LickItem
}

enum Subview {
    None,
    LickItem
}


pub struct InspectInvItemMenu {
    key_in: Input,
    selected_slot: i32,
    subview: Subview
}

impl OverlayMenu for InspectInvItemMenu {
    fn handle_keypress(&mut self, vkc: VirtualKeyCode) {
        match vkc {
            VirtualKeyCode::Up => {
                if let Subview::None = self.subview {
                    self.selected_slot -= 1;
                }
            },
            VirtualKeyCode::Down => {
                if let Subview::None = self.subview {
                    self.selected_slot += 1;
                }
            },
            VirtualKeyCode::PageUp => {
                if let Subview::None = self.subview {
                    self.selected_slot -= 16;
                }
            },
            VirtualKeyCode::PageDown => {
                if let Subview::None = self.subview {
                    self.selected_slot += 16;
                }
            },

            VirtualKeyCode::D => {
                if let Subview::None = self.subview {
                    self.key_in = Input::DropItem;
                }
            },

            VirtualKeyCode::L => {
                if let Subview::None = self.subview {
                    self.key_in = Input::LickItem;
                }
            },

            VirtualKeyCode::Escape => {
                self.key_in = Input::Close;
            },

            VirtualKeyCode::Return => {
                self.key_in = Input::Close;
            },
            _ => {}
        }
    }

    fn update(&mut self, actor: &Actor, context: MenuContext) -> OverlayReturn {
        let n_items = actor.inventory.len();

        self.clamp_selection( n_items );

        match self.subview {
            Subview::None => {
                if let Input::Close = self.key_in {
                    return OverlayReturn::ChangeInterface( Box::new( crate::menu::InventoryMenu::from_slot(self.selected_slot) ) )
                } else if let Input::DropItem = self.key_in {
                    if n_items > 0 {
                        return OverlayReturn::AppendImmediate( Command::DropItem( self.selected_slot as usize ) );
                    }
                } else if let Input::LickItem = self.key_in {
                    self.subview = Subview::LickItem;
                    self.key_in = Input::None;
                }
            },
            Subview::LickItem => {
                if let Input::Close = self.key_in {
                    self.subview = Subview::None;
                    self.key_in = Input::None;
                }
            }
        }

        return OverlayReturn::NoAction;

    }

    fn draw_overlay(&self, bt: &mut BTerm, actor: &Actor, context: MenuContext) {
        let inf_deep = palette_color(&"inf_deep").unwrap();
        let inf_grey = palette_color(&"inf_grey").unwrap();
        let white: RGBA = WHITE.into();

        let mut batch = DrawBatch::new();

        let size = bt.get_char_size();

        let item = &actor.inventory.inventory.get(self.selected_slot as usize).expect("incorrect item index...");

        batch.draw_double_box(
            Rect{ x1: 10, x2: size.0 as i32 - 10, y1: 5, y2: size.1 as i32 - 2 },
            ColorPair{ fg: white, bg: inf_deep }
        );
        batch.fill_region(
            Rect{ x1: 11, x2: size.0 as i32 - 10, y1: 6, y2: size.1 as i32 - 2 },
            ColorPair{ fg: white, bg: inf_deep },
            ' '
        );

        batch.print_color( Point{ x: 11, y: 6}, &item.display_name,  ColorPair{bg: inf_deep, fg: white } );

        let mut brief_line = "".to_string();

        if item.can_stack > 0 {
            let num = format!("#[]x{:03}", item.stack);
            brief_line += &num;
        }

        match item.size {
            ItemSize::Volume(v) => {
                let mut vprint = format!("#[]{:>5}", v);

                if item.stack > 1 {
                    vprint = format!("#[]{:>5} ea", v);
                }

                brief_line += &vprint;
            },
            ItemSize::Bulky => {
                brief_line += "#[inf_bulk]Bulky#[]";
            },
            ItemSize::AttachOnly => {}
        }

        brief_line += "#[inf_grey] │#[] ^v to scroll";

        batch.printer( Point{ x: size.0 as i32 - 11, y: 6}, brief_line, TextAlign::Right, Some(inf_deep));

        batch.set( Point{x: size.0 as i32 / 2 - 1, y: 8}, ColorPair{fg: inf_grey, bg: inf_deep }, to_cp437( '┤' ) );
        batch.set( Point{x: size.0 as i32 / 2 + 1, y: 8}, ColorPair{fg: inf_grey, bg: inf_deep }, to_cp437( '├' ) );
        batch.set( Point{x: size.0 as i32 / 2, y: 7}, ColorPair{fg: inf_grey, bg: inf_deep }, to_cp437( '┴' ) );
        batch.set( Point{x: size.0 as i32 / 2, y: 9}, ColorPair{fg: inf_grey, bg: inf_deep }, to_cp437( '┬' ) );
        batch.set( Point{x: size.0 as i32 / 2, y: 8}, ColorPair{fg: item.color.into(), bg: inf_deep }, to_cp437( item.display_ch )); // draw item

        let ins_context = ItemInspectContext{ at: context.at };
        let lines = item.get_inspect_text( ins_context, (size.0 - 22) as usize );

        let mut y = 10;

        for l in lines {
            y += 1;
            batch.printer( Point{ x: 12, y}, l, TextAlign::Left, Some(inf_deep) );
        }

        batch.printer( Point{ x: 11, y: size.1 as i32 - 3}, "#[inf_gold]d#[] to drop. #[inf_gold]l#[] to lick.", TextAlign::Left, Some(inf_deep));

        let _ = batch.submit(5000);

        // handle subview drawing
        match self.subview {
            Subview::None => {},
            Subview::LickItem => {
                let mut batch = DrawBatch::new();

                let mut txt = vec![];

                let mut bw = 0;

                match &item.lick_result {
                    LickResponse::FlavorText(s, ln) => {
                        txt.push( s.clone() );
                        bw = (*ln) as i32 + 3;
                    },
                    LickResponse::LongText(v, ln) => {
                        txt.append( &mut v.iter().map( |s| s.clone() ).collect() );
                        bw = (*ln) as i32 + 3;
                    },
                    LickResponse::PoisonRefusal => {
                        txt.push( "#[inf_invl]It is obviously poisonous!".to_string() );
                        bw = 29;
                    },
                    LickResponse::HeatRefusal => {
                        txt.push( "#[inv_invl]It would burn your tongue.".to_string() );
                        bw = 29;
                    },
                    LickResponse::NonBioRefusal => {
                        txt.push( "#[inf_invl]Don't think this tastes like anything of interest.".to_string() );
                        bw = 53;
                    }
                }

                let bh = txt.len() as i32 + 2;

                batch.draw_double_box(
                    Rect{ x1: size.0 as i32 / 2 - bw / 2, x2: size.0 as i32 / 2 + bw / 2, y1: size.1 as i32 / 2 - bh / 2, y2: size.1 as i32 / 2 + bh / 2 },
                    ColorPair{ fg: white, bg: inf_deep }
                );
                batch.fill_region(
                    Rect{ x1: size.0 as i32 / 2 - bw / 2 + 1, x2: size.0 as i32 / 2 + bw / 2, y1: size.1 as i32 / 2 - bh / 2 + 1, y2: size.1 as i32 / 2 + bh / 2 },
                    ColorPair{ fg: white, bg: inf_deep },
                    ' '
                );

                batch.print_color( Point{x: size.0 as i32 / 2 - bw / 2 + 1, y: size.1 as i32 / 2 - bh / 2}, "╡ Licking item... ╞", ColorPair{bg: inf_deep, fg: white } );

                let mut y = size.1 as i32 / 2 - bh / 2 + 1;

                for line in txt.iter() {
                    batch.printer( Point{x: size.0 as i32 / 2, y}, line, TextAlign::Center, Some(inf_deep) );
                    y += 1;
                }

                let _ = batch.submit(12000);
            }
        }
    }
}


impl InspectInvItemMenu {
    pub fn new() -> Self {
        Self {
            key_in: Input::None,
            selected_slot: 0,
            subview: Subview::None
        }
    }

    pub fn from_slot(s: i32) -> Self {
        Self {
            key_in: Input::None,
            selected_slot: s,
            subview: Subview::None
        }
    }

    fn clamp_selection(&mut self, n_items: usize) {
        if n_items > 0 {
            while self.selected_slot < 0 {
                self.selected_slot += n_items as i32;
            }
            self.selected_slot %= n_items as i32;
        } else {
            self.selected_slot = 0;
        }
    }
}
