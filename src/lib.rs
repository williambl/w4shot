#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;

use std::cell::{Cell, RefCell, UnsafeCell};
use std::iter::Filter;
use std::slice::{Iter, IterMut};
use wasm4::*;
use crate::State::{Game, Lose, Menu};

#[rustfmt::skip]
const SMILEY: [u8; 8] = [
    0b11000011,
    0b10000001,
    0b00100100,
    0b00100100,
    0b00000000,
    0b00100100,
    0b10011001,
    0b11000011,
];

static mut STATE: State = Menu(MenuState { selected: 0, pressed: false });
static mut LAST_GAMEPAD: u8 = 0;

#[no_mangle]
fn update() {
    let gamepad = unsafe { *GAMEPAD1 };
    let last_gamepad = unsafe { LAST_GAMEPAD };
    let state = match unsafe { STATE } {
        Menu(state) => update_menu(state, gamepad, last_gamepad),
        Game(state) => update_game(state, gamepad, last_gamepad),
        Lose(state) => update_lose(state, gamepad, last_gamepad),
    };
    match state {
        Menu(state) => render_menu(state),
        Game(state) => render_game(state),
        Lose(state) => render_lose(state),
    }

    unsafe { STATE = state };
    unsafe { LAST_GAMEPAD = gamepad };
}

#[derive(Copy, Clone)]
enum State {
    Menu(MenuState),
    Game(GameState),
    Lose(LoseState)
}

#[derive(Copy, Clone)]
struct MenuState {
    selected: u8,
    pressed: bool,
}

fn update_menu(state: MenuState, gamepad: u8, last_gamepad: u8) -> State {
    let mut new_state = state;
    if gamepad & !last_gamepad & BUTTON_UP != 0 {
        new_state.selected = new_state.selected.wrapping_sub(1) % 2;
    }
    if gamepad & !last_gamepad & BUTTON_DOWN != 0 {
        new_state.selected = new_state.selected.wrapping_add(1) % 2;
    }
    if gamepad & BUTTON_1 != 0 {
        new_state.pressed = true;
    } else if last_gamepad & BUTTON_1 != 0 {
        if new_state.selected == 0 {
            return Game(create_game_state())
        } else {
            panic!()
        }
        new_state.pressed = false;
    }
    Menu(new_state)
}

fn render_menu(state: MenuState) {
    unsafe { *DRAW_COLORS = 0x0003 }
    text("w4-PLAT", 10, 10);
    unsafe { *DRAW_COLORS = if state.selected == 0 { if state.pressed { 0x0002 } else { 0x0004 } } else { 0x0003 } }
    text("Play", 10, 30);
    unsafe { *DRAW_COLORS = if state.selected == 1 { if state.pressed { 0x0002 } else { 0x0004 } } else { 0x0003 } }
    text("Exit", 10, 40);
}

#[derive(Copy, Clone)]
struct GameState {
    player_x: u8,
    player_y: u8,
    player_dx: i8,
    player_dy: i8,
    player_health: u8,
    time: u32,
    entities: [Entity; 64],
}

fn create_game_state() -> GameState {
    let mut state = GameState {
        player_x: 0,
        player_y: 0,
        player_dx: 0,
        player_dy: 0,
        player_health: 3,
        time: 0,
        entities: [EMPTY_ENTITY; 64],
    };
    for i in 0..8 {
        state.add_entity(Entity {
            x: (160f32*(i as f32/8f32)) as u8 - 4,
            y: 60,
            size: 8,
            dx: 0,
            dy: 0,
            age: 0,
            entity_type: EntityType::BasicEnemy,
        });
    }

    state
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Entity {
    x: u8,
    y: u8,
    size: u8,
    dx: i8,
    dy: i8,
    age: u16,
    entity_type: EntityType,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum EntityType {
    None,
    Bullet{
        player: bool
    },
    BasicEnemy,
}

const EMPTY_ENTITY: Entity = Entity {
    x: 0,
    y: 0,
    size: 0,
    dx: 0,
    dy: 0,
    age: 0,
    entity_type: EntityType::None,
};

enum GameEvent {
    PlayerHurt,
    PowerUp
}

struct ChangeRequests<'a> {
    entities_to_add: Vec<Entity>,
    entities_to_remove: Vec<& 'a Entity>,
    events: Vec<GameEvent>,
}

impl GameState {
    fn add_entity(&mut self, entity: Entity) -> bool {
        for existing_entity in self.entities.iter_mut() {
            if existing_entity.entity_type == EntityType::None {
                *existing_entity = entity;
                return true;
            }
        }
        false
    }

    fn update_movement_from_gamepad(&mut self, gamepad: u8) {
        if gamepad & BUTTON_UP != 0 {
            self.player_dy -= 1;
        }
        if gamepad & BUTTON_DOWN != 0 {
            self.player_dy += 1;
        }
        if gamepad & BUTTON_LEFT != 0 {
            self.player_dx -= 1;
        }
        if gamepad & BUTTON_RIGHT != 0 {
            self.player_dx += 1;
        }
    }

    fn update_player(&mut self, gamepad: u8, last_gamepad: u8) {
        self.player_dx = 0;
        self.player_dy = 0;

        self.update_movement_from_gamepad(gamepad);
        self.update_movement_from_gamepad(last_gamepad);

        self.player_x = if self.player_dx < 0 {
            self.player_x.saturating_sub(self.player_dx.saturating_abs() as u8)
        } else {
            self.player_x.saturating_add(self.player_dx as u8)
        }.clamp(0, 160);
        self.player_y = if self.player_dy < 0 {
            self.player_y.saturating_sub(self.player_dy.saturating_abs() as u8)
        } else {
            self.player_y.saturating_add(self.player_dy as u8)
        }.clamp(0, 160);

        if gamepad & BUTTON_1 & !(self.time % 20) as u8 != 0 {
            self.add_entity(Entity {
                x: self.player_x,
                y: self.player_y.saturating_sub(3),
                size: 1,
                dx: 0,
                dy: -3,
                age: 0,
                entity_type: EntityType::Bullet { player: true },
            });
        }
    }

    fn with_updated_entities(&self) -> GameState {
        let self_clone = &self.clone();
        let mut new_state = self.clone();
        let new_entities_and_change_requests: Vec<(Entity, ChangeRequests)> = new_state.entities.iter().map(|entity| entity.update(self_clone)).collect();
        let mut new_entities: Vec<Entity> = Vec::new();
        let mut change_requests: Vec<ChangeRequests> = Vec::new();
        for (entity, change_request) in new_entities_and_change_requests {
            new_entities.push(entity);
            change_requests.push(change_request);
        }
        new_state.entities = <[Entity; 64]>::try_from(new_entities).unwrap();

        for ChangeRequests { entities_to_add, entities_to_remove, events } in change_requests {
            for entity in entities_to_remove {
                for existing_entity in new_state.entities.iter_mut() {
                    if existing_entity == entity {
                        *existing_entity = EMPTY_ENTITY;
                        break;
                    }
                }
            }
            for entity in entities_to_add {
                new_state.add_entity(entity);
            }
            for event in events {
                match event {
                    GameEvent::PlayerHurt => new_state.player_health = new_state.player_health.saturating_sub(1),
                    GameEvent::PowerUp => new_state.player_health = new_state.player_health.saturating_add(1)
                };
            }
        }
        new_state
    }

    fn get_random(self) -> u32 {
        (self.time * 15417) ^ (self.time << 31) ^ ((self.time * 123651) >> 7)
    }
}

fn entity_collides_with_wall(entity: &Entity) -> bool {
    entity.x == 0 && entity.dx < 0 || entity.x == 160 && entity.dx > 0 || entity.y == 0 && entity.dy < 0 || entity.y == 160 && entity.dy > 0
}

fn collides(entity: &Entity, other_entity: &Entity) -> bool {
    entity.entity_type != EntityType::None && other_entity.entity_type != EntityType::None
    && entity.x - entity.size/2 < other_entity.x + other_entity.size/2
    && entity.x + entity.size/2 > other_entity.x - other_entity.size/2
    && entity.y - entity.size/2 < other_entity.y + other_entity.size/2
    && entity.y + entity.size/2 > other_entity.y - other_entity.size/2
}

fn collides_with_player(entity: &Entity, state: &GameState) -> bool {
    entity.entity_type != EntityType::None
    && entity.x - entity.size/2 < state.player_x + 4
    && entity.x + entity.size/2 > state.player_x - 4
    && entity.y - entity.size/2 < state.player_y + 4
    && entity.y + entity.size/2 > state.player_y - 4
}

impl Entity {
    fn update_movement(&mut self) {
        self.x = if self.dx < 0 {
            self.x.saturating_sub(self.dx.saturating_abs() as u8)
        } else {
            self.x.saturating_add(self.dx as u8)
        }.clamp(0, 160);
        self.y = if self.dy < 0 {
            self.y.saturating_sub(self.dy.saturating_abs() as u8)
        } else {
            self.y.saturating_add(self.dy as u8)
        }.clamp(0, 160);
    }

    fn update(self, state_snapshot: &GameState) -> (Entity, ChangeRequests) {
        let mut change_requests = ChangeRequests {
            entities_to_add: Vec::new(),
            entities_to_remove: Vec::new(),
            events: Vec::new(),
        };
        let mut new_entity = self.clone();
        new_entity.age += 1;
        match new_entity.entity_type {
            EntityType::None => {},
            EntityType::Bullet {player} => {
                new_entity.update_movement();
                if new_entity.age > 200 || entity_collides_with_wall(&new_entity) {
                    new_entity = EMPTY_ENTITY.clone();
                }
                if !player && collides_with_player(&new_entity, state_snapshot) {
                    new_entity = EMPTY_ENTITY.clone();
                    change_requests.events.push(GameEvent::PlayerHurt);
                }
            },
            EntityType::BasicEnemy => {
                if new_entity.age % 60 == 0 {
                    new_entity.dx = if state_snapshot.get_random() & 0x10 != 0 { 1 } else { -1 };
                    new_entity.dy = if state_snapshot.get_random() & 0x01 != 0 { 1 } else { -1 };
                } else if new_entity.age % 60 == 30 || entity_collides_with_wall(&new_entity) {
                    new_entity.dx = 0;
                    new_entity.dy = 0;
                }
                new_entity.update_movement();
                if new_entity.age % 60 == 0 {
                    change_requests.entities_to_add.push(Entity {
                        x: new_entity.x,
                        y: new_entity.y,
                        size: 1,
                        dx: 0,
                        dy: 1,
                        age: 0,
                        entity_type: EntityType::Bullet { player: false },
                    });
                }
                for entity in state_snapshot.entities.iter() {
                    if (entity.entity_type == EntityType::Bullet { player: true }) && collides(&new_entity, entity) {
                        new_entity = EMPTY_ENTITY.clone();
                        change_requests.entities_to_remove.push(entity);
                        break;
                    }
                }
            }
        }
        (new_entity, change_requests)
    }
}

fn update_game(state: GameState, gamepad: u8, last_gamepad: u8) -> State {
    let mut new_state = state;
    new_state.time += 1;
    new_state.update_player(gamepad, last_gamepad);
    new_state = new_state.with_updated_entities();

    if new_state.player_health == 0 {
        Lose(LoseState {
            score: new_state.time,
            pressed: false
        })
    } else {
        Game(new_state)
    }
}

fn render_entities(state: GameState) {
    state.entities.iter().for_each(|entity| match entity.entity_type {
        EntityType::None => { },
        EntityType::Bullet {..} => {
            unsafe { *DRAW_COLORS = 0x0004 }
            // draw rect of size entity.size
            let half_size = (entity.size / 2) as i32;
            rect((entity.x as i32 - half_size), (entity.y as i32 - half_size), entity.size as u32, entity.size as u32);
        },
        EntityType::BasicEnemy => {
            unsafe { *DRAW_COLORS = 0x0003 }
            let half_size = (entity.size / 2) as i32;
            rect((entity.x as i32 - half_size), (entity.y as i32 - half_size), entity.size as u32, entity.size as u32);
        }
    });
}

fn render_game(state: GameState) {
    unsafe { *DRAW_COLORS = 0x0004 }
    rect((state.player_x as i32 - 4), (state.player_y as i32 - 4), 8, 8);
    text(format!("Health: {}", state.player_health).as_str(), 0, 0);
    render_entities(state);
}

#[derive(Copy, Clone)]
struct LoseState {
    score: u32,
    pressed: bool,
}

fn update_lose(state: LoseState, gamepad: u8, last_gamepad: u8) -> State {
    let mut new_state = state;
    if state.pressed && gamepad & BUTTON_1 == 0 {
        Menu(MenuState {
            selected: 0,
            pressed: false,
        })
    } else {
        if gamepad & BUTTON_1 != 0 {
            new_state.pressed = true;
        }

        Lose(new_state)
    }
}

fn render_lose(state: LoseState) {
    unsafe { *DRAW_COLORS = 0x0003 }
    text("GAME OVER", 10, 10);
    text(format!("Score: {}", state.score).as_str(), 10, 20);

    unsafe { *DRAW_COLORS = if state.pressed { 0x0002 } else { 0x0004 } }
    text("Press X to return", 10, 70);
    text("to Main Menu", 10, 80);
}
