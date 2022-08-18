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
pub const YELLOW: u32 = 0xe0b94a;
pub const GREEN: u32 = 0x9cb35d;
pub const PURPLE: u32 = 0x9b6970;
pub const RED: u32 = 0xda995e;

pub const ALT_PAPER: u32 = 0xd3b288;
pub const ALT_BROWN: u32 = 0xbc9171;
pub const ALT_BLACK: u32 = 0x25272a;
pub const ALT_RED: u32 = 0xba543d;

pub const DEFAULT: [u32; 4] = [PAPER, BROWN, BLACK, PAPER];
pub const SNOW: [u32; 4] = [PAPER, BROWN, BLACK, WHITE];
pub const BLOOD: [u32; 4] = [PAPER, BROWN, BLACK, RED];
pub const GOLD: [u32; 4] = [PAPER, BROWN, BLACK, YELLOW];
pub const NATURE: [u32; 4] = [PAPER, BROWN, BLACK, GREEN];
pub const WINE: [u32; 4] = [PAPER, BROWN, BLACK, PURPLE];
pub const ROMAN: [u32; 4] = [ALT_PAPER, ALT_BROWN, ALT_BLACK, ALT_RED];

pub fn setup()
{
	unsafe { *PALETTE = DEFAULT };
}
