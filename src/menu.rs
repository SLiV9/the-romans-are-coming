//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use crate::palette;
use crate::sprites;
use crate::wreath;

pub struct Menu
{
	items: &'static [MenuItem],
	selected_item: Option<MenuItem>,
	ticks: u32,
	previous_gamepad: u8,
	previous_mousebuttons: u8,
	loading_transition: Option<Transition>,
}

const NUM_INTRO_ANIMATION_TICKS: u32 = 90;

const X_OF_CENTER_OF_MENU_ITEM: i32 = 80;
const Y_OF_TOP_OF_MENU_ITEM: i32 = 130;
const MENU_ITEM_WIDTH: u32 = 80;
const MENU_ITEM_HEIGHT: u32 = 12;

impl Menu
{
	pub fn new() -> Self
	{
		Self {
			items: &[MenuItem::Start, MenuItem::Freeplay],
			selected_item: None,
			ticks: 0,
			previous_gamepad: 0,
			previous_mousebuttons: 0,
			loading_transition: None,
		}
	}

	pub fn update(&mut self) -> Option<Transition>
	{
		if self.loading_transition.is_some()
		{
			return self.loading_transition;
		}

		let gamepad = unsafe { *GAMEPAD1 };
		let mousebuttons = unsafe { *MOUSE_BUTTONS };

		self.ticks += 1;

		let (mouse_x, mouse_y): (i16, i16) = unsafe { (*MOUSE_X, *MOUSE_Y) };
		let mouse_x = mouse_x as i32;
		let mouse_y = mouse_y as i32;

		let hovered_item = self
			.items
			.iter()
			.enumerate()
			.find(|(offset, _item)| {
				let x = X_OF_CENTER_OF_MENU_ITEM - (MENU_ITEM_WIDTH as i32) / 2;
				let y = Y_OF_TOP_OF_MENU_ITEM
					+ (*offset as i32) * (MENU_ITEM_HEIGHT as i32);
				let w = MENU_ITEM_WIDTH as i32;
				let h = MENU_ITEM_HEIGHT as i32;
				mouse_x > x && mouse_y > y && mouse_x < x + w && mouse_y < y + h
			})
			.map(|(_offset, item)| *item);

		if gamepad & BUTTON_DOWN != 0
			&& self.previous_gamepad & BUTTON_DOWN == 0
		{
			if let Some(i) = self
				.items
				.iter()
				.position(|x| self.selected_item == Some(*x))
			{
				if i + 1 < self.items.len()
				{
					self.selected_item = Some(self.items[i + 1]);
				}
			}
			else
			{
				self.selected_item = Some(self.items[0]);
			}
		}
		else if gamepad & BUTTON_UP != 0
			&& self.previous_gamepad & BUTTON_UP == 0
		{
			if let Some(i) = self
				.items
				.iter()
				.position(|x| self.selected_item == Some(*x))
			{
				if i > 0
				{
					self.selected_item = Some(self.items[i - 1]);
				}
			}
			else
			{
				self.selected_item = Some(self.items[0]);
			}
		}
		else if let Some(item) = hovered_item
		{
			self.selected_item = Some(item);
		}

		let clicked_item = if gamepad & BUTTON_1 != 0
			&& self.previous_gamepad & BUTTON_1 == 0
		{
			self.selected_item
		}
		else if mousebuttons & MOUSE_LEFT != 0
			&& self.previous_mousebuttons & MOUSE_LEFT == 0
		{
			hovered_item
		}
		else
		{
			None
		};
		self.loading_transition = match clicked_item
		{
			Some(MenuItem::Start) => Some(Transition::Start { rng_seed: 1 }),
			Some(MenuItem::Freeplay) => Some(Transition::Start {
				rng_seed: self.ticks as u64,
			}),
			None => None,
		};

		if self.selected_item.is_none()
			&& gamepad & BUTTON_1 != 0
			&& self.previous_gamepad & BUTTON_1 == 0
		{
			self.selected_item = Some(self.items[0]);
		}

		self.previous_gamepad = gamepad;
		self.previous_mousebuttons = mousebuttons;
		None
	}

	pub fn draw(&mut self)
	{
		if self.loading_transition.is_some()
		{
			unsafe { *PALETTE = palette::DEFAULT };
			let x = 80 - 4;
			let y = 80;
			unsafe { *DRAW_COLORS = 0x1320 };
			sprites::draw_tree(x, y + 6, 0);
			sprites::draw_tree(x + 4, y + 7, 0);
			sprites::draw_tree(x + 8, y + 6, 0);
			return;
		}

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
			for (offset, item) in self.items.iter().enumerate()
			{
				if self.selected_item == Some(*item)
				{
					unsafe { *DRAW_COLORS = 0x44 };
					rect(
						X_OF_CENTER_OF_MENU_ITEM - (MENU_ITEM_WIDTH as i32) / 2,
						Y_OF_TOP_OF_MENU_ITEM
							+ (offset as i32) * (MENU_ITEM_HEIGHT as i32),
						MENU_ITEM_WIDTH,
						MENU_ITEM_HEIGHT,
					);
					unsafe { *DRAW_COLORS = 1 };
				}
				else
				{
					unsafe { *DRAW_COLORS = 3 };
				}
				let txt = match item
				{
					MenuItem::Start => "Start",
					MenuItem::Freeplay => "Freeplay",
				};
				let len = txt.len();
				text(
					txt,
					80 - (8 * (len as i32)) / 2,
					Y_OF_TOP_OF_MENU_ITEM
						+ (offset as i32) * (MENU_ITEM_HEIGHT as i32)
						- 4 + (MENU_ITEM_HEIGHT as i32) / 2,
				);
			}
		}

		if unsafe { *GAMEPAD1 } & BUTTON_2 != 0
		{
			unsafe { *DRAW_COLORS = 2 }
			text("v1.1", 126, 150);
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Transition
{
	Start
	{
		rng_seed: u64
	},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuItem
{
	Start,
	Freeplay,
}
