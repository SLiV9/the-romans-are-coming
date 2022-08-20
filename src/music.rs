//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

const SCALE_SIZE: usize = 8;
const SCALE: [i8; SCALE_SIZE] = [0, 2, 3, 5, 7, 8, 11, 12];
const RATE: usize = 14;
const GROUND_FREQUENCY: f64 = 220f64;

const SEED_ALPHA: usize = 174;
const SEED_CASTLE: usize = 130;
const SEED_ALTERNATING: usize = 76;
const SEED_DESCENDING: usize = 136;
const SEED_BLOOD: usize = 7;
const SEED_ENGINE: usize = 11;
const SEED_INVESTIGATE: usize = 13;
const SEED_ULTRA: usize = 29;
const SEED_ARRIVAL: usize = 1;
const SEED_PENSIVE: usize = 3;
const SEED_RAIN: usize = 311;
const SEED_UNSURE: usize = 211;
const SEED_DEMISE: usize = 213;

pub struct Music
{
	pub seed: usize,
	pub base: i8,
	pub target_volume: u8,

	volume: u8,
	ticks: usize,
}

impl Music
{
	pub fn main_theme() -> Music
	{
		Music {
			seed: SEED_DEMISE,
			base: -9,
			target_volume: 30,
			volume: 0,
			ticks: 0,
		}
	}

	pub fn update(&mut self)
	{
		let t = self.ticks / RATE;
		if self.ticks % RATE != 0
		{
			self.ticks += 1;
			return;
		}
		self.ticks += 1;

		if self.volume < self.target_volume
		{
			self.volume += 1;
		}
		else if self.volume > self.target_volume
		{
			self.volume -= 1;
		}

		let note = self.base + determine_note(t, self.seed);
		play_note(note, self.volume)
	}
}

const fn determine_note(t: usize, seed: usize) -> i8
{
	let ground = 12;
	if t % ground == 0
	{
		if (5 * (t / ground)) % 17 < 5
		{
			12
		}
		else
		{
			0
		}
	}
	else
	{
		let offset = seed.wrapping_mul(t) % 313;
		SCALE[offset % SCALE_SIZE]
	}
}

fn play_note(note: i8, volume: u8)
{
	let power = 12;
	let magic = 2f64.powf(1.0 / (power as f64));
	let freq: f64 = GROUND_FREQUENCY * magic.powf(note as f64);
	let sustain = (RATE / 2) as u32;
	let release = (RATE / 2) as u32;
	tone(
		freq.round() as u32,
		sustain | (release << 8),
		volume as u32,
		TONE_TRIANGLE,
	);
}
