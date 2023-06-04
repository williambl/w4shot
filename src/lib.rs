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
            return Game(GameState {
                player_x: 0,
                player_y: 0,
                player_dx: 0,
                player_dy: 0,
                time: 0,
            })
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
}

fn update_movement_from_gamepad(gamepad: u8, state: GameState) -> GameState {
    let mut new_state = state;
    if gamepad & BUTTON_UP != 0 {
        new_state.player_dy -= 1;
    }
    if gamepad & BUTTON_DOWN != 0 {
        new_state.player_dy += 1;
    }
    if gamepad & BUTTON_LEFT != 0 {
        new_state.player_dx -= 1;
    }
    if gamepad & BUTTON_RIGHT != 0 {
        new_state.player_dx += 1;
    }
    new_state
}

fn update_movement(mut state: GameState, gamepad: u8, last_gamepad: u8) -> GameState {
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

    state
}

fn update_game(state: GameState, gamepad: u8, last_gamepad: u8) -> State {
    let mut new_state = state;
    new_state.time += 1;
    new_state = update_movement(new_state, gamepad, last_gamepad);

    Game(new_state)
}

fn render_game(state: GameState) {
    unsafe { *DRAW_COLORS = 0x0004 }
    rect((state.player_x as i32 - 4), (state.player_y as i32 - 4), 8, 8);
}

