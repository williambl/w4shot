use crate::wasm4::{blit, blit_sub};

// eye
const EYE_WIDTH: u32 = 8;
const EYE_HEIGHT: u32 = 8;
const EYE_FLAGS: u32 = 1; // BLIT_2BPP
const EYE: [u8; 16] = [ 0x0f,0xf0,0x3a,0xac,0xe5,0x5b,0xd7,0xd7,0xd7,0xd7,0xe5,0x5b,0x3a,0xac,0x0f,0xf0 ];

// ship
const SHIP_WIDTH: u32 = 8;
const SHIP_HEIGHT: u32 = 8;
const SHIP_FLAGS: u32 = 1; // BLIT_2BPP
const SHIP: [u8; 16] = [ 0x02,0x80,0x09,0x60,0x09,0x60,0x2d,0x78,0x2d,0x78,0xbd,0x7e,0x8d,0x72,0x03,0xc0 ];

pub fn render_eye(x: i32, y: i32) {
    blit(&EYE, x, y, EYE_WIDTH, EYE_HEIGHT, EYE_FLAGS)
}

pub fn render_ship(x: i32, y: i32) {
    blit(&SHIP, x, y, SHIP_WIDTH, SHIP_HEIGHT, SHIP_FLAGS)
}