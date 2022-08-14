//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

// Painted Parchment 9 by skeddles
// https://lospec.com/palette-list/painted-parchment-9
pub const PAPER: u32 = 0xdda963;
pub const BROWN: u32 = 0xc9814b;
pub const BLACK: u32 = 0x25272a;
pub const WHITE: u32 = 0xdbc1af;
pub const RED: u32 = 0xcf6a4f;
pub const YELLOW: u32 = 0xe0b94a;
pub const GREEN: u32 = 0xb2af5c;
pub const BLUE: u32 = 0xa7a79e;
pub const PURPLE: u32 = 0x9b6970;

pub const DEFAULT: [u32; 4] = [PAPER, BROWN, BLACK, RED];
pub const SNOW: [u32; 4] = [PAPER, BROWN, BLACK, WHITE];
pub const GOLD: [u32; 4] = [PAPER, BROWN, BLACK, YELLOW];
pub const GRASS: [u32; 4] = [PAPER, BROWN, BLACK, GREEN];
pub const WATER: [u32; 4] = [PAPER, BROWN, BLACK, BLUE];
pub const WINE: [u32; 4] = [PAPER, BROWN, BLACK, PURPLE];

pub fn setup()
{
	unsafe { *PALETTE = DEFAULT };
}
