//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use fastrand;
use perlin2d::PerlinNoise2D;

pub const MAP_SIZE: usize = 160;
pub const BITMAP_SIZE: usize = MAP_SIZE * MAP_SIZE / 8;
pub const QUADMAP_SIZE: usize = MAP_SIZE * MAP_SIZE / 4;
pub const GRID_SIZE: usize = 16;
pub const GRID_CELL_SIZE: usize = MAP_SIZE / GRID_SIZE;

const NOISE_OCTAVES: i32 = 4;
const NOISE_AMPLITUDE: f64 = 50.0;
const NOISE_FREQUENCY: f64 = 3.0;
const NOISE_PERSISTENCE: f64 = 1.0;
const NOISE_LACUNARITY: f64 = 2.0;
const NOISE_SCALE: (f64, f64) = (MAP_SIZE as f64, MAP_SIZE as f64);
const NOISE_BIAS: f64 = 45.0;

pub struct Map
{
	bitmap: [u8; BITMAP_SIZE],
	region_bitmap: [u8; BITMAP_SIZE],
	centroid_bitmap: [u8; BITMAP_SIZE],
	hit_quadmap: [u8; QUADMAP_SIZE],
}

impl Map
{
	pub const fn empty() -> Self
	{
		Self {
			bitmap: [0; BITMAP_SIZE],
			region_bitmap: [0; BITMAP_SIZE],
			centroid_bitmap: [0; BITMAP_SIZE],
			hit_quadmap: [0; QUADMAP_SIZE],
		}
	}

	pub fn generate(&mut self, rng: &mut fastrand::Rng)
	{
		let noise_seed = rng.u16(..) as i32;
		let noise = PerlinNoise2D::new(
			NOISE_OCTAVES,
			NOISE_AMPLITUDE,
			NOISE_FREQUENCY,
			NOISE_PERSISTENCE,
			NOISE_LACUNARITY,
			NOISE_SCALE,
			NOISE_BIAS,
			noise_seed,
		);
		for y in 0..MAP_SIZE
		{
			for x in 0..MAP_SIZE
			{
				let elevation = noise.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				if elevation > 0.0
				{
					draw_on_bitmap(&mut self.bitmap, x, y);
				}
				else
				{
					erase_on_bitmap(&mut self.bitmap, x, y);
				}
				erase_on_bitmap(&mut self.region_bitmap, x, y);
				erase_on_bitmap(&mut self.centroid_bitmap, x, y);
				let mut quad_value = 0;
				if (x % GRID_CELL_SIZE) >= GRID_CELL_SIZE / 2
				{
					quad_value |= 0b01;
				}
				if (y % GRID_CELL_SIZE) >= GRID_CELL_SIZE / 2
				{
					quad_value |= 0b10;
				}
				draw_on_quadmap(&mut self.hit_quadmap, x, y, quad_value);
			}
		}
		let noise = PerlinNoise2D::new(
			NOISE_OCTAVES,
			NOISE_AMPLITUDE,
			8.0,
			NOISE_PERSISTENCE,
			NOISE_LACUNARITY,
			NOISE_SCALE,
			0.0,
			noise_seed,
		);
		for y in 0..MAP_SIZE
		{
			for x in 0..MAP_SIZE
			{
				let v = noise.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let period = (2 * GRID_CELL_SIZE) as f64;
				let t = (y as f64) / period + 0.25;
				let amplitude = 1.25 * (NOISE_AMPLITUDE as f64);
				let triangle =
					amplitude * (4.0 * (t - (t + 0.5).floor()).abs() - 1.0);
				let w = v + triangle;
				if w > 0.0
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
				let inner_x = 2 + rng.usize(0..(GRID_CELL_SIZE - 4));
				let inner_y = 2 + rng.usize(0..(GRID_CELL_SIZE - 4));
				let x = c * GRID_CELL_SIZE + inner_x;
				let y = r * GRID_CELL_SIZE + inner_y;
				{
					draw_on_bitmap(&mut self.centroid_bitmap, x, y);
					draw_on_bitmap(&mut self.centroid_bitmap, x + 1, y);
					draw_on_bitmap(&mut self.centroid_bitmap, x, y + 1);
					draw_on_bitmap(&mut self.centroid_bitmap, x + 1, y + 1);
				}
			}
		}
		for r1 in 1..GRID_SIZE
		{
			let r0 = r1 - 1;
			for c1 in 1..GRID_SIZE
			{
				let c0 = c1 - 1;
				self.fudge_grid_at_rc(r0, c0, rng);
			}
		}
	}

	pub fn draw(&self)
	{
		//unsafe { *DRAW_COLORS = 0x2341 };
		//blit(
		//	&self.hit_quadmap,
		//	0,
		//	0,
		//	MAP_SIZE as u32,
		//	MAP_SIZE as u32,
		//	BLIT_2BPP,
		//);
		unsafe { *DRAW_COLORS = 0x04 };
		blit(
			&self.bitmap,
			0,
			0,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x20 };
		blit(
			&self.region_bitmap,
			0,
			0,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x30 };
		blit(
			&self.region_bitmap,
			0,
			0,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP | BLIT_ROTATE,
		);
		unsafe { *DRAW_COLORS = 0x30 };
		blit(
			&self.centroid_bitmap,
			0,
			0,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
	}

	fn fudge_grid_at_rc(
		&mut self,
		r0: usize,
		c0: usize,
		rng: &mut fastrand::Rng,
	)
	{
		let x0 = c0 * GRID_CELL_SIZE + GRID_CELL_SIZE / 2;
		let y0 = r0 * GRID_CELL_SIZE + GRID_CELL_SIZE / 2;
		//let x1 = x0 + GRID_CELL_SIZE;
		//let y1 = y0 + GRID_CELL_SIZE;
		let cx = x0 + GRID_CELL_SIZE / 2 - 1;
		let cy = y0 + GRID_CELL_SIZE / 2 - 1;
		let center_value = rng.u8(0..4);
		draw_on_quadmap(&mut self.hit_quadmap, cx, cy, center_value);
		draw_on_quadmap(&mut self.hit_quadmap, cx + 1, cy, center_value);
		draw_on_quadmap(&mut self.hit_quadmap, cx, cy + 1, center_value);
		draw_on_quadmap(&mut self.hit_quadmap, cx + 1, cy + 1, center_value);
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
