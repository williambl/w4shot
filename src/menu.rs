use crate::game::GameState;
use crate::State;
use crate::State::{Game, Menu};
use crate::wasm4::{BUTTON_1, BUTTON_DOWN, BUTTON_UP, DRAW_COLORS, text};

#[derive(Copy, Clone)]
pub struct MenuState {
    selected: u8,
    pressed: bool,
}

impl MenuState {
    pub const fn new() -> Self {
        Self {
            selected: 0,
            pressed: false,
        }
    }
}

pub fn update_menu(state: MenuState, gamepad: u8, last_gamepad: u8) -> State {
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
            return Game(GameState::new())
        } else {
            panic!()
        }
        new_state.pressed = false;
    }
    Menu(new_state)
}

pub fn render_menu(state: MenuState) {
    unsafe { *DRAW_COLORS = 0x0003 }
    text("w4-PLAT", 10, 10);
    unsafe { *DRAW_COLORS = if state.selected == 0 { if state.pressed { 0x0002 } else { 0x0004 } } else { 0x0003 } }
    text("Play", 10, 30);
    unsafe { *DRAW_COLORS = if state.selected == 1 { if state.pressed { 0x0002 } else { 0x0004 } } else { 0x0003 } }
    text("Exit", 10, 40);
}
