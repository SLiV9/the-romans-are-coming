//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use crate::palette;

pub struct Level {}

impl Level
{
	pub fn new() -> Self
	{
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

		unsafe { *DRAW_COLORS = 2 };
		hline(0, 15, 160);
		unsafe { *DRAW_COLORS = 3 };
		hline(0, 16, 160);

		unsafe { *DRAW_COLORS = 0x10 };
		rect(3, 3, 33, 9);

		unsafe { *DRAW_COLORS = 3 };
		text("0000", 4, 4);
	}
}

pub struct Transition
{
	pub score: u8,
}
