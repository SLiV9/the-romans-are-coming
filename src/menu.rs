//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use crate::music::Music;
use crate::palette;
use crate::wreath;

pub struct Menu
{
	ticks: u32,
	music: Music,
}

const NUM_INTRO_ANIMATION_TICKS: u32 = 90;

impl Menu
{
	pub fn new() -> Self
	{
		Self {
			ticks: 0,
			music: Music::main_theme(),
		}
	}

	pub fn update(&mut self) -> Option<Transition>
	{
		let gamepad = unsafe { *GAMEPAD1 };
		let mousebuttons = unsafe { *MOUSE_BUTTONS };

		self.ticks += 1;

		self.music.update();

		if gamepad & BUTTON_1 != 0
		{
			Some(Transition::Start {
				rng_seed: self.ticks as u64,
			})
		}
		else if mousebuttons & MOUSE_LEFT != 0
		{
			Some(Transition::Start {
				rng_seed: self.ticks as u64,
			})
		}
		else
		{
			None
		}
	}

	pub fn draw(&mut self)
	{
		if self.ticks >= NUM_INTRO_ANIMATION_TICKS
		{
			unsafe { *PALETTE = palette::MENU };
		}
		else if self.ticks < 30
		{
			unsafe { *PALETTE = palette::MENU };
			return;
		}
		else if self.ticks % 15 == 0
		{
			let t = self.ticks - 30;
			let maxt = NUM_INTRO_ANIMATION_TICKS - 30;
			let mut work_palette = [0u32; 4];
			for i in 0..4
			{
				work_palette[i] = 0x000000;
				let black = palette::BLACK;
				let target = palette::MENU[i];
				for d in 0..3
				{
					let from: u32 = (black >> (8 * d)) & 0xFF;
					let to: u32 = (target >> (8 * d)) & 0xFF;
					let x = from + t * (to - from) / maxt;
					work_palette[i] |= (x & 0xFF) << (8 * d);
				}
			}
			unsafe { *PALETTE = work_palette };
		}

		unsafe { *DRAW_COLORS = 0x2340 };
		wreath::draw_laurel_wreath(80, 80);

		unsafe { *DRAW_COLORS = 2 };
		{
			text("THE", 16 + 1, 16 + 1);
			text("ROMANS", 16 + 32 + 1, 16 + 1);
			text("ARE", 64 - 8 + 1, 26 + 1);
			text("COMING!", 64 + 24 + 1, 26 + 1);
		}
		unsafe { *DRAW_COLORS = 4 };
		{
			text("THE", 16, 16);
			text("ROMANS", 16 + 32, 16);
			text("ARE", 64 - 8, 26);
			text("COMING!", 64 + 24, 26);
		}

		if self.ticks > NUM_INTRO_ANIMATION_TICKS + 30
		{
			unsafe { *DRAW_COLORS = 3 }
			text("Start", 64, 130);
			//text("Options", 56, 140);
		}

		if unsafe { *GAMEPAD1 } & BUTTON_2 != 0
		{
			unsafe { *DRAW_COLORS = 2 }
			text("v1.0", 126, 150);
		}
	}
}

pub enum Transition
{
	Start
	{
		rng_seed: u64
	},
}
