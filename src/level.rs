//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use crate::global_state::Wrapper;
use crate::map::Map;
use crate::palette;

use fastrand;

static MAP: Wrapper<Map> = Wrapper::new(Map::empty());

pub struct Level {}

impl Level
{
	pub fn new(seed: u64) -> Self
	{
		let mut rng = fastrand::Rng::with_seed(seed);
		{
			// TODO do this more cleanly?
			let map = MAP.get_mut();
			map.generate(&mut rng);
		}
		Self {}
	}

	pub fn update(&mut self) -> Option<Transition>
	{
		let gamepad = unsafe { *GAMEPAD1 };
		let mousebuttons = unsafe { *MOUSE_BUTTONS };

		if mousebuttons & MOUSE_LEFT != 0
		{
			// TODO
		}

		None
	}

	pub fn draw(&mut self)
	{
		unsafe { *PALETTE = palette::DEFAULT };

		{
			// TODO do this more cleanly?
			let map = MAP.get_mut();
			map.draw();
		}

		if true
		{
			unsafe { *DRAW_COLORS = 0x11 };
			rect(0, 0, 160, 10);
			unsafe { *DRAW_COLORS = 3 };
			hline(0, 9, 160);

			unsafe { *DRAW_COLORS = 0x10 };
			rect(1, 0, 33, 9);

			unsafe { *DRAW_COLORS = 3 };
			text("0000", 2, 1);
		}
	}
}

pub struct Transition
{
	pub rng_seed: u64,
}
