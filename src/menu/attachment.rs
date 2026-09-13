
use bracket_lib::prelude::*;

use crate::menu::{OverlayMenu, OverlayReturn, MenuContext};

use crate::actor::Actor;
use crate::actor::attachment::{SlotBorrow, AttachmentFeatureDescriptor, component::AttachmentsComponent, is_compat};
use crate::item::Inventory;
use crate::map::NonExclusiveOccupant;
use crate::turn::{AttachmentAcquireSource, Command};

enum Input {
    None,
    Close,
    CloseEsc,
    Attach,
    Detach
}

enum SubSelectMode {
    None,
    Attach{ selected: i32 }
}

pub struct AttachmentOverviewMenu {
    selected_slot: i32,
    key_in: Input,
    mode: SubSelectMode,
    acquire: Vec<(AttachmentAcquireSource, bool)>
}

impl OverlayMenu for AttachmentOverviewMenu {
    fn handle_keypress(&mut self, vkc: VirtualKeyCode) {
        match vkc {
            VirtualKeyCode::Up => {
                if let SubSelectMode::Attach{selected: s} = self.mode {
                    self.mode = SubSelectMode::Attach{selected: s-1};
                } else {
                    self.selected_slot -= 1;
                }
            },
            VirtualKeyCode::Down => {
                if let SubSelectMode::Attach{selected: s} = self.mode {
                    self.mode = SubSelectMode::Attach{selected: s+1};
                } else {
                    self.selected_slot += 1;
                }
            },
            VirtualKeyCode::PageUp => {
                if let SubSelectMode::Attach{selected: s} = self.mode {
                    self.mode = SubSelectMode::Attach{selected: s-16};
                } else {
                    self.selected_slot -= 16;
                }
            },
            VirtualKeyCode::PageDown => {
                if let SubSelectMode::Attach{selected: s} = self.mode {
                    self.mode = SubSelectMode::Attach{selected: s+16};
                } else {
                    self.selected_slot += 16;
                }
            },

            VirtualKeyCode::S => {
                self.key_in = Input::Attach;
            },

            VirtualKeyCode::D => {
                self.key_in = Input::Detach;
            },

            VirtualKeyCode::Escape => {
                self.key_in = Input::CloseEsc;
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

            let mut stack = Vec::<(i64, usize, SlotBorrow)>::new();
            stack.push(
                (-1, 0, SlotBorrow::Attached( attachments.hm.get(&attachments.root_id).unwrap() ) )
            );

            let mut n_att = 0;
            let mut ids = Vec::<(i64, usize)>::new();

            while !stack.is_empty() {
                let current = stack.pop().unwrap();

                if let SlotBorrow::Attached(att) = current.2 {
                    for sidx in (0..att.kind.provides_slots.len() ).rev() {
                        stack.push( ( att.id.unwrap(), sidx, attachments.borrow_slot( att.id.unwrap(), sidx ) ) );
                    }
                }
                ids.push( (current.0, current.1) );
                n_att += 1;
            }

            //print!("{:?}\n", ids);

            // clamp selected_slot to the relevant number of slots
            self.clamp_selection( n_att );

            if let Input::Attach = self.key_in {
                if let SubSelectMode::None = self.mode {
                    if self.selected_slot != 0 && ids[self.selected_slot as usize].0 != -1 {
                        let st = attachments.hm.get( &ids[self.selected_slot as usize].0 ).unwrap();

                        let slot_type = st.kind.provides_slots[ids[self.selected_slot as usize].1].0.clone();

                        self.get_relevant_attachments( slot_type, &actor.inventory, &actor.position, &context );

                        if self.acquire.len() > 0 {
                            self.mode = SubSelectMode::Attach{selected:0};
                        }
                    }
                }
                self.key_in = Input::None;
            } else if let Input::Detach = self.key_in {
                if let SubSelectMode::None = self.mode {
                    return OverlayReturn::AppendImmediate(
                        Command::DetachAction{ parent: ids[self.selected_slot as usize].0, slot: ids[self.selected_slot as usize].1 }
                    );

                } else if let SubSelectMode::Attach{selected:_} = self.mode {
                    self.mode = SubSelectMode::None;
                    self.acquire = vec![];
                }
                self.key_in = Input::None;
            } else if let SubSelectMode::Attach{selected} = self.mode {
                if let Input::Close = self.key_in {
                    let mut v = vec![];
                    v.append(&mut self.acquire);

                    return OverlayReturn::AppendImmediate( Command::AttachAction{ parent: ids[self.selected_slot as usize].0, slot: ids[self.selected_slot as usize].1, source: v.remove(selected as usize).0 } );
                } else if let Input::CloseEsc = self.key_in {
                    return OverlayReturn::SubmitCommands;
                }
            }

        }

        if let Input::Close | Input::CloseEsc = self.key_in {
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
                for sidx in (0..att.kind.provides_slots.len() ).rev() {
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

        attachments;


        if let SubSelectMode::None = self.mode {
            for (idx, l) in lines.drain(..).enumerate() {

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

            batch.printer( Point{ x: 11, y: size.1 as i32 - 3}, "#[inf_gold]s#[] to select new attachment. #[inf_gold]d#[] to detach.", TextAlign::Left, Some(inf_deep));

        } else if let SubSelectMode::Attach{selected: line_idx}= self.mode {
            // abbreviated version of the basic line display
            for (idx, l) in lines.drain(..).enumerate() {

                let mut start_colors = ColorPair{bg: inf_deep, fg: white };

                let mut abbv = l.0.clone();

                if idx == self.selected_slot as usize {
                    start_colors = ColorPair{fg: inf_deep, bg: white };
                } else if l.0.len() > 4 {
                    abbv = format!( "{}.", l.0[..3].to_string() );
                }

                let mut start = format!("└ {}:", abbv);
                if idx == 0 {
                    start = format!("☼ {}:", abbv);
                }

                let offs = l.2 + start.len() as i32 - 1;

                batch.print_color( Point{ x: l.2, y: idx as i32 + 2}, start, start_colors );

                let name = format!("#[]{}", l.1 );

                batch.printer( Point{ x: offs, y: idx as i32 + 2}, name, TextAlign::Left, Some(inf_deep) );
            }

            // cut off just in case
            batch.fill_region(
                Rect{ x1: size.0 as i32 / 2 - 3, x2: size.0 as i32 - 5, y1: 2, y2: size.1 as i32 - 2 },
                ColorPair{ fg: WHITE.into(), bg: inf_deep },
                ' '
            );
            // draw divider bar
            for y in 2..size.1 as i32 - 2 {
                batch.set( Point{x: size.0 as i32 / 2 - 4, y}, ColorPair{fg: white, bg: inf_deep}, to_cp437('│') );
            }
            batch.set( Point{x: size.0 as i32 / 2 - 4, y: 1}, ColorPair{fg: white, bg: inf_deep}, to_cp437('╒') );
            batch.set( Point{x: size.0 as i32 / 2 - 4, y: size.1 as i32 - 2}, ColorPair{fg: white, bg: inf_deep}, to_cp437('╘') );

            let mut lines = Vec::<(String, Vec<AttachmentFeatureDescriptor>)>::new();

            for (acq, br) in self.acquire.iter() {
                match acq {
                    AttachmentAcquireSource::Inventory(idx) => {
                        let item = actor.inventory.inventory.get(*idx).unwrap();
                        let att_as = item.attaches_as.as_ref().unwrap();
                        lines.push( att_as.describe(&context.at) );
                    },
                    AttachmentAcquireSource::Floor{x, y, idx} => {
                        let opos = (actor.position.0 + x, actor.position.1 + y);
                        let opt = context.map.list_neos( opos );

                        if let Some(neo_vec) = opt {
                            let neo = &neo_vec[*idx];
                            if let NonExclusiveOccupant::Item(item) = neo {
                                let att_as = item.attaches_as.as_ref().unwrap();
                                lines.push( att_as.describe(&context.at) );
                            }
                        }
                    }
                }
            }

            let mut y = 2;
            for (idx, lin) in lines.drain(..).enumerate() {
                let mut start_colors = ColorPair{bg: inf_deep, fg: white };
                if idx == line_idx as usize {
                    start_colors = ColorPair{fg: inf_deep, bg: white };
                }

                batch.print_color( Point{ x: size.0 as i32 / 2 - 3, y}, lin.0, start_colors );

                let mut x = size.0 as i32 - 7;

                for afd in lin.1.iter() {
                    match afd {
                        AttachmentFeatureDescriptor::SingleChar{ch, fg, bg} => {
                            batch.set( Point{x, y}, ColorPair{fg: *fg, bg: *bg}, to_cp437(*ch) );
                            x -= 1;
                        },
                        AttachmentFeatureDescriptor::PrinterString(text, width) => {
                            batch.printer( Point{x, y}, text, TextAlign::Right, Some(inf_deep) );
                            x -= width;
                        }
                    }

                }

                y += 1;
            }

            batch.printer( Point{ x: 11, y: size.1 as i32 - 3}, "#[inf_gold]Enter#[] to confirm. #[inf_gold]d#[] to cancel.", TextAlign::Left, Some(inf_deep));
        }

        let _ = batch.submit(5000);

    }
}

impl AttachmentOverviewMenu {
    pub fn new() -> Self {
        Self {
            key_in: Input::None,
            selected_slot: 0,
            mode: SubSelectMode::None,
            acquire: vec![]
        }
    }

    pub fn from_slot(s: i32) -> Self {
        Self {
            key_in: Input::None,
            selected_slot: s,
            mode: SubSelectMode::None,
            acquire: vec![]
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

    fn get_relevant_attachments( &mut self, slot_type: String, inv: &Inventory, pos: &(i32, i32), ctx: &MenuContext ) {
        let mut out = vec![];

        //print!("{}\n", slot_type);

        for (idx, item) in inv.inventory.iter().enumerate() {

            let mut works = false;
            let mut brace = false;

            if let Some(packed) = item.attaches_as.as_ref() {
                let kind = ctx.at.get( &packed.kind ).unwrap();
                let c = is_compat( &kind.holder_kind, &slot_type );
                if c.0 {
                    works = true;
                    brace = c.1;
                }
            }

            if works {
                out.push( (AttachmentAcquireSource::Inventory(idx), brace) );
            }
        }

        for offs in vec![ (0,0), (-1,-1), (0,-1), (1,-1), (-1,0), (1,0), (-1,1), (0,1), (1,1) ].iter() {
            let opos = (pos.0 + offs.0, pos.1 + offs.1);
            let opt = ctx.map.list_neos( opos );

            if let Some(neo_vec) = opt {
                for (idx, neo) in neo_vec.iter().enumerate() {
                    if let NonExclusiveOccupant::Item(item) = neo {
                        let mut works = false;
                        let mut brace = false;

                        if let Some(packed) = item.attaches_as.as_ref() {
                            let kind = ctx.at.get( &packed.kind ).unwrap();
                            let c = is_compat( &kind.holder_kind, &slot_type );
                            if c.0 {
                                works = true;
                                brace = c.1;
                            }
                        }

                        if works {
                            out.push( (AttachmentAcquireSource::Floor{x: offs.0, y: offs.1, idx}, brace) );
                        }

                    }
                }
            }
        }

        self.acquire = out;
    }
}
