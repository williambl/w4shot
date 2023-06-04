#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
mod menu;
mod game;
mod lose;

use std::cell::{Cell, RefCell, UnsafeCell};
use std::iter::Filter;
use std::slice::{Iter, IterMut};
use game::GameState;
use lose::LoseState;
use menu::MenuState;
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

static mut STATE: State = Menu(MenuState::new());
static mut LAST_GAMEPAD: u8 = 0;

#[no_mangle]
fn update() {
    let gamepad = unsafe { *GAMEPAD1 };
    let last_gamepad = unsafe { LAST_GAMEPAD };
    let state = match unsafe { STATE } {
        Menu(state) => menu::update_menu(state, gamepad, last_gamepad),
        Game(state) => game::update_game(state, gamepad, last_gamepad),
        Lose(state) => lose::update_lose(state, gamepad, last_gamepad),
    };
    match state {
        Menu(state) => menu::render_menu(state),
        Game(state) => game::render_game(state),
        Lose(state) => lose::render_lose(state),
    }

    unsafe { STATE = state };
    unsafe { LAST_GAMEPAD = gamepad };
}

#[derive(Copy, Clone)]
pub enum State {
    Menu(MenuState),
    Game(GameState),
    Lose(LoseState)
}
