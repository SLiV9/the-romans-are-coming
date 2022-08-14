//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use crate::palette;

pub struct Menu {
	ticks: u32,
}

const NUM_INTRO_ANIMATION_TICKS: u32 = 160;

impl Menu {
	pub fn new() -> Self {
		Self { ticks: 0 }
	}

	pub fn update(&mut self) -> Option<Transition> {
		let gamepad = unsafe { *GAMEPAD1 };
		let mousebuttons = unsafe { *MOUSE_BUTTONS };

		self.ticks += 1;

		if gamepad & BUTTON_1 != 0 {
			Some(Transition::Start {
				rng_seed: self.ticks as u64,
			})
		} else if mousebuttons & MOUSE_LEFT != 0 {
			Some(Transition::Start {
				rng_seed: self.ticks as u64,
			})
		} else {
			None
		}
	}

	pub fn draw(&mut self) {
		unsafe { *PALETTE = palette::DEFAULT };

		unsafe { *DRAW_COLORS = 3 };
		{
			if self.ticks > 60 {
				text("THE", 60, 20);
			}
			if self.ticks > 80 {
				text("ROMANS", 60 + 8, 30);
			}
			if self.ticks > 100 {
				text("ARE", 60 + 16, 40);
			}
			if self.ticks > 120 {
				text("COMING!", 60 + 24, 50);
			}
		}

		if self.ticks > NUM_INTRO_ANIMATION_TICKS + 30 {
			unsafe { *DRAW_COLORS = 2 }
			text("Click to begin", 3, 150);
		}
	}
}

pub enum Transition {
	Start { rng_seed: u64 },
}
