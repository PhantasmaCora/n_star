
use bracket_lib::prelude::*;

use crate::menu::{OverlayMenu, OverlayReturn, MenuContext};

use crate::actor::Actor;
use crate::turn::Command;
use crate::map::NonExclusiveOccupant;
use crate::item::{InvItem, ItemSize};


enum Input {
    None,
    Close,
    GrabItem
}


pub struct GrabMenu {
    key_in: Input,
    selected_slot: i32
}

impl OverlayMenu for GrabMenu {
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

            VirtualKeyCode::Escape => {
                self.key_in = Input::Close;
            },

            VirtualKeyCode::Return => {
                self.key_in = Input::GrabItem;
            },
            _ => {}
        }
    }

    fn update(&mut self, actor: &Actor, context: MenuContext) -> OverlayReturn {
        let grabs = self.compile_grabables( actor, &context );

        if grabs.len() == 0 {
            return OverlayReturn::SubmitCommands;
        }

        // clamp selected slot
        self.clamp_selection( grabs.len() );

        if let Input::GrabItem = self.key_in {
            if !grabs.is_empty() {
                let g = grabs[self.selected_slot as usize];

                let item = &g.2;

                if actor.inventory.can_add_item(item) {
                    let cmd = Command::GrabItem{
                        x: g.0.0,
                        y: g.0.1,
                        idx: g.1
                    };
                    return OverlayReturn::AppendImmediate(cmd);
                }
            }
            self.key_in = Input::None;
        } else if let Input::Close = self.key_in {
            return OverlayReturn::SubmitCommands;
        }

        return OverlayReturn::NoAction;
    }

    fn draw_overlay(&self, bt: &mut BTerm, actor: &Actor, context: MenuContext) {
        let inf_deep = palette_color(&"inf_deep").unwrap();
        let inf_bulk = palette_color(&"inf_bulk").unwrap();
        let inf_invl = palette_color(&"inf_invl").unwrap();
        let white: RGBA = WHITE.into();

        let size = bt.get_char_size();
        let mut y = 1;

        let grabs = self.compile_grabables( actor, &context );

        let mut batch = DrawBatch::new();

        batch.draw_double_box(
            Rect{ x1: size.0 as i32 - 32, x2: size.0 as i32 - 2, y1: 1, y2: size.1 as i32 - 2 },
            ColorPair{ fg: white, bg: inf_deep }
        );
        batch.fill_region(
            Rect{ x1: size.0 as i32 - 31, x2: size.0 as i32 - 2, y1: 2, y2: size.1 as i32 - 2 },
            ColorPair{ fg: white, bg: inf_deep },
            ' '
        );

        let inv = &actor.inventory;

        let volume_print = format!("╡{:>5}/{:>5}v-", (inv.inv_volume.1 * 10.0).ceil() / 10.0, (inv.inv_volume.0 * 10.0).ceil() / 10.0);
        batch.print_color( Point{ x: size.0 as i32 - 30, y: 1}, volume_print, ColorPair{bg: inf_deep, fg: white } );

        let bulk_print = format!("{:2}/{:2}B", inv.inv_bulky.1, inv.inv_bulky.0);
        batch.print_color( Point{ x: size.0 as i32 - 16, y: 1}, bulk_print, ColorPair{bg: inf_deep, fg: inf_bulk } );

        batch.set( Point{x: size.0 as i32 - 10, y: 1}, ColorPair{fg: white, bg: inf_deep }, to_cp437('╞') );

        let mut can_grab_current = true;

        for (idx, grab) in grabs.iter().enumerate() {
            y += 1;

            if y > size.1 as i32 - 5 {
                break;
            }

            let item = &grab.2;

            let can_take = inv.can_add_item(item);

            let num_id = format!("{:02}", idx);
            let mut bright = RGBA::from(WHITE);

            if !can_take {
                bright = inf_invl;
            }

            let mut num_colors = ColorPair{bg: inf_deep, fg: bright.into() };
            if idx == self.selected_slot as usize {
                can_grab_current = can_take;

                num_colors = ColorPair{fg: inf_deep, bg: bright.into() };
            }
            batch.print_color( Point{ x: size.0 as i32 - 31, y}, num_id, num_colors );

            batch.set( Point{x: size.0 as i32 - 28, y}, ColorPair{fg: item.color.into(), bg: inf_deep }, to_cp437( item.display_ch ) );

            let mut name = item.display_name.clone();
            if name.len() > 15 {
                name = item.display_name[0..12].to_string() + "...";
            }

            batch.print_color( Point{ x: size.0 as i32 - 26, y}, name, ColorPair{bg: inf_deep, fg: white } );

            if item.can_stack > 0 {
                let num = format!("x{:03}", item.stack);
                batch.print_color( Point{ x: size.0 as i32 - 13, y}, num, ColorPair{bg: inf_deep, fg: white } );
            }

            match item.size {
                ItemSize::Volume(v) => {
                    let mut vprint = format!("{:>5}", v);

                    if item.stack > 1 {
                        vprint = format!("{:>5} ea", v);
                    }

                    batch.print_color( Point{ x: size.0 as i32 - 9, y}, vprint, ColorPair{bg: inf_deep, fg: white } );
                },
                ItemSize::Bulky => {
                    batch.print_color( Point{ x: size.0 as i32 - 9, y}, "B", ColorPair{bg: inf_deep, fg: inf_bulk } );
                },
                ItemSize::AttachOnly => {}
            }
        }

        if can_grab_current {
            batch.printer( Point{ x: size.0 as i32 - 31, y: size.1 as i32 - 3}, "#[inf_gold]Enter#[] to grab selected.", TextAlign::Left, Some(inf_deep));
        } else {
            batch.print_color( Point{ x: size.0 as i32 - 31, y: size.1 as i32 - 3}, "Not enough room!", ColorPair{bg: inf_deep, fg: inf_invl } );
        }

        let _ = batch.submit(5000);
    }
}


impl GrabMenu {
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

    fn compile_grabables<'a>(&self, actor: &Actor, context: &'a MenuContext) -> Vec<((i32, i32), usize, &'a InvItem)> {
        let mut out = Vec::new();

        for offs in vec![ (0,0), (-1,-1), (0,-1), (1,-1), (-1,0), (1,0), (-1,1), (0,1), (1,1) ].iter() {
            let opos = (actor.position.0 + offs.0, actor.position.1 + offs.1);
            let opt = context.map.list_neos( opos );

            if let Some(neo_vec) = opt {
                for (idx, neo) in neo_vec.iter().enumerate() {
                    if let NonExclusiveOccupant::Item(it) = neo {
                        out.push( (*offs, idx, it) );
                    }
                }
            }
        }

        return out;
    }
}
