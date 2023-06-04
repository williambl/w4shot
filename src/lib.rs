#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
use wasm4::*;
use crate::State::{Game, Menu};

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
    };
    match state {
        Menu(state) => render_menu(state),
        Game(state) => render_game(state),
    }

    unsafe { STATE = state };
    unsafe { LAST_GAMEPAD = gamepad };
}

#[derive(Copy, Clone)]
enum State {
    Menu(MenuState),
    Game(GameState),
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
    time: u32,
    entities: [Entity; 64],
}

fn create_game_state() -> GameState {
    GameState {
        player_x: 0,
        player_y: 0,
        player_dx: 0,
        player_dy: 0,
        time: 0,
        entities: [EMPTY_ENTITY; 64],
    }
}

#[derive(Copy, Clone)]
struct Entity {
    x: u8,
    y: u8,
    dx: i8,
    dy: i8,
    age: u16,
    entity_type: EntityType,
}

#[derive(Copy, Clone, PartialEq)]
enum EntityType {
    None,
    Bullet,
}

const EMPTY_ENTITY: Entity = Entity {
    x: 0,
    y: 0,
    dx: 0,
    dy: 0,
    age: 0,
    entity_type: EntityType::None,
};

fn add_entity(mut state: GameState, entity: Entity) -> GameState {
    for existing_entity in state.entities.iter_mut() {
        if existing_entity.entity_type == EntityType::None {
            *existing_entity = entity;
            return state;
        }
    }
    state
}

fn update_movement_from_gamepad(gamepad: u8, mut state: GameState) -> GameState {
    if gamepad & BUTTON_UP != 0 {
        state.player_dy -= 1;
    }
    if gamepad & BUTTON_DOWN != 0 {
        state.player_dy += 1;
    }
    if gamepad & BUTTON_LEFT != 0 {
        state.player_dx -= 1;
    }
    if gamepad & BUTTON_RIGHT != 0 {
        state.player_dx += 1;
    }
    state
}

fn update_player(mut state: GameState, gamepad: u8, last_gamepad: u8) -> GameState {
    state.player_dx = 0;
    state.player_dy = 0;

    state = update_movement_from_gamepad(gamepad, state);
    state = update_movement_from_gamepad(last_gamepad, state);

    state.player_x = if state.player_dx < 0 {
        state.player_x.saturating_sub(state.player_dx.saturating_abs() as u8)
    } else {
        state.player_x.saturating_add(state.player_dx as u8)
    }.clamp(0, 160);
    state.player_y = if state.player_dy < 0 {
        state.player_y.saturating_sub(state.player_dy.saturating_abs() as u8)
    } else {
        state.player_y.saturating_add(state.player_dy as u8)
    }.clamp(0, 160);

    if gamepad & BUTTON_1 & !(state.time % 20) as u8 != 0 {
        state = add_entity(state, Entity {
            x: state.player_x,
            y: state.player_y.saturating_sub(3),
            dx: 0,
            dy: -3,
            age: 0,
            entity_type: EntityType::Bullet,
        });
    }

    state
}

fn entity_collides_with_wall(entity: Entity) -> bool {
    entity.x == 0 && entity.dx < 0 || entity.x == 160 && entity.dx > 0 || entity.y == 0 && entity.dy < 0 || entity.y == 160 && entity.dy > 0
}

fn update_entity_movement(mut entity: Entity) -> Entity {
    entity.x = if entity.dx < 0 {
        entity.x.saturating_sub(entity.dx.saturating_abs() as u8)
    } else {
        entity.x.saturating_add(entity.dx as u8)
    }.clamp(0, 160);
    entity.y = if entity.dy < 0 {
        entity.y.saturating_sub(entity.dy.saturating_abs() as u8)
    } else {
        entity.y.saturating_add(entity.dy as u8)
    }.clamp(0, 160);
    entity
}

fn update_entity(mut entity: Entity) -> Entity {
    entity.age += 1;
    match entity.entity_type {
        EntityType::None => { },
        EntityType::Bullet => {
            if entity.age > 60 || entity_collides_with_wall(entity) {
                entity.entity_type = EntityType::None;
            }
        },
    }
    entity = update_entity_movement(entity);
    entity
}

fn update_entities(mut state: GameState) -> GameState {
    state.entities = state.entities.map(|entity| {
        if entity.entity_type == EntityType::None {
            entity
        } else {
            update_entity(entity)
        }
    });
    state
}

fn update_game(state: GameState, gamepad: u8, last_gamepad: u8) -> State {
    let mut new_state = state;
    new_state.time += 1;
    new_state = update_player(new_state, gamepad, last_gamepad);
    new_state = update_entities(new_state);

    Game(new_state)
}

fn render_entities(state: GameState) {
    unsafe { *DRAW_COLORS = 0x0004 }
    state.entities.iter().for_each(|entity| match entity.entity_type {
        EntityType::None => { },
        EntityType::Bullet => {
            rect((entity.x as i32 - 1), (entity.y as i32 - 1), 2, 2);
        },
    });
}

fn render_game(state: GameState) {
    unsafe { *DRAW_COLORS = 0x0004 }
    rect((state.player_x as i32 - 4), (state.player_y as i32 - 4), 8, 8);
    render_entities(state);
}

