#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
use wasm4::*;
use crate::GameState::{Game, Menu};

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

static mut STATE: GameState = Menu(MenuState { selected: 0, pressed: false });
static mut LAST_GAMEPAD: u8 = 0;

#[no_mangle]
fn update() {
    let gamepad = unsafe { *GAMEPAD1 };
    let last_gamepad = unsafe { LAST_GAMEPAD };
    let state = match unsafe { STATE } {
        Menu(state) => update_menu(state, gamepad, last_gamepad),
        Game => Game,
    };
    match state {
        Menu(state) => render_menu(state),
        Game => {},
    }

    unsafe { STATE = state };
    unsafe { LAST_GAMEPAD = gamepad };
}

#[derive(Copy, Clone)]
enum GameState {
    Menu(MenuState),
    Game,
}

#[derive(Copy, Clone)]
struct MenuState {
    selected: u8,
    pressed: bool,
}

fn update_menu(state: MenuState, gamepad: u8, last_gamepad: u8) -> GameState {
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
            return Game
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
