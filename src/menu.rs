use crate::game::GameState;
use crate::sprites::{render_eye, render_powerup, render_ship};
use crate::State;
use crate::State::{Game, Menu};
use crate::wasm4::{BUTTON_1, BUTTON_DOWN, BUTTON_UP, DRAW_COLORS, text};

#[derive(Copy, Clone)]
pub struct MenuState {
    selected: u8,
    pressed: bool,
    difficulty: Difficulty
}

#[derive(Copy, Clone)]
enum Difficulty {
    Boring,
    Easy,
    Normal,
    Hard,
    Insane
}

impl Difficulty {
    fn next(&self) -> Self {
        match self {
            Difficulty::Boring => Difficulty::Easy,
            Difficulty::Easy => Difficulty::Normal,
            Difficulty::Normal => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Insane,
            Difficulty::Insane => Difficulty::Boring
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Difficulty::Boring => "Boring",
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
            Difficulty::Insane => "INSANE"
        }
    }

    fn to_difficulty_level(&self) -> u8 {
        match self {
            Difficulty::Boring => 3,
            Difficulty::Easy => 5,
            Difficulty::Normal => 7,
            Difficulty::Hard => 9,
            Difficulty::Insane => 10
        }
    }
}

impl MenuState {
    pub const fn new() -> Self {
        Self {
            selected: 0,
            pressed: false,
            difficulty: Difficulty::Normal
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
            return Game(GameState::new(new_state.difficulty.to_difficulty_level()))
        } else if new_state.selected == 1 {
            new_state.difficulty = new_state.difficulty.next();
        }
        new_state.pressed = false;
    }
    Menu(new_state)
}

pub fn render_menu(state: MenuState) {
    unsafe { *DRAW_COLORS = 0x0003 }
    text("W4-SHOT", 10, 10);
    unsafe { *DRAW_COLORS = if state.selected == 0 { if state.pressed { 0x0002 } else { 0x0004 } } else { 0x0003 } }
    text("Play", 10, 30);
    unsafe { *DRAW_COLORS = 0x0003 }
    text("Difficulty: ", 10, 40);
    unsafe { *DRAW_COLORS = if state.selected == 1 { if state.pressed { 0x0002 } else { 0x0004 } } else { 0x0003 } }
    text(state.difficulty.to_str(), 17, 50);

    unsafe { *DRAW_COLORS = 0x0003 }
    text("The enemy:", 40, 80);
    text("      You:", 40, 100);
    text("  Powerup:", 40, 120);
    unsafe { *DRAW_COLORS = 0x2430 }
    render_ship(130, 100);
    unsafe { *DRAW_COLORS = 0x0432 }
    render_eye(130, 80);
    render_powerup(130, 120);
}
