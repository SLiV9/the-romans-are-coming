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
}

impl Map
{
	pub const fn empty() -> Self
	{
		Self {
			bitmap: [0; BITMAP_SIZE],
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
		for r in 0..MAP_SIZE
		{
			for c in 0..MAP_SIZE
			{
				let elevation = noise.get_noise(r as f64 + 0.5, c as f64 + 0.5);
				if elevation > 0.0
				{
					draw_on_bitmap(&mut self.bitmap, r, c);
				}
				else
				{
					erase_on_bitmap(&mut self.bitmap, r, c);
				}
			}
		}
	}

	pub fn blit(&self)
	{
		blit(
			&self.bitmap,
			0,
			0,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
	}
}

fn draw_on_bitmap(bitmap: &mut [u8; BITMAP_SIZE], row: usize, col: usize)
{
	let offset = row * MAP_SIZE + col;
	let byte_offset = offset / 8;
	assert!(byte_offset < BITMAP_SIZE);
	let bit_shift = offset % 8;
	bitmap[byte_offset] |= 0b10000000 >> bit_shift;
}

fn erase_on_bitmap(bitmap: &mut [u8; BITMAP_SIZE], row: usize, col: usize)
{
	let offset = row * MAP_SIZE + col;
	let byte_offset = offset / 8;
	assert!(byte_offset < BITMAP_SIZE);
	let bit_shift = offset % 8;
	bitmap[byte_offset] &= !(0b10000000 >> bit_shift);
}
