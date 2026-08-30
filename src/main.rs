use std::collections::HashMap;
use std::cmp::max;
use std::rc::Rc;

use rand::prelude::*;
use rand::rngs::ChaCha20Rng;

use bracket_lib::prelude::*;


pub mod actor;
use crate::actor::{Actor, ActorKind, HealthComponent};

pub mod turn;
use crate::turn::{Command, TurnAttempt, ActionResult};


pub mod npc_brain;
use crate::npc_brain::StandardMonsterBrain;


pub mod map;
use crate::map::{Map, Tile};

pub mod mapgen;
use mapgen::MapGenerator;



struct State {
    actor_awaiting_input: Option<String>,
    kind_table: HashMap<String, Rc<ActorKind>>,
    actors: HashMap<String, Actor>,
    action_order: Vec<actor::ActorRegister>,
    player_orders: Option<Command>,
    current_map: Option<Map>,
    gameplay_random: ChaCha20Rng
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {

        self.handle_input(ctx);

        self.update();

        self.render(ctx);
    }
}

// organization --  the Big Three methods
impl State {
    fn handle_input(&mut self, ctx: &mut BTerm) {

        let mut input = INPUT.lock();

        while let Some(ev) = input.pop() {
            match ev {
                BEvent::CloseRequested => { // Window close. Update this to add save option later!
                    ctx.quit();
                },
                BEvent::KeyboardInput{key, pressed, scan_code: _} => {
                    if !pressed {continue}; // we're not interested in key released events
                    self.handle_keypress(key);
                }
                _ => {}
            }
        }
    }

    fn update(&mut self) {
        let mut looping = true;
        // loop of actor turns
        while looping {
            self.action_order.sort_by_key(|a| a.breath ); // sort by who has the most breath points left

            let up = self.action_order.pop();
            if let Some(mut up_register) = up {

                if up_register.breath <= 0 {
                    self.action_order.push(up_register);
                    self.dispense_breath();

                } else {
                    if let Some(mut up_actor) = self.actors.remove( &up_register.name ) {

                        let map = self.current_map.as_mut().expect("No current map!");

                        let mut asc = actor::ActionSelectionContext{
                            player_orders: &mut self.player_orders,
                            map,
                            other_actors: &self.actors,
                            rng: &mut self.gameplay_random
                        };

                        let mut tr = up_actor.get_action(&mut asc);

                        while tr == TurnAttempt::CallMeAgain {
                            tr = up_actor.get_action(&mut asc);
                        }

                        match tr {
                            TurnAttempt::CallMeAgain => {
                                // shouldnt be reachable given above while loop
                            },
                            TurnAttempt::Selected(action) => { // taking action!
                                let mut ar = turn::map_command(action);
                                let mut resctx = turn::ActionResolutionContext{
                                    map,
                                    other_actors: &mut self.actors,
                                    rng: &mut self.gameplay_random
                                };

                                let mut actlooping = true;
                                while actlooping {
                                    let result = ar.resolve( &mut up_actor, &mut resctx );

                                    match result {
                                        ActionResult::Succeeded(breath_cost) => {
                                            up_register.breath -= breath_cost;
                                            actlooping = false;
                                        },
                                        ActionResult::TryAlternate(abox) => {
                                            ar = abox;
                                        },
                                        ActionResult::Failed => {
                                            if up_actor.is_player {
                                                // notify player that their attempt failed
                                            } else {
                                                up_register.breath -= 16; // mild confusion penalty
                                            }

                                            actlooping = false;
                                        }
                                    }
                                }
                            }
                            TurnAttempt::AwaitingInput{name: who} => { self.actor_awaiting_input = Some(who); looping = false; }
                        }

                        self.actors.insert(up_actor.name.clone(), up_actor);
                        self.action_order.push(up_register);
                    }
                    // if the actor is not found, the actor register is no longer needed and should not be pushed back to the turn order
                }

                // maintain phase. clear out anybody who died etc.
            }
        }
        // loop broken - either it is the player's turn or the player character is dead
    }

    fn render(&self, ctx: &mut BTerm) {

        let size = ctx.get_char_size();
        let size = ( size.0 as i32, size.1 as i32 );

        let mut cam_offset = (0, 0);

        // clears to black. update to draw-where-dirty later???
        ctx.cls();

        // tweak later???
        if let Some(name) = &self.actor_awaiting_input {
            if let Some(present_actor) = self.actors.get(name) {
                cam_offset = ( present_actor.position.0 - size.0/2, present_actor.position.1 - size.1/2 );
            }
        }

        // draw map
        if let Some(map) = &self.current_map {
            for x in 0..(map.tiles.dim().0 as i32) {
                for y in 0..(map.tiles.dim().1 as i32) {
                    let spos = (x - cam_offset.0, y - cam_offset.1);
                    if Self::check_in_bounds(spos, size) {
                        let tidx = map.tiles[[ x as usize, y as usize ]];
                        let tile = map.tileset[tidx];
                        ctx.set( spos.0, spos.1, tile.fg, tile.bg, to_cp437(tile.ch) );
                    }
                }
            }
        }

        // draw actors
        for a in self.actors.values() {
            let wpos = a.position;
            let spos = (wpos.0 - cam_offset.0, wpos.1 - cam_offset.1);
            if Self::check_in_bounds(spos, size) {
                let di = a.get_draw_info(); // retrieve tuple of (color, glyph)
                ctx.set( spos.0, spos.1, di.0, (0,0,0), to_cp437(di.1) );
            }
        }
    }

}

// organization -- secondary calls
impl State {

    fn handle_keypress(&mut self, vkc: VirtualKeyCode) {
        match vkc {
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Backslash => { self.player_orders = Some(Command::Wait(512)); },

            VirtualKeyCode::Numpad4 | VirtualKeyCode::Left => { self.player_orders = Some(Command::MoveStep{x: -1, y:0}); },
            VirtualKeyCode::Numpad6 | VirtualKeyCode::Right => { self.player_orders = Some(Command::MoveStep{x: 1, y:0}); },
            VirtualKeyCode::Numpad2 | VirtualKeyCode::Down => { self.player_orders = Some(Command::MoveStep{x: 0, y:1}); },
            VirtualKeyCode::Numpad8 | VirtualKeyCode::Up => { self.player_orders = Some(Command::MoveStep{x: 0, y:-1}); },

            VirtualKeyCode::Numpad1 | VirtualKeyCode::End => { self.player_orders = Some(Command::MoveStep{x: -1, y:1}); },
            VirtualKeyCode::Numpad7 | VirtualKeyCode::Home => { self.player_orders = Some(Command::MoveStep{x: -1, y:-1}); },
            VirtualKeyCode::Numpad3 | VirtualKeyCode::PageUp => { self.player_orders = Some(Command::MoveStep{x: 1, y:-1}); },
            VirtualKeyCode::Numpad9 | VirtualKeyCode::PageDown => { self.player_orders = Some(Command::MoveStep{x: 1, y:1}); },
            _ => {}
        }
    }

    // called when every actor is at zero or negative breath
    fn dispense_breath(&mut self) {
        for ar in self.action_order.iter_mut() {
            if let Some(corresponding_actor) = self.actors.get_mut( &ar.name ) {
                ar.breath += corresponding_actor.bonus_breath;
                corresponding_actor.bonus_breath = 0;

                if ar.breath < 0 { // apply interest if in breath debt
                    let interest = ar.breath * corresponding_actor.kind.breath_interest / 4096;
                    ar.breath += max(interest, -768); // limit interest gain so that we dont get stuck forever
                }

                ar.breath += 1024; // "one turn" by default is 1024 units of time
            }
        }
    }

    fn add_actor( &mut self, mut a: actor::Actor ) {
        let original_name = a.name.clone();
        let n = 1;

        while self.actors.contains_key(&a.name) {
            a.name = original_name.clone() + " " + &n.to_string();
        }

        let register = a.generate_register();
        self.action_order.push(register);

        if let Some(map) = self.current_map.as_mut() {
            map.exclusive_occupancy.insert( a.position, a.name.clone() );
        }

        self.actors.insert( a.name.clone(), a );
    }

    fn check_in_bounds(point: (i32, i32), size: (i32, i32)) -> bool {
        point.0 >= 0 && point.1 >= 0 && point.0 < size.0 && point.1 < size.1
    }
}

fn main() -> BError {
    let context = BTermBuilder::new()
    .with_resource_path("res/")
    .with_dimensions(80*12, 50*12)
    .with_font( "cp437_12x12.png", 12, 12 )
    .with_simple_console(80, 50, "cp437_12x12.png")
    .build()?;

    {
        let mut input = INPUT.lock();
        input.activate_event_queue();
    }

    let playerpawn_kind = actor::ActorKind {
        name: "Player Pawn".to_string(),
        class: '@',
        color: (255, 255, 255),
        breath_interest: 32,
        max_stability: 16
    };

    let npc_kind = actor::ActorKind {
        name: "NPC".to_string(),
        class: 'c',
        color: (255, 128, 64),
        breath_interest: 32,
        max_stability: 16
    };

    let mut kind_table = HashMap::<String, Rc<ActorKind>>::new();
    kind_table.insert(playerpawn_kind.name.clone(), Rc::new(playerpawn_kind));
    kind_table.insert(npc_kind.name.clone(), Rc::new(npc_kind));

    let mut player = Actor {
        is_player: true,
        kind: kind_table["Player Pawn"].clone(),
        name: "Player".to_string(),
        brain: Some(Box::new( actor::PlayerControlBrain{} )),
        position: (0,0),
        health: Some(HealthComponent {
            is_alive: true,
            stability: 16,
            wounds: 3,
            max_stability: 16,
            max_wounds: 3
        }),
        overrides: HashMap::new(),
        bonus_breath: 0
    };

    let mut aslin = Actor {
        is_player: false,
        kind: kind_table["NPC"].clone(),
        name: "Aslin".to_string(),
        brain: Some(Box::new( StandardMonsterBrain{
            courage: 0.5,
            wander_distance: 9.0,
            packmates: vec!["Lienne".to_string(), "Amerta".to_string()],
            pack_center_distance: 12.0,
            pack_loyalty_frac: 0.05,
            priority: npc_brain::MonsterPriority::Wander{x: 5, y:9},
            sleep_time: 0
        } )),
        position: (6,6),
        health: Some(HealthComponent {
            is_alive: true,
            stability: 16,
            wounds: 3,
            max_stability: 16,
            max_wounds: 3
        }),
        overrides: HashMap::new(),
        bonus_breath: 0
    };
    let mut amerta = Actor {
        is_player: false,
        kind: kind_table["NPC"].clone(),
        name: "Amerta".to_string(),
        brain: Some(Box::new( StandardMonsterBrain{
            courage: 0.5,
            wander_distance: 9.0,
            packmates: vec!["Lienne".to_string(), "Aslin".to_string()],
            pack_center_distance: 12.0,
            pack_loyalty_frac: 0.05,
            priority: npc_brain::MonsterPriority::Wander{x: 5, y:9},
            sleep_time: 0
        } )),
        position: (6,6),
        health: Some(HealthComponent {
            is_alive: true,
            stability: 16,
            wounds: 3,
            max_stability: 16,
            max_wounds: 3
        }),
        overrides: HashMap::new(),
        bonus_breath: 0
    };
    let mut lienne = Actor {
        is_player: false,
        kind: kind_table["NPC"].clone(),
        name: "Lienne".to_string(),
        brain: Some(Box::new( StandardMonsterBrain{
            courage: 0.5,
            wander_distance: 9.0,
            packmates: vec!["Amerta".to_string(), "Aslin".to_string()],
            pack_center_distance: 12.0,
            pack_loyalty_frac: 0.05,
            priority: npc_brain::MonsterPriority::Wander{x: 5, y:9},
            sleep_time: 0
        } )),
        position: (6,6),
        health: Some(HealthComponent {
            is_alive: true,
            stability: 16,
            wounds: 3,
            max_stability: 16,
            max_wounds: 3
        }),
        overrides: HashMap::new(),
        bonus_breath: 0
    };

    let mut gs: State = State {
        player_orders: None,
        actor_awaiting_input: None,
        kind_table,
        actors: HashMap::new(),
        action_order: vec![],
        current_map: None,
        gameplay_random: rand::make_rng()
    };

    //gs.add_actor(npc);

    let bt = Tile{
        fg: (96, 96, 96),
        bg: (0,0,0),
        ch: '.',
        passable: true,
        opaque: false
    };
    let wt = Tile{
        fg: (64, 128, 208),
        bg: (0,0,0),
        ch: '■',
        passable: false,
        opaque: true
    };
    let lwt = Tile{
        fg: (64, 128, 208),
        bg: (0,0,0),
        ch: '▄',
        passable: false,
        opaque: true
    };
    let uwt = Tile{
        fg: (64, 128, 208),
        bg: (0,0,0),
        ch: '▀',
        passable: false,
        opaque: true
    };
    let fwt = Tile{
        fg: (64, 128, 208),
        bg: (0,0,0),
        ch: '▌',
        passable: false,
        opaque: true
    };
    let rwt = Tile{
        fg: (64, 128, 208),
        bg: (0,0,0),
        ch: '▐',
        passable: false,
        opaque: true
    };
    let dt = Tile{
        fg: (0,0,0),
        bg: (0,0,0),
        ch: '.',
        passable: false,
        opaque: true
    };
    let mg = MapGenerator{w: 128, h: 128};
    let m = mg.generate_map(
        vec![ bt, wt, lwt, uwt, fwt, rwt, dt ]
    );

    let mut idx = 0;
    loop {
        let ps = m.is_passable(idx);
        if ps {
            break;
        } else {
            idx += 1;
        }
    }

    let pt = m.index_to_point2d(idx);

    player.position = (pt.x, pt.y);

    let mut idx = 576;
    loop {
        let ps = m.is_passable(idx);
        if ps {
            break;
        } else {
            idx += 1;
        }
    }
    let npc_pt = m.index_to_point2d(idx);
    aslin.position = (npc_pt.x, npc_pt.y);
    amerta.position = (npc_pt.x + 1, npc_pt.y);
    lienne.position = (npc_pt.x + 2, npc_pt.y);

    gs.current_map = Some(m);
    gs.add_actor(player);
    gs.add_actor(aslin);
    gs.add_actor(amerta);
    gs.add_actor(lienne);

    main_loop(context, gs)
}
