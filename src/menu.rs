use std::collections::{VecDeque, HashMap};


use bracket_lib::prelude::*;

use crate::actor::Actor;
use crate::turn::Command;
use crate::map::{Map, NonExclusiveOccupant};
use crate::item::InvItem;


#[derive(PartialEq, Eq)]
pub enum OverlayKind {
    Closed,
    Inventory,
    Grab,
    Attachments
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

    command_queue: VecDeque<Command>,

    in_progress: CommandFormer
}

// key api calls
impl MenuManager {
    pub fn make() -> Self {
        MenuManager {
            active_kind: OverlayKind::Closed,
            selected_slot: 0,
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
            VirtualKeyCode::Up => { self.selected_slot -= 1; },
            VirtualKeyCode::Down => { self.selected_slot += 1; },
            VirtualKeyCode::PageUp => { self.selected_slot -= 16; },
            VirtualKeyCode::PageDown => { self.selected_slot += 16; },

            VirtualKeyCode::D => {
                match self.active_kind {
                    OverlayKind::Inventory => {
                        self.in_progress = CommandFormer::DropItem;
                    },
                    _ => {}
                }

            }

            VirtualKeyCode::Escape => {
                self.command_queue.clear();
                self.active_kind = OverlayKind::Closed;
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
        match self.active_kind {
            OverlayKind::Attachments => {
                return OverlayReturn::NoAction;
                // clamp selected_slot to the relevant number of slots
            },
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
            OverlayKind::Closed => {
                return OverlayReturn::SubmitCommands( self.command_queue.drain(..).collect() );
            }
        }
    }

    pub fn draw_overlay(&self, ctx: &mut BTerm, actor: Option<&Actor>, context: MenuContext) {
        match self.active_kind {
            OverlayKind::Attachments => {
                if let Some(act) = actor {
                    self.draw_attachments(ctx, act);
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
        let size = ctx.get_char_size();
        let mut y = 1;

        let grabs = self.compile_grabables( actor, &context );

        let mut batch = DrawBatch::new();

        batch.draw_double_box(
            Rect{ x1: size.0 as i32 - 32, x2: size.0 as i32 - 5, y1: 1, y2: size.1 as i32 - 2 },
            ColorPair{ fg: (255, 255, 255).into(), bg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0} }
        );
        batch.fill_region(
            Rect{ x1: size.0 as i32 - 31, x2: size.0 as i32 - 5, y1: 2, y2: size.1 as i32 - 2 },
            ColorPair{ fg: (255, 255, 255).into(), bg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0} },
            ' '
        );

        for (idx, grab) in grabs.iter().enumerate() {
            y += 1;

            let item = &grab.2;

            let num_id = format!("{:03}", idx);
            let mut num_colors = ColorPair{bg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0}, fg: (255, 255, 255).into() };
            if idx == self.selected_slot as usize {
                num_colors = ColorPair{fg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0}, bg: (255, 255, 255).into() };
            }
            batch.print_color( Point{ x: size.0 as i32 - 31, y}, num_id, num_colors );

            batch.set( Point{x: size.0 as i32 - 27, y}, ColorPair{fg: item.color.into(), bg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0} }, to_cp437( item.display_ch ) );

            let mut name = item.display_name.clone();
            if name.len() > 18 {
                name = item.display_name[0..(size.0 as usize - 14)].to_string() + "...";
            }

            batch.print_color( Point{ x: size.0 as i32 - 25, y}, name, ColorPair{bg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0}, fg: (255, 255, 255).into() } );
        }

        let _ = batch.submit(5000);
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
        let inv = &actor.inventory;

        let size = ctx.get_char_size();
        let mut y = 1;

        let mut batch = DrawBatch::new();

        batch.draw_double_box(
            Rect{ x1: 5, x2: size.0 as i32 - 5, y1: 1, y2: size.1 as i32 - 2 },
            ColorPair{ fg: (255, 255, 255).into(), bg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0} }
        );
        batch.fill_region(
            Rect{ x1: 6, x2: size.0 as i32 - 5, y1: 2, y2: size.1 as i32 - 2 },
            ColorPair{ fg: (255, 255, 255).into(), bg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0} },
            ' '
        );

        for (idx, item) in inv.iter().enumerate() {
            y += 1;

            let num_id = format!("{:03}", idx);
            let mut num_colors = ColorPair{bg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0}, fg: (255, 255, 255).into() };
            if idx == self.selected_slot as usize {
                num_colors = ColorPair{fg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0}, bg: (255, 255, 255).into() };
            }
            batch.print_color( Point{ x:6, y}, num_id, num_colors );

            batch.set( Point{x: 10, y}, ColorPair{fg: item.color.into(), bg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0} }, to_cp437( item.display_ch ) );

            let mut name = item.display_name.clone();
            if name.len() > size.0 as usize - 16 {
                name = item.display_name[0..(size.0 as usize - 20)].to_string() + "...";
            }

            batch.print_color( Point{ x:12, y}, name, ColorPair{bg: RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0}, fg: (255, 255, 255).into() } );
        }

        batch.target(0);
        let _ = batch.submit(5000);
    }
}
