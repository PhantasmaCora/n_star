use std::collections::{VecDeque, HashMap};

use bracket_lib::prelude::*;

use crate::actor::Actor;
use crate::turn::Command;
use crate::map::Map;


mod inventory;
pub use inventory::InventoryMenu;

mod grab;
pub use grab::GrabMenu;

mod inspect_inv;
pub use inspect_inv::InspectInvItemMenu;

mod attachment;
pub use attachment::AttachmentOverviewMenu;


pub enum OverlayManagerReturn {
    NoAction,
    SubmitCommands( VecDeque<Command> )
}

pub enum OverlayReturn {
    NoAction,
    ChangeInterface( Box<dyn OverlayMenu> ),
    AppendCommand( Command ),
    AppendImmediate( Command ),
    SubmitCommands
}

pub struct MenuContext<'a> {
    pub map: &'a Map,
    pub other_actors: &'a HashMap<String, Actor>
}

pub struct OverlayMenuManager {
    active: Option<Box<dyn OverlayMenu>>,

    command_queue: VecDeque<Command>
}


impl OverlayMenuManager {
    pub fn make() -> Self {
        OverlayMenuManager {
            active: None,
            command_queue: VecDeque::<Command>::new()
        }
    }

    pub fn set_mode( &mut self, m: Box<dyn OverlayMenu> ) {
        self.active = Some(m);
    }

    pub fn handle_keypress(&mut self, vkc: VirtualKeyCode) {
        if let Some(menu) = &mut self.active {
            menu.handle_keypress(vkc);
        }
    }

    pub fn update(&mut self, actor: &Actor, context: MenuContext) -> OverlayManagerReturn {
        if let Some(menu) = &mut self.active {
            let or = menu.update( actor, context );

            match or {
                OverlayReturn::NoAction => {
                    return OverlayManagerReturn::NoAction;
                },
                OverlayReturn::AppendCommand(cmd) => {
                    self.command_queue.push_back(cmd);
                    return OverlayManagerReturn::NoAction;
                },
                OverlayReturn::AppendImmediate(cmd) => {
                    self.command_queue.push_back(cmd);
                    self.active = None;
                    return OverlayManagerReturn::SubmitCommands( self.command_queue.drain(..).collect() );
                },
                OverlayReturn::SubmitCommands => {
                    self.active = None;
                    return OverlayManagerReturn::SubmitCommands( self.command_queue.drain(..).collect() );
                },
                OverlayReturn::ChangeInterface(menu) => {
                    self.active = Some(menu);
                    return OverlayManagerReturn::NoAction;
                }
            }
        }

        return OverlayManagerReturn::SubmitCommands( self.command_queue.drain(..).collect() );
    }

    pub fn draw_overlay(&self, bt: &mut BTerm, actor: &Actor, context: MenuContext) {
        if let Some(menu) = &self.active {
            menu.draw_overlay( bt, actor, context );
        }
    }

}


pub trait OverlayMenu {
    fn handle_keypress(&mut self, vkc: VirtualKeyCode);

    fn update(&mut self, actor: &Actor, context: MenuContext) -> OverlayReturn;

    fn draw_overlay(&self, bt: &mut BTerm, actor: &Actor, context: MenuContext);
}
