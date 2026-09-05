use std::collections::{VecDeque, HashMap};


use textwrap::wrap;

use bracket_lib::prelude::*;

use crate::actor::Actor;
use crate::turn::Command;
use crate::map::{Map, NonExclusiveOccupant};
use crate::item::{InvItem, ItemSize, LickResponse};


#[derive(PartialEq, Eq)]
pub enum OverlayKind {
    Closed,
    Inventory,
    Grab,
    InspectItem(InspectSubView)
}

#[derive(PartialEq, Eq)]
pub enum InspectSubView {
    None,
    Lick
}

pub enum OverlayReturn {
    NoAction,
    SubmitCommands( VecDeque<Command> )
}

#[derive(PartialEq)]
pub enum CommandFormer {
    NoAction,
    DropItem,
    GrabItem
}

pub struct MenuContext<'a> {
    pub map: &'a Map,
    pub other_actors: &'a HashMap<String, Actor>
}

pub struct MenuManager {
    active_kind: OverlayKind,
    selected_slot: i32,
    scroll: i32,

    command_queue: VecDeque<Command>,

    in_progress: CommandFormer
}

// key api calls
impl MenuManager {
    pub fn make() -> Self {
        MenuManager {
            active_kind: OverlayKind::Closed,
            selected_slot: 0,
            scroll: 0,
            command_queue: VecDeque::<Command>::new(),
            in_progress: CommandFormer::NoAction
        }
    }

    pub fn try_set_mode( &mut self, kind: OverlayKind ) {
        if self.active_kind == OverlayKind::Closed {
            self.active_kind = kind;
            self.selected_slot = 0;
        }
    }

    pub fn handle_keypress(&mut self, vkc: VirtualKeyCode) {
        match vkc {
            VirtualKeyCode::Up => {
                match self.active_kind {
                    OverlayKind::InspectItem(InspectSubView::Lick) => {},
                    _ => {self.selected_slot -= 1;}
                }
            },
            VirtualKeyCode::Down => {
                match self.active_kind {
                    OverlayKind::InspectItem(InspectSubView::Lick) => {},
                    _ => {self.selected_slot += 1;}
                }
            },
            VirtualKeyCode::PageUp => {
                match self.active_kind {
                    OverlayKind::InspectItem(InspectSubView::Lick) => {},
                    _ => {self.selected_slot -= 16;}
                }
            },
            VirtualKeyCode::PageDown => {
                match self.active_kind {
                    OverlayKind::InspectItem(InspectSubView::Lick) => {},
                    _ => {self.selected_slot += 16;}
                }
            },

            VirtualKeyCode::D => {
                match self.active_kind {
                    OverlayKind::Inventory => {
                        self.in_progress = CommandFormer::DropItem;
                    },
                    _ => {}
                }

            },

            VirtualKeyCode::I => {
                match self.active_kind {
                    OverlayKind::Inventory => {
                        self.active_kind = OverlayKind::InspectItem(InspectSubView::None);
                    },
                    _ => {}
                }

            },

            VirtualKeyCode::L => {
                match self.active_kind {
                    OverlayKind::InspectItem(InspectSubView::None) => {
                        self.active_kind = OverlayKind::InspectItem(InspectSubView::Lick);
                    },
                    _ => {}
                }

            },

            VirtualKeyCode::Escape => {
                match &self.active_kind {
                    OverlayKind::InspectItem(InspectSubView::None) => {
                        self.active_kind = OverlayKind::Inventory;
                    },
                    OverlayKind::InspectItem(_sv) => {
                        self.active_kind = OverlayKind::InspectItem(InspectSubView::None);
                    },
                    _ => {
                        self.command_queue.clear();
                        self.active_kind = OverlayKind::Closed;
                    }
                }
            },

            VirtualKeyCode::Return => {
                match self.active_kind {
                    OverlayKind::Grab => {
                        self.in_progress = CommandFormer::GrabItem;
                    },
                    _ => {
                        self.active_kind = OverlayKind::Closed;
                    }
                }

            },
            _ => {}
        }
    }

    pub fn update(&mut self, actor: Option<&Actor>, context: MenuContext) -> OverlayReturn {
        match &self.active_kind {
            /*OverlayKind::Attachments => {
                return OverlayReturn::NoAction;
                // clamp selected_slot to the relevant number of slots
            },*/
            OverlayKind::Grab => {
                let grabs = self.compile_grabables( actor.unwrap(), &context );

                // clamp selected slot
                self.clamp_selection( grabs.len() );

                if let CommandFormer::GrabItem = self.in_progress {
                    if !grabs.is_empty() {
                        let g = grabs[self.selected_slot as usize];

                        let cmd = Command::GrabItem{
                            x: g.0.0,
                            y: g.0.1,
                            idx: g.1
                        };

                        self.command_queue.push_back( cmd );

                        self.in_progress = CommandFormer::NoAction;
                        self.active_kind = OverlayKind::Closed;

                        return OverlayReturn::SubmitCommands( self.command_queue.drain(..).collect() );
                    }
                }

                if grabs.is_empty() {
                    return OverlayReturn::SubmitCommands( self.command_queue.drain(..).collect() );
                }

                return OverlayReturn::NoAction;
            }
            OverlayKind::Inventory => {
                // clamp selected slot
                let n_items = actor.unwrap().inventory.len();

                self.clamp_selection( n_items );

                if let CommandFormer::DropItem = self.in_progress {
                    if n_items > 0 {
                        self.command_queue.push_back( Command::DropItem( self.selected_slot as usize ) );

                        self.in_progress = CommandFormer::NoAction;
                        self.active_kind = OverlayKind::Closed;

                        return OverlayReturn::SubmitCommands( self.command_queue.drain(..).collect() );
                    }
                }

                return OverlayReturn::NoAction;
            },
            OverlayKind::InspectItem(_sv) => {
                let n_items = actor.unwrap().inventory.len();

                self.clamp_selection( n_items );

                return OverlayReturn::NoAction;
            }
            OverlayKind::Closed => {
                return OverlayReturn::SubmitCommands( self.command_queue.drain(..).collect() );
            }
        }
    }

    pub fn draw_overlay(&self, ctx: &mut BTerm, actor: Option<&Actor>, context: MenuContext) {
        match &self.active_kind {
            /*OverlayKind::Attachments => {
                if let Some(act) = actor {
                    self.draw_attachments(ctx, act);
                }
            },*/
            OverlayKind::InspectItem(sv) => {
                if let Some(act) = actor {
                    self.draw_inspect_item(ctx, act, sv);
                }
            },
            OverlayKind::Inventory => {
                if let Some(act) = actor {
                    self.draw_inventory(ctx, act);
                }
            },
            OverlayKind::Grab => {
                if let Some(act) = actor {
                    self.draw_grab(ctx, act, context);
                }
            },
            OverlayKind::Closed => { /* Should be unreachable! */ }
        }
    }

}

// internals
impl MenuManager {

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

    fn draw_grab(&self, ctx: &mut BTerm, actor: &Actor, context: MenuContext) {
        let inf_deep = palette_color(&"inf_deep").unwrap();
        let inf_bulk = palette_color(&"inf_bulk").unwrap();
        let inf_invl = palette_color(&"inf_invl").unwrap();

        let size = ctx.get_char_size();
        let mut y = 1;

        let grabs = self.compile_grabables( actor, &context );

        let mut batch = DrawBatch::new();

        batch.draw_double_box(
            Rect{ x1: size.0 as i32 - 32, x2: size.0 as i32 - 2, y1: 1, y2: size.1 as i32 - 2 },
            ColorPair{ fg: WHITE.into(), bg: inf_deep }
        );
        batch.fill_region(
            Rect{ x1: size.0 as i32 - 31, x2: size.0 as i32 - 2, y1: 2, y2: size.1 as i32 - 2 },
            ColorPair{ fg: WHITE.into(), bg: inf_deep },
            ' '
        );

        let inv = &actor.inventory;

        let volume_print = format!("╡{:>5}/{:>5}v-", (inv.inv_volume.1 * 10.0).ceil() / 10.0, (inv.inv_volume.0 * 10.0).ceil() / 10.0);
        batch.print_color( Point{ x: size.0 as i32 - 30, y: 1}, volume_print, ColorPair{bg: inf_deep, fg: WHITE.into() } );

        let bulk_print = format!("{:2}/{:2}B", inv.inv_bulky.1, inv.inv_bulky.0);
        batch.print_color( Point{ x: size.0 as i32 - 16, y: 1}, bulk_print, ColorPair{bg: inf_deep, fg: inf_bulk } );

        batch.set( Point{x: size.0 as i32 - 10, y: 1}, ColorPair{fg: WHITE.into(), bg: inf_deep }, to_cp437('╞') );

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
            if name.len() > 12 {
                name = item.display_name[0..(size.0 as usize - 9)].to_string() + "...";
            }

            batch.print_color( Point{ x: size.0 as i32 - 26, y}, name, ColorPair{bg: inf_deep, fg: WHITE.into() } );

            if item.can_stack > 0 {
                let num = format!("x{:03}", item.stack);
                batch.print_color( Point{ x: size.0 as i32 - 13, y}, num, ColorPair{bg: inf_deep, fg: WHITE.into() } );
            }

            match item.size {
                ItemSize::Volume(v) => {
                    let mut vprint = format!("{:>5}", v);

                    if item.stack > 1 {
                        vprint = format!("{:>5} ea", v);
                    }

                    batch.print_color( Point{ x: size.0 as i32 - 9, y}, vprint, ColorPair{bg: inf_deep, fg: WHITE.into() } );
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



    fn draw_inspect_item(&self, ctx: &mut BTerm, actor: &Actor, subview: &InspectSubView) {
        let inf_deep = palette_color(&"inf_deep").unwrap();
        let inf_grey = palette_color(&"inf_grey").unwrap();
        let white: RGBA = WHITE.into();

        let mut batch = DrawBatch::new();

        let size = ctx.get_char_size();

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
                let mut vprint = format!("{:>5}", v);

                if item.stack > 1 {
                    vprint = format!("{:>5} ea", v);
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

        let lines = item.get_inspect_text( (size.0 - 22) as usize );

        let mut y = 10;

        for l in lines {
            y += 1;
            batch.print_color( Point{ x: 12, y}, l,  ColorPair{bg: inf_deep, fg: white } );
        }

        batch.printer( Point{ x: 11, y: size.1 as i32 - 3}, "#[inf_gold]d#[] to drop. #[inf_gold]l#[] to lick.", TextAlign::Left, Some(inf_deep));

        let _ = batch.submit(5000);

        // handle subview drawing
        match subview {
            InspectSubView::None => {},
            InspectSubView::Lick => {
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


    fn draw_attachments(&self, ctx: &mut BTerm, actor: &Actor) {
        if actor.attachments.is_none() {
            return;
        }

        let mut y = 0;
        let mut x_indent = 0;
        let mut x = 0;
    }



    fn draw_inventory(&self, ctx: &mut BTerm, actor: &Actor) {
        let inf_deep = palette_color(&"inf_deep").unwrap();
        let inf_bulk = palette_color(&"inf_bulk").unwrap();

        let inv = &actor.inventory;

        let size = ctx.get_char_size();
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
            if name.len() > 16 {
                name = item.display_name[0..(size.0 as usize - 12)].to_string() + "...";
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
