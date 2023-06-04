use crate::menu::MenuState;
use crate::State;
use crate::State::{Lose, Menu};
use crate::wasm4::{BUTTON_1, DRAW_COLORS, text};

#[derive(Copy, Clone)]
pub struct LoseState {
    score: u32,
    pressed: bool,
}

impl LoseState {
    pub fn new(score: u32) -> Self {
        Self {
            score,
            pressed: false,
        }
    }
}

pub fn update_lose(state: LoseState, gamepad: u8, last_gamepad: u8) -> State {
    let mut new_state = state;
    if state.pressed && gamepad & BUTTON_1 == 0 {
        Menu(MenuState::new())
    } else {
        if gamepad & BUTTON_1 != 0 {
            new_state.pressed = true;
        }

        Lose(new_state)
    }
}

pub fn render_lose(state: LoseState) {
    unsafe { *DRAW_COLORS = 0x0003 }
    text("GAME OVER", 10, 10);
    text(format!("Score: {}", state.score).as_str(), 10, 20);

    unsafe { *DRAW_COLORS = if state.pressed { 0x0002 } else { 0x0004 } }
    text("Press X to return", 10, 70);
    text("to Main Menu", 10, 80);
}
