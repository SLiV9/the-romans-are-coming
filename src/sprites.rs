//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

pub fn draw_surface(x: i32, y: i32, alt: u8)
{
	let frame = (alt as u32) % SURFACE_FRAMES;
	blit_sub(
		&SURFACE_SHEET,
		x + 1 - (SURFACE_WIDTH as i32) / 2,
		y + 1 - (SURFACE_HEIGHT as i32),
		SURFACE_WIDTH,
		SURFACE_HEIGHT,
		frame * SURFACE_WIDTH,
		0,
		SURFACE_SHEET_WIDTH,
		SURFACE_SHEET_FLAGS,
	);
}

const SURFACE_WIDTH: u32 = 8;
const SURFACE_HEIGHT: u32 = SURFACE_SHEET_HEIGHT;
const SURFACE_FRAMES: u32 = SURFACE_SHEET_WIDTH / SURFACE_WIDTH;

// surface_sheet
const SURFACE_SHEET_WIDTH: u32 = 32;
const SURFACE_SHEET_HEIGHT: u32 = 4;
const SURFACE_SHEET_FLAGS: u32 = 0; // BLIT_1BPP
const SURFACE_SHEET: [u8; 16] = [
	0x3e, 0x3f, 0x3e, 0x3f, 0xfc, 0xfc, 0x7c, 0x7c, 0x3f, 0x3e, 0x3f, 0x3e,
	0x7c, 0x7c, 0xfc, 0xfc,
];

pub fn draw_grass(x: i32, y: i32, alt: u8)
{
	let frame = (alt as u32) % GRASS_FRAMES;
	blit_sub(
		&GRASS_SHEET,
		x + 1 - (GRASS_WIDTH as i32) / 2,
		y + 1 - (GRASS_HEIGHT as i32),
		GRASS_WIDTH,
		GRASS_HEIGHT,
		frame * GRASS_WIDTH,
		0,
		GRASS_SHEET_WIDTH,
		GRASS_SHEET_FLAGS,
	);
}

const GRASS_WIDTH: u32 = 4;
const GRASS_HEIGHT: u32 = GRASS_SHEET_HEIGHT;
const GRASS_FRAMES: u32 = GRASS_SHEET_WIDTH / GRASS_WIDTH;

// grass_sheet
const GRASS_SHEET_WIDTH: u32 = 28;
const GRASS_SHEET_HEIGHT: u32 = 2;
const GRASS_SHEET_FLAGS: u32 = 1; // BLIT_2BPP
const GRASS_SHEET: [u8; 14] = [
	0x10, 0x40, 0x28, 0x41, 0x01, 0x41, 0x04, 0x11, 0x44, 0xbe, 0x11, 0x11,
	0x44, 0x44,
];

pub fn draw_tree(x: i32, y: i32, alt: u8)
{
	let frame = (alt as u32) % TREE_FRAMES;
	blit_sub(
		&TREE_SHEET,
		x + 1 - (TREE_WIDTH as i32) / 2,
		y + 1 - (TREE_HEIGHT as i32),
		TREE_WIDTH,
		TREE_HEIGHT,
		frame * TREE_WIDTH,
		0,
		TREE_SHEET_WIDTH,
		TREE_SHEET_FLAGS,
	);
}

const TREE_WIDTH: u32 = 8;
const TREE_HEIGHT: u32 = TREE_SHEET_HEIGHT;
const TREE_FRAMES: u32 = TREE_SHEET_WIDTH / TREE_WIDTH;

// tree_sheet
const TREE_SHEET_WIDTH: u32 = 8;
const TREE_SHEET_HEIGHT: u32 = 8;
const TREE_SHEET_FLAGS: u32 = 1; // BLIT_2BPP
const TREE_SHEET: [u8; 16] = [
	0x02, 0x00, 0x0b, 0x80, 0x2f, 0xe0, 0x2f, 0xe0, 0x2f, 0xe0, 0x0a, 0x80,
	0x16, 0x50, 0x05, 0x40,
];

pub fn draw_hill(x: i32, y: i32, alt: u8)
{
	let frame = (alt as u32) % HILL_FRAMES;
	blit_sub(
		&HILL_SHEET,
		x + 1 - (HILL_WIDTH as i32) / 2,
		y + 1 - (HILL_HEIGHT as i32),
		HILL_WIDTH,
		HILL_HEIGHT,
		frame * HILL_WIDTH,
		0,
		HILL_SHEET_WIDTH,
		HILL_SHEET_FLAGS,
	);
}

const HILL_WIDTH: u32 = 12;
const HILL_HEIGHT: u32 = HILL_SHEET_HEIGHT;
const HILL_FRAMES: u32 = HILL_SHEET_WIDTH / HILL_WIDTH;

// hill_sheet
const HILL_SHEET_WIDTH: u32 = 12;
const HILL_SHEET_HEIGHT: u32 = 4;
const HILL_SHEET_FLAGS: u32 = 1; // BLIT_2BPP
const HILL_SHEET: [u8; 12] = [
	0x02, 0x80, 0xa0, 0x0b, 0xe6, 0xf8, 0x2f, 0xf9, 0x40, 0x00, 0x00, 0x00,
];

pub fn draw_mountain(x: i32, y: i32, alt: u8)
{
	let frame = (alt as u32) % MOUNTAIN_FRAMES;
	blit_sub(
		&MOUNTAIN_SHEET,
		x + 1 - (MOUNTAIN_WIDTH as i32) / 2,
		y + 1 - (MOUNTAIN_HEIGHT as i32),
		MOUNTAIN_WIDTH,
		MOUNTAIN_HEIGHT,
		frame * MOUNTAIN_WIDTH,
		0,
		MOUNTAIN_SHEET_WIDTH,
		MOUNTAIN_SHEET_FLAGS,
	);
}

const MOUNTAIN_WIDTH: u32 = 12;
const MOUNTAIN_HEIGHT: u32 = MOUNTAIN_SHEET_HEIGHT;
const MOUNTAIN_FRAMES: u32 = MOUNTAIN_SHEET_WIDTH / MOUNTAIN_WIDTH;

// mountain_sheet
const MOUNTAIN_SHEET_WIDTH: u32 = 36;
const MOUNTAIN_SHEET_HEIGHT: u32 = 12;
const MOUNTAIN_SHEET_FLAGS: u32 = 1; // BLIT_2BPP
const MOUNTAIN_SHEET: [u8; 108] = [
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x20, 0x00,
	0x00, 0x20, 0x00, 0x00, 0xa8, 0x00, 0x00, 0xa8, 0x00, 0x00, 0xa8, 0x00,
	0x02, 0xbe, 0x00, 0x02, 0xbe, 0x00, 0x02, 0xbe, 0x00, 0x0a, 0x7f, 0x80,
	0x0a, 0x5f, 0x80, 0x0a, 0x7f, 0x80, 0x2a, 0xdf, 0xe0, 0x2a, 0x74, 0xe0,
	0x2a, 0x5c, 0xe0, 0xaa, 0x74, 0x38, 0x02, 0x9c, 0x00, 0x02, 0x94, 0x00,
	0x0a, 0x9f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub fn draw_boat(x: i32, y: i32, alt: u8)
{
	let frame = (alt as u32) % BOAT_FRAMES;
	blit_sub(
		&BOAT_SHEET,
		x + 1 - (BOAT_WIDTH as i32) / 2,
		y + 1 - (BOAT_HEIGHT as i32),
		BOAT_WIDTH,
		BOAT_HEIGHT,
		frame * BOAT_WIDTH,
		0,
		BOAT_SHEET_WIDTH,
		BOAT_SHEET_FLAGS,
	);
}

const BOAT_WIDTH: u32 = 4;
const BOAT_HEIGHT: u32 = BOAT_SHEET_HEIGHT;
const BOAT_FRAMES: u32 = BOAT_SHEET_WIDTH / BOAT_WIDTH;

// boat_sheet
const BOAT_SHEET_WIDTH: u32 = 8;
const BOAT_SHEET_HEIGHT: u32 = 4;
const BOAT_SHEET_FLAGS: u32 = 1; // BLIT_2BPP
const BOAT_SHEET: [u8; 8] = [0x30, 0x0c, 0x3c, 0x3c, 0xaa, 0xaa, 0x14, 0x14];
