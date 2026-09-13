use std::collections::{HashMap, HashSet, VecDeque};
use std::cmp::max;
use std::rc::Rc;

use rand::prelude::*;
use rand::rngs::ChaCha20Rng;

use bracket_lib::prelude::*;




use n_star::dataread::get_data;

use n_star::actor;
use n_star::actor::{Actor, ActorKind, HealthComponent};
use n_star::actor::attachment::{Attachment, AttachmentType};

use n_star::turn;
use n_star::turn::{Command, TurnAttempt, ActionResult};

use n_star::item::{Inventory, InvItem, ItemSize, LickResponse};

use n_star::map::{Map, Tile, NonExclusiveOccupant};
use n_star::map::tile_render::{TileRenderContext, TileRender, TileDrawType, FixedTileRender, Wall4WayTileRender};

use n_star::mapgen::MapGenerator;

use n_star::menu::{ OverlayMenuManager, OverlayManagerReturn, MenuContext, GrabMenu, InventoryMenu, AttachmentOverviewMenu };



const EIGHT_WAYS: [(i32, i32); 8] = [(-1,-1), (0,-1), (1,-1), (-1,0), (1,0), (-1,1), (0,1), (1,1)];


enum GameMode {
    Playing,
    OverlayMenu,
    TitleMenu
}

struct State {
    actor_awaiting_input: Option<String>,
    kind_table: HashMap<String, Rc<ActorKind>>,
    attach_table: HashMap<String, Rc<AttachmentType>>,
    actors: HashMap<String, Actor>,
    action_order: Vec<actor::ActorRegister>,
    player_orders: Option<Command>,
    chain_player_orders: VecDeque<Command>,
    current_map: Option<Map>,
    gameplay_random: ChaCha20Rng,
    game_mode: GameMode,
    menu_manager: OverlayMenuManager,
    frame: usize,
    beat: usize
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {

        self.handle_input(ctx);

        // clears to black. update to draw-where-dirty later???
        ctx.cls();

        match self.game_mode {
            GameMode::Playing => {
                self.update();

                self.render_world(ctx);
                // self.render_hud(ctx); for later
            },
            GameMode::OverlayMenu => {
                let mut actor_opt = None;
                if let Some(name) = &self.actor_awaiting_input {
                    actor_opt = self.actors.get(name);
                } else {
                    panic![]
                }

                let actor_ref = actor_opt.unwrap();

                let upd_context = MenuContext{
                    map: self.current_map.as_ref().unwrap(),
                    other_actors: &self.actors,
                    at: &self.attach_table
                };

                let menu_result = self.menu_manager.update( actor_ref, upd_context );

                if let OverlayManagerReturn::SubmitCommands(mut cvec) = menu_result {
                    self.chain_player_orders.append(&mut cvec);

                    self.game_mode = GameMode::Playing;

                    self.update();

                    self.render_world(ctx);
                    // self.render_hud(ctx); for later
                } else {
                    self.render_world(ctx);
                    // self.render_hud(ctx); for later

                    let draw_context = MenuContext{
                        map: self.current_map.as_ref().unwrap(),
                        other_actors: &self.actors,
                        at: &self.attach_table
                    };

                    self.menu_manager.draw_overlay( ctx, actor_ref, draw_context );
                }
            },
            GameMode::TitleMenu => {
                // do some stuff later on
            }
        }

        let _ = render_draw_buffer(ctx);

        self.frame += 1;
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

                    match self.game_mode {
                        GameMode::Playing => { self.handle_keypress(key); },
                        GameMode::OverlayMenu => { self.menu_manager.handle_keypress(key) },
                        GameMode::TitleMenu => {}
                    }

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

                        self.dechain();

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
                                    at: &self.attach_table,
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
                            TurnAttempt::AwaitingInput{name: who} => {
                                self.actor_awaiting_input = Some(who); looping = false;
                            }
                        }

                        self.actors.insert(up_actor.name.clone(), up_actor);
                        self.action_order.push(up_register);
                    }
                    // if the actor is not found, the actor register is no longer needed and should not be pushed back to the turn order
                }

                // maintain phase. clear out anybody who died etc.
                let mut dead_keys = vec![];

                for k in self.actors.keys() {
                    let mut dead = false;
                    {
                        let a = self.actors.get(k).unwrap();
                        if let Some(hc) = &a.health {
                            if !hc.is_alive {
                                dead_keys.push(k.clone());
                            }
                        }
                    }
                }

                for k in dead_keys.drain(..) {
                    let a = self.actors.remove(&k).unwrap();
                    self.current_map.as_mut().unwrap().exclusive_occupancy.remove(&a.position);
                    self.current_map.as_mut().unwrap().add_neo( NonExclusiveOccupant::Corpse, a.position );
                }

            }
        }
        // loop broken - either it is the player's turn or the player character is dead
    }

    fn render_world(&self, ctx: &mut BTerm) {

        let mut batch = DrawBatch::new();

        let size = ctx.get_char_size();
        let size = ( size.0 as i32, size.1 as i32 );

        let mut cam_offset = (0, 0);

        // tweak later???
        let mut fov: Option<&HashSet<Point>> = None;
        let mut memory: Option<&HashSet<Point>> = None;

        if let Some(name) = &self.actor_awaiting_input {
            if let Some(present_actor) = self.actors.get(name) {
                cam_offset = ( present_actor.position.0 - size.0/2, present_actor.position.1 - size.1/2 );

                fov = present_actor.fov.as_ref();
                memory = present_actor.memory.as_ref();
            }
        }

        // draw map
        if let Some(map) = &self.current_map {
            for x in 0..(map.tiles.dim().0 as i32) {
                for y in 0..(map.tiles.dim().1 as i32) {
                    let spos = (x - cam_offset.0, y - cam_offset.1);

                    let pt = Point{x,y};

                    if Self::check_in_bounds(spos, size) && ( (fov.is_none() || fov.unwrap().contains(&pt)) || (memory.is_some() && memory.unwrap().contains(&pt)) ) {

                        let tidx = map.tiles[[ x as usize, y as usize ]];
                        let tile = &map.tileset[tidx];

                        let mut neighbors = [None; 8];

                        for (idx, offs) in EIGHT_WAYS.iter().enumerate() {
                            let ox = x + offs.0;
                            let oy = y + offs.1;

                            let opt = (ox as usize, oy as usize);

                            let o_bracketpt = Point{x: ox, y: oy};

                            if ( ox < 0 || ox >= map.tiles.dim().0 as i32 || oy < 0 || oy >= map.tiles.dim().1 as i32 ) || (
                                ( fov.is_some() && !fov.unwrap().contains( &o_bracketpt ) ) &&
                                ( memory.is_none() || !memory.unwrap().contains( &o_bracketpt ) )
                            ) {
                                continue;
                            }

                            neighbors[idx] = Some( map.tiles[[opt.0, opt.1]] );
                        }

                        let tr_ctx = TileRenderContext{
                            me: tidx,
                            neighbors
                        };

                        let mut fg_color : RGBA = (0,0,0).into();
                        let mut bg_color : RGBA = (0,0,0).into();
                        let mut character : char = ' ';

                        match tile.tr.get_draw(tr_ctx, &map.tileset) {
                            TileDrawType::Dont => {continue;}
                            TileDrawType::Regular(ch, flip) => {
                                character = ch;
                                if flip {
                                    fg_color = tile.bg.into();
                                    bg_color = tile.fg.into();
                                } else {
                                    fg_color = tile.fg.into();
                                    bg_color = tile.bg.into();
                                }
                            },
                            TileDrawType::Override{ch, fg, bg} => {
                                character = ch;
                                fg_color = fg.into();
                                bg_color = bg.into();
                            }
                        }

                        if fov.is_none() || fov.unwrap().contains(&pt) {
                            // draw non-exclusive occupants if any
                            let wrap = map.list_neos( (x, y) );

                            if let Some(neo_vec) = wrap {
                                let f = self.frame / 30;
                                let idx = f % neo_vec.len();
                                let occ = neo_vec.get(idx).unwrap();

                                match occ {
                                    NonExclusiveOccupant::Item(it) => {
                                        character = it.display_ch;
                                        fg_color = it.color.into();
                                    }
                                    NonExclusiveOccupant::Corpse => {
                                        character = '%';
                                        fg_color = ( 240, 56, 32 ).into();
                                    }
                                }
                            }

                        } else {
                            let fade = RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0};

                            let fade_v = HSV::from(fade).v;
                            let fg_v = HSV::from(fg_color).v;
                            let bg_v = HSV::from(bg_color).v;

                            if fg_v > fade_v {
                                fg_color = fg_color.lerp(fade, 0.75);
                            }
                            if bg_v > fade_v {
                                bg_color = bg_color.lerp(fade, 0.75);
                            }
                        }

                        batch.set( Point{x: spos.0, y: spos.1}, ColorPair{fg: fg_color, bg: bg_color}, to_cp437(character) );

                    }
                }
            }
        }

        // draw actors
        for a in self.actors.values() {
            let wpos = a.position;

            let spos = (wpos.0 - cam_offset.0, wpos.1 - cam_offset.1);
            if Self::check_in_bounds(spos, size) && (fov.is_none() || fov.unwrap().contains( &Point{x: wpos.0, y: wpos.1} ) ) {
                let di = a.get_draw_info(); // retrieve tuple of (color, glyph)
                batch.set( Point{x: spos.0, y: spos.1}, ColorPair{fg: di.0.into(), bg: (0,0,0).into()}, to_cp437(di.1) );
            }
        }

        batch.target(0);
        let _  = batch.submit(0);
    }

}

// organization -- secondary calls
impl State {

    fn handle_keypress(&mut self, vkc: VirtualKeyCode) {
        match vkc {
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Backslash => { self.chain_player_orders.push_back(Command::Wait(512)); },

            VirtualKeyCode::Numpad4 | VirtualKeyCode::Left => { self.chain_player_orders.push_back(Command::MoveStep{x: -1, y:0}); },
            VirtualKeyCode::Numpad6 | VirtualKeyCode::Right => { self.chain_player_orders.push_back(Command::MoveStep{x: 1, y:0}); },
            VirtualKeyCode::Numpad2 | VirtualKeyCode::Down => { self.chain_player_orders.push_back(Command::MoveStep{x: 0, y:1}); },
            VirtualKeyCode::Numpad8 | VirtualKeyCode::Up => { self.chain_player_orders.push_back(Command::MoveStep{x: 0, y:-1}); },

            VirtualKeyCode::Numpad1 | VirtualKeyCode::End => { self.chain_player_orders.push_back(Command::MoveStep{x: -1, y:1}); },
            VirtualKeyCode::Numpad7 | VirtualKeyCode::Home => { self.chain_player_orders.push_back(Command::MoveStep{x: -1, y:-1}); },
            VirtualKeyCode::Numpad3 | VirtualKeyCode::PageUp => { self.chain_player_orders.push_back(Command::MoveStep{x: 1, y:-1}); },
            VirtualKeyCode::Numpad9 | VirtualKeyCode::PageDown => { self.chain_player_orders.push_back(Command::MoveStep{x: 1, y:1}); },

            VirtualKeyCode::E => { self.menu_manager.set_mode( Box::new( InventoryMenu::new() ) ); self.game_mode = GameMode::OverlayMenu; },
            VirtualKeyCode::A => { self.menu_manager.set_mode( Box::new( AttachmentOverviewMenu::new() ) ); self.game_mode = GameMode::OverlayMenu; },
            VirtualKeyCode::G => { self.menu_manager.set_mode( Box::new( GrabMenu::new() ) ); self.game_mode = GameMode::OverlayMenu; },
            _ => {}
        }
    }

    fn dechain(&mut self) {
        if self.player_orders.is_none() && !self.chain_player_orders.is_empty() {
            self.player_orders = self.chain_player_orders.pop_front();
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

        self.beat += 1;
    }

    fn add_actor( &mut self, mut a: actor::Actor ) {
        if let None = a.id {
            a.id = Some( format!( "a{}", self.gameplay_random.random::<u32>() ) );
        }

        let register = a.generate_register();
        self.action_order.push(register);

        if let Some(map) = self.current_map.as_mut() {
            map.exclusive_occupancy.insert( a.position, a.id.as_ref().unwrap().clone() );
        }

        self.actors.insert( a.id.as_ref().unwrap().clone(), a );
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

    bracket_lib::color::register_palette_color("inf_gold", (255, 208, 128) ); // interface gold
    bracket_lib::color::register_palette_color("inf_deep", RGBA{r: 0.02, g: 0.1, b: 0.14, a: 1.0} ); // interface deep
    bracket_lib::color::register_palette_color("inf_grey", HSV{h: 0.0, s: 0.0, v: 0.7} ); // interface grey
    bracket_lib::color::register_palette_color("inf_dgry", HSV{h: 0.5, s: 0.05, v: 0.45} ); // interface dark gray
    bracket_lib::color::register_palette_color("inf_bulk", (208, 128, 16) ); // interface bulk
    bracket_lib::color::register_palette_color("inf_invl", (208, 128, 128) ); // interface invalid
    bracket_lib::color::register_palette_color("inf_good", (145, 204, 163) ); // interface good
    bracket_lib::color::register_palette_color("inf_attc", HSV{h: 0.05, s: 0.75, v: 0.78} ); // interface attachment

    {
        let mut input = INPUT.lock();
        input.activate_event_queue();
    }

    let playerpawn_kind = actor::ActorKind {
        name: "Player Pawn".to_string(),
        class: '@',
        color: (255, 255, 255),
        breath_interest: 32,
        max_stability: 16,
        sight_range: 32
    };

    let npc_kind = actor::ActorKind {
        name: "NPC".to_string(),
        class: 'c',
        color: (255, 128, 64),
        breath_interest: 32,
        max_stability: 16,
        sight_range: 24
    };



    let mut asset_data = get_data("./res".into()).unwrap();

    //print!("{:?}\n", asset_data);

    let attach_table = asset_data.get_attachment_table();

    let mut kind_table = HashMap::<String, Rc<ActorKind>>::new();
    kind_table.insert(playerpawn_kind.name.clone(), Rc::new(playerpawn_kind));
    kind_table.insert(npc_kind.name.clone(), Rc::new(npc_kind));

    let mut npcs = vec![];
    for _i in 0..64 {
        let mut npc = Actor {
            is_player: false,
            kind: kind_table["NPC"].clone(),
            name: "Npc".to_string(),
            id: None,
            brain: Some(Box::new( actor::NullBrain{} )),
            position: (0,0),
            health: Some(HealthComponent {
                is_alive: true,
                stability: 16,
                wounds: 3,
                max_stability: 16,
                max_wounds: 3,
                armor_rating: 1
            }),
            attachments: None,
            inventory: Inventory{
                inventory: Vec::new(),
                inv_volume: (32.0, 0.0),
                inv_bulky: (3, 0)
            },
            overrides: HashMap::new(),
            bonus_breath: 0,
            fov: None,
            memory: None
        };

        npcs.push(npc);
    }


    let attachments = actor::attachment::make_test_att_comp(&attach_table);

    let mut player = Actor {
        is_player: true,
        kind: kind_table["Player Pawn"].clone(),
        name: "Player".to_string(),
        id: Some("Player".to_string()),
        brain: Some(Box::new( actor::PlayerControlBrain{} )),
        position: (0,0),
        health: Some(HealthComponent {
            is_alive: true,
            stability: 16,
            wounds: 3,
            max_stability: 16,
            max_wounds: 3,
            armor_rating: 1
        }),
        attachments: Some(attachments),
        inventory: Inventory{
            inventory: Vec::new(),
            inv_volume: (32.0, 0.0),
            inv_bulky: (3, 0)
        },
        overrides: HashMap::new(),
        bonus_breath: 0,
        fov: Some( HashSet::<Point>::new() ),
        memory: Some( HashSet::<Point>::new() )
    };

    /*let shotgun = Attachment{
        kind: attach_table.get("1_Shotgun2").unwrap().clone(),
        slots: vec![]
    };

    let _ = player.inventory.add_item( shotgun.as_item() );

    let leg = Attachment{
    kind: attach_table.get("Leg3").unwrap().clone(),
    slots: vec![]
    };

    let _ = player.inventory.add_item( leg.as_item() );*/

    let sword = Attachment{
        kind: attach_table.get("1_Sword1").unwrap().clone(),
        slots: vec![],
        brace: None,
        parent: None,
        id: None
    };

    let _ = player.inventory.add_item( sword.as_simple_item() );

    let ps1 = Attachment{
        kind: attach_table.get("0_Pistol2").unwrap().clone(),
        slots: vec![],
        brace: None,
        parent: None,
        id: None
    };

    let _ = player.inventory.add_item( ps1.as_simple_item() );

    let bg = Attachment{
        kind: attach_table.get("1_BeamGun1").unwrap().clone(),
        slots: vec![],
        brace: None,
        parent: None,
        id: None
    };

    let _ = player.inventory.add_item( bg.as_simple_item() );

    let _ = player.inventory.add_item(
        InvItem{display_name: "Regen Cell".to_string(), display_ch: 'ö', color: (255, 64, 64), can_stack: 2, stack: 3, size: ItemSize::Volume(2.1), flavor_text: "A standard healing item, administered orally. Pulsates slightly with lively essence.".to_string(), attaches_as: None, lick_result: LickResponse::FlavorText("#[inf_good]Tingles pleasantly on your tongue.#[]".to_string(), 34)  }
    );


    let mut gs: State = State {
        player_orders: None,
        actor_awaiting_input: None,
        kind_table,
        attach_table,
        actors: HashMap::new(),
        action_order: vec![],
        current_map: None,
        chain_player_orders: VecDeque::new(),
        game_mode: GameMode::Playing,
        menu_manager: OverlayMenuManager::make(),
        gameplay_random: rand::make_rng(),
        frame: 0,
        beat: 0
    };

    let bt = Tile{
        fg: (96, 96, 96),
        bg: (0,0,0),
        tr: Box::new( FixedTileRender{ch: '.'} ),
        passable: true,
        opaque: false
    };
    let wt = Tile{
        fg: (64, 128, 208),
        bg: (0,0,0),
        tr:  Box::new( Wall4WayTileRender{
            connects: vec![1usize].drain(..).collect(),
            lower:('▄', false),
            upper:('▀', false),
            left:('▌', false),
            right:('▐', false),
            horizontal:('─', true),
            vertical:('│', true),
            misc:('■', false)
        } ),
        passable: false,
        opaque: true
    };
    let gt = Tile{
        fg: (89, 86, 85),
        bg: (0,0,0),
        tr:  Box::new( Wall4WayTileRender{
            connects: vec![1usize, 2usize].drain(..).collect(),
            lower:('▄', false),
            upper:('▀', false),
            left:('▌', false),
            right:('▐', false),
            horizontal:('▓', false),
            vertical:('▓', false),
            misc:(' ', true)
        } ),
        passable: false,
        opaque: true
    };
    let gf = Tile{
        fg: (133, 131, 130),
        bg: (0,0,0),
        tr: Box::new( FixedTileRender{ch: '*'} ),
        passable: true,
        opaque: false
    };

    let mg = MapGenerator{w: 128, h: 128};
    let m = mg.generate_map(
        vec![ bt, wt, gt, gf ]
    );

    let mut idx = 512;
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

    player.update_fov( &m );

    let mut open = vec![];
    for x in 1..127 {
        for y in 1..127 {
            if m.is_coord_passable((x,y)) {
                open.push((x,y));
            }
        }
    }

    gs.current_map = Some(m);
    gs.add_actor(player);

    for mut npc in npcs.drain(..) {
        let pos = open.remove( gs.gameplay_random.random_range(0..open.len()) );

        npc.position = (pos.0 as i32, pos.1 as i32);

        gs.add_actor(npc);
    }

    main_loop(context, gs)
}


