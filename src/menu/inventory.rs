
use bracket_lib::prelude::*;

use crate::menu::{OverlayMenu, OverlayReturn, MenuContext};

use crate::actor::Actor;
use crate::turn::Command;
use crate::item::ItemSize;


enum Input {
    None,
    Close,
    DropItem,
    InspectItem
}


pub struct InventoryMenu {
    key_in: Input,
    selected_slot: i32
}

impl OverlayMenu for InventoryMenu {
    fn handle_keypress(&mut self, vkc: VirtualKeyCode) {

        match vkc {
            VirtualKeyCode::Up => {
                self.selected_slot -= 1;
            },
            VirtualKeyCode::Down => {
                self.selected_slot += 1;
            },
            VirtualKeyCode::PageUp => {
                self.selected_slot -= 16;
            },
            VirtualKeyCode::PageDown => {
                self.selected_slot += 16;
            },

            VirtualKeyCode::D => {
                self.key_in = Input::DropItem;
            },

            VirtualKeyCode::I => {
                self.key_in = Input::InspectItem;
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
        // clamp selected slot
        let n_items = actor.inventory.len();

        self.clamp_selection( n_items );

        if let Input::DropItem = self.key_in {
            if n_items > 0 {
                return OverlayReturn::AppendImmediate( Command::DropItem( self.selected_slot as usize ) );
            }
        } else if let Input::InspectItem = self.key_in {
            return OverlayReturn::ChangeInterface( Box::new( crate::menu::InspectInvItemMenu::from_slot(self.selected_slot) ) )
        } else if let Input::Close = self.key_in {
            return OverlayReturn::SubmitCommands;
        }

        return OverlayReturn::NoAction;

    }

    fn draw_overlay(&self, bt: &mut BTerm, actor: &Actor, context: MenuContext) {
        let inf_deep = palette_color(&"inf_deep").unwrap();
        let inf_bulk = palette_color(&"inf_bulk").unwrap();

        let inv = &actor.inventory;

        let size = bt.get_char_size();
        let mut y = 1;

        let mut batch = DrawBatch::new();

        batch.draw_double_box(
            Rect{ x1: 5, x2: size.0 as i32 - 5, y1: 1, y2: size.1 as i32 - 2 },
            ColorPair{ fg: WHITE.into(), bg: inf_deep }
        );
        batch.fill_region(
            Rect{ x1: 6, x2: size.0 as i32 - 5, y1: 2, y2: size.1 as i32 - 2 },
            ColorPair{ fg: WHITE.into(), bg: inf_deep },
                          ' '
        );

        let volume_print = format!("╡Inventory » {:>5}/{:>5}v-", (inv.inv_volume.1 * 10.0).ceil() / 10.0, (inv.inv_volume.0 * 10.0).ceil() / 10.0);
        batch.print_color( Point{ x: 8, y: 1}, volume_print, ColorPair{bg: inf_deep, fg: WHITE.into() } );

        let bulk_print = format!("{:2}/{:2}B", inv.inv_bulky.1, inv.inv_bulky.0);
        batch.print_color( Point{ x: 34, y: 1}, bulk_print, ColorPair{bg: inf_deep, fg: inf_bulk } );

        batch.set( Point{x: 40, y: 1}, ColorPair{fg: WHITE.into(), bg: inf_deep }, to_cp437('╞') );


        for (idx, item) in inv.inventory.iter().enumerate() {
            y += 1;

            let num_id = format!("{:03}", idx);
            let mut num_colors = ColorPair{bg: inf_deep, fg: WHITE.into() };
            if idx == self.selected_slot as usize {
                num_colors = ColorPair{fg: inf_deep, bg: WHITE.into() };
            }
            batch.print_color( Point{ x:6, y}, num_id, num_colors );

            batch.set( Point{x: 10, y}, ColorPair{fg: item.color.into(), bg: inf_deep }, to_cp437( item.display_ch ) );

            let mut name = item.display_name.clone();
            if name.len() > 20 {
                name = item.display_name[0..16].to_string() + "...";
            }

            batch.print_color( Point{ x:12, y}, name, ColorPair{bg: inf_deep, fg: WHITE.into() } );

            if item.can_stack > 0 {
                let num = format!("x{:03}", item.stack);
                batch.print_color( Point{ x: 28, y}, num, ColorPair{bg: inf_deep, fg: WHITE.into() } );
            }

            match item.size {
                ItemSize::Volume(v) => {
                    let mut vprint = format!("{:>5}", v);

                    if item.stack > 1 {
                        vprint = format!("{:>5} ea", v);
                    }

                    batch.print_color( Point{ x: 33, y}, vprint, ColorPair{bg: inf_deep, fg: WHITE.into() } );
                },
                ItemSize::Bulky => {
                    batch.print_color( Point{ x: 39, y}, "B", ColorPair{bg: inf_deep, fg: inf_bulk } );
                },
                ItemSize::AttachOnly => {}
            }
        }

        batch.target(0);
        let _ = batch.submit(5000);
    }
}

impl InventoryMenu {
    pub fn new() -> Self {
        Self {
            key_in: Input::None,
            selected_slot: 0
        }
    }

    pub fn from_slot(s: i32) -> Self {
        Self {
            key_in: Input::None,
            selected_slot: s
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
