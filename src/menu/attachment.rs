
use bracket_lib::prelude::*;

use crate::menu::{OverlayMenu, OverlayReturn, MenuContext};

use crate::actor::Actor;
use crate::actor::attachment::{SlotBorrow, AttachmentFeatureDescriptor};

enum Input {
    None,
    Close
}

pub struct AttachmentOverviewMenu {
    selected_slot: i32,
    key_in: Input
}

impl OverlayMenu for AttachmentOverviewMenu {
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
                self.key_in = Input::Close;
            },
            _ => {}
        }
    }

    fn update(&mut self, actor: &Actor, context: MenuContext) -> OverlayReturn {

        let attachments_opt = actor.attachments.as_ref();

        if let Some(attachments) = attachments_opt {

            let mut stack = Vec::<SlotBorrow>::new();
            stack.push( SlotBorrow::Attached( attachments.hm.get(&attachments.root_id).unwrap() ) );

            let mut n_att = 0;

            while !stack.is_empty() {
                let current = stack.pop().unwrap();

                if let SlotBorrow::Attached(att) = current {
                    for sidx in (0..att.slots.len() ).rev() {
                        stack.push( attachments.borrow_slot( att.id.unwrap(), sidx ) );
                    }
                }

                n_att += 1;
            }

            // clamp selected_slot to the relevant number of slots
            self.clamp_selection( n_att );
        }

        if let Input::Close = self.key_in {
            return OverlayReturn::SubmitCommands;
        }

        return OverlayReturn::NoAction;
    }

    fn draw_overlay(&self, bt: &mut BTerm, actor: &Actor, context: MenuContext) {
        if actor.attachments.is_none() {
            return;
        }

        let size = bt.get_char_size();

        let inf_deep = palette_color(&"inf_deep").unwrap();
        let inf_invl = palette_color(&"inf_invl").unwrap();
        let white: RGBA = WHITE.into();

        let attachments = actor.attachments.as_ref().unwrap();

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

        let title = "╡Attachments╞".to_string();
        batch.print_color( Point{ x: 8, y: 1}, title, ColorPair{bg: inf_deep, fg: WHITE.into() } );

        let mut lines = Vec::<(String, String, i32, Vec<AttachmentFeatureDescriptor>)>::new();

        let mut stack = Vec::<(String, SlotBorrow, i32)>::new();

        let rootstr = "Core".to_string();

        stack.push( (rootstr, SlotBorrow::Attached( &attachments.hm.get(&attachments.root_id).unwrap() ), 6) );

        while !stack.is_empty() {
            let current = stack.pop().unwrap();

            if let SlotBorrow::Attached(att) = current.1 {
                for sidx in (0..att.slots.len() ).rev() {
                    let label = att.kind.provides_slots[sidx].1.clone();
                    stack.push( (label, attachments.borrow_slot( att.id.unwrap(), sidx ), current.2 + 1) );
                }
            }

            let mut occupant = "".to_string();
            let mut features = Vec::new();

            match current.1 {
                SlotBorrow::Attached(att) => {
                    occupant = att.kind.display_name.clone();
                    features = att.kind.get_descriptors();
                },
                SlotBorrow::Empty => {
                    occupant = "#[inf_dgry](None)#[]".to_string();
                },
                SlotBorrow::Bracing(att) => {
                    occupant = format!("#[inf_dgry](Bracing {})#[]", att.kind.display_name);
                }
            }

            lines.push( ( current.0, occupant, current.2, features ) );
        }

        for (idx, l) in lines.iter().enumerate() {

            let mut start_colors = ColorPair{bg: inf_deep, fg: white };
            if idx == self.selected_slot as usize {
                start_colors = ColorPair{fg: inf_deep, bg: white };
            }

            let mut start = format!("└ {}:", l.0);
            if idx == 0 {
                start = format!("☼ {}:", l.0);
            }

            let offs = l.2 + start.len() as i32 - 1;

            batch.print_color( Point{ x: l.2, y: idx as i32 + 2}, start, start_colors );

            let name = format!("#[]{}", l.1 );

            batch.printer( Point{ x: offs, y: idx as i32 + 2}, name, TextAlign::Left, Some(inf_deep) );

            let mut x = size.0 as i32 - 7;

            for afd in l.3.iter().rev() {
                match afd {
                    AttachmentFeatureDescriptor::SingleChar{ch, fg, bg} => {
                        batch.set( Point{x, y: idx as i32 + 2}, ColorPair{fg: *fg, bg: *bg}, to_cp437(*ch) );
                        x -= 1;
                    },
                    AttachmentFeatureDescriptor::PrinterString(text, width) => {
                        batch.printer( Point{x, y: idx as i32 + 2}, text, TextAlign::Right, Some(inf_deep) );
                        x -= width;
                    }
                }

            }
        }

        let _ = batch.submit(5000);

    }
}

impl AttachmentOverviewMenu {
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
