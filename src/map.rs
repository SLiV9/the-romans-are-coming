//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use fastrand;
use perlin2d::PerlinNoise2D;

pub const MAP_SIZE: usize = 180;
pub const BITMAP_SIZE: usize = MAP_SIZE * MAP_SIZE / 8;
pub const QUADMAP_SIZE: usize = MAP_SIZE * MAP_SIZE / 4;
pub const GRID_SIZE: usize = 7;
pub const GRID_CELL_SIZE: usize = MAP_SIZE / GRID_SIZE;

const NOISE_OCTAVES: i32 = 5;
const NOISE_AMPLITUDE: f64 = 50.0;
const NOISE_FREQUENCY: f64 = 1.0;
const NOISE_FREQUENCY_FOREST: f64 = 1.0;
const NOISE_PERSISTENCE: f64 = 1.0;
const NOISE_PERSISTENCE_FOREST: f64 = 2.0;
const NOISE_LACUNARITY: f64 = 2.0;
const NOISE_SCALE: (f64, f64) = (MAP_SIZE as f64, MAP_SIZE as f64);
const NOISE_BIAS_ELEVATION: f64 = 45.0;
const NOISE_BIAS_FOREST: f64 = -20.0;

const CENTROID_FIX_RADIUS: f64 = 3.0;
const CENTROID_SPREAD_RADIUS: f64 = 6.0;

pub struct Map
{
	bitmap: [u8; BITMAP_SIZE],
	region_bitmap: [u8; BITMAP_SIZE],
	centroid_bitmap: [u8; BITMAP_SIZE],
	hit_quadmap: [u8; QUADMAP_SIZE],
	cells: [[Cell; GRID_SIZE]; GRID_SIZE],
}

#[derive(Debug, Clone, Copy, Default)]
struct Cell
{
	centroid_x: u8,
	centroid_y: u8,
	elevation: f32,
	forest: f32,
	size: u16,
}

const EMPTY_CELL: Cell = Cell {
	centroid_x: 0,
	centroid_y: 0,
	elevation: 0.0,
	forest: 0.0,
	size: 0,
};

impl Map
{
	pub const fn empty() -> Self
	{
		Self {
			bitmap: [0; BITMAP_SIZE],
			region_bitmap: [0; BITMAP_SIZE],
			centroid_bitmap: [0; BITMAP_SIZE],
			hit_quadmap: [0; QUADMAP_SIZE],
			cells: [[EMPTY_CELL; GRID_SIZE]; GRID_SIZE],
		}
	}

	pub fn generate(&mut self, rng: &mut fastrand::Rng)
	{
		let seed = rng.u16(..) as i32;
		let elevation = PerlinNoise2D::new(
			NOISE_OCTAVES,
			NOISE_AMPLITUDE,
			NOISE_FREQUENCY,
			NOISE_PERSISTENCE,
			NOISE_LACUNARITY,
			NOISE_SCALE,
			NOISE_BIAS_ELEVATION,
			seed,
		);
		let seed = rng.u16(..) as i32;
		let forest = PerlinNoise2D::new(
			NOISE_OCTAVES,
			NOISE_AMPLITUDE,
			NOISE_FREQUENCY_FOREST,
			NOISE_PERSISTENCE_FOREST,
			NOISE_LACUNARITY,
			NOISE_SCALE,
			NOISE_BIAS_FOREST,
			seed,
		);
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				let (x, y) = pick_random_centroid_xy_at_rc(r, c, rng);
				cell.centroid_x = x as u8;
				cell.centroid_y = y as u8;
				cell.elevation = 0.0;
				cell.forest = 0.0;
				cell.size = 0;
			}
		}
		for y in 0..MAP_SIZE
		{
			for x in 0..MAP_SIZE
			{
				let (r, c, distance) =
					self.closest_rc_to_xy(x as i32, y as i32);
				let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let f = forest.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				self.cells[r][c].elevation += e as f32;
				self.cells[r][c].forest += f as f32;
				self.cells[r][c].size += 1;
				if e > 70.0
				{
					draw_on_bitmap(&mut self.bitmap, x, y);
				}
				else if e > 50.0
				{
					erase_on_bitmap(&mut self.bitmap, x, y);
				}
				else if e > 0.0
				{
					draw_on_bitmap(&mut self.bitmap, x, y);
				}
				else
				{
					erase_on_bitmap(&mut self.bitmap, x, y);
				}
				if f > 0.0
				{
					draw_on_bitmap(&mut self.region_bitmap, x, y);
				}
				else
				{
					erase_on_bitmap(&mut self.region_bitmap, x, y);
				}
			}
		}
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				cell.elevation = cell.elevation / (cell.size as f32);
				cell.forest = cell.forest / (cell.size as f32);
				let x = cell.centroid_x as usize;
				let y = cell.centroid_y as usize;
				let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let f = forest.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let mut badness = (e as f32 - cell.elevation).abs()
					+ (f as f32 - cell.forest).abs();
				for _i in 0..10
				{
					if badness < 5.0
					{
						break;
					}
					let (x, y) = pick_random_centroid_xy_at_rc(r, c, rng);
					let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
					let f = forest.get_noise(x as f64 + 0.5, y as f64 + 0.5);
					let b = (e as f32 - cell.elevation).abs()
						+ (f as f32 - cell.forest).abs();
					if b + 5.0 < badness
					{
						cell.centroid_x = x as u8;
						cell.centroid_y = y as u8;
						badness = b;
					}
				}
				{
					let x = cell.centroid_x as usize;
					let y = cell.centroid_y as usize;
					draw_on_bitmap(&mut self.centroid_bitmap, x, y);
				}
			}
		}
	}

	pub fn draw(&self)
	{
		let x = -(GRID_CELL_SIZE as i32) / 2;
		let y = x;
		unsafe { *DRAW_COLORS = 0x20 };
		blit(
			&self.region_bitmap,
			x,
			y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x04 };
		blit(
			&self.bitmap,
			x,
			y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x30 };
		blit(
			&self.centroid_bitmap,
			x,
			y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
	}

	fn closest_rc_to_xy(&self, x: i32, y: i32) -> (usize, usize, f64)
	{
		let xx = std::cmp::max(0, x) as usize;
		let yy = std::cmp::max(0, y) as usize;
		let r0 = std::cmp::min(yy / GRID_CELL_SIZE, GRID_SIZE - 2);
		let c0 = std::cmp::min(xx / GRID_CELL_SIZE, GRID_SIZE - 2);
		(0..4)
			.map(|i| {
				let r = r0 + 1 - i / 2;
				let c = c0 + 1 - i % 2;
				let cell = &self.cells[r][c];
				let ddx = cell.centroid_x as i32 - x;
				let ddy = cell.centroid_y as i32 - y;
				let sqdis: i32 = ddx * ddx + ddy * ddy;
				(r, c, sqdis)
			})
			.min_by_key(|(_r, _c, sqdis)| *sqdis)
			.map(|(r, c, sqdis)| (r, c, (sqdis as f64).sqrt()))
			.unwrap()
	}
}

fn draw_on_bitmap(bitmap: &mut [u8; BITMAP_SIZE], x: usize, y: usize)
{
	let offset = y * MAP_SIZE + x;
	let byte_offset = offset / 8;
	assert!(byte_offset < BITMAP_SIZE);
	let bit_shift = offset % 8;
	bitmap[byte_offset] |= 0b10000000 >> bit_shift;
}

fn erase_on_bitmap(bitmap: &mut [u8; BITMAP_SIZE], x: usize, y: usize)
{
	let offset = y * MAP_SIZE + x;
	let byte_offset = offset / 8;
	assert!(byte_offset < BITMAP_SIZE);
	let bit_shift = offset % 8;
	bitmap[byte_offset] &= !(0b10000000 >> bit_shift);
}

fn draw_on_quadmap(
	quadmap: &mut [u8; QUADMAP_SIZE],
	x: usize,
	y: usize,
	value: u8,
)
{
	let offset = y * MAP_SIZE + x;
	let byte_offset = offset / 4;
	assert!(byte_offset < QUADMAP_SIZE);
	let bit_shift = 2 * (offset % 4);
	quadmap[byte_offset] &= !(0b11000000 >> bit_shift);
	let value = value & 0b11;
	quadmap[byte_offset] |= (value << 6) >> bit_shift;
}

fn pick_random_centroid_xy_at_rc(
	r: usize,
	c: usize,
	rng: &mut fastrand::Rng,
) -> (usize, usize)
{
	let padding = 3;
	let inner_x = padding + rng.usize(0..(GRID_CELL_SIZE - 2 * padding));
	let inner_y = padding + rng.usize(0..(GRID_CELL_SIZE - 2 * padding));
	let x = c * GRID_CELL_SIZE + inner_x;
	let y = r * GRID_CELL_SIZE + inner_y;
	let x = std::cmp::min(
		std::cmp::max(GRID_CELL_SIZE / 2 + padding, x),
		MAP_SIZE - 1 - padding - GRID_CELL_SIZE / 2,
	);
	let y = std::cmp::min(
		std::cmp::max(GRID_CELL_SIZE / 2 + padding, y),
		MAP_SIZE - 1 - padding - GRID_CELL_SIZE / 2,
	);
	(x, y)
}
