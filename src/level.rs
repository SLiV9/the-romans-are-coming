//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use crate::global_state::Wrapper;
use crate::map::Map;
use crate::map::MAX_NUM_REGIONS;
use crate::palette;
use crate::sprites;

use fastrand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType
{
	Grass,
	Forest,
	Hill,
	Mountain,
	Water,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker
{
	Worker,
	DeadWorker,
	Roman,
	DeadRoman,
}

#[derive(Debug, Clone, Copy)]
struct Region
{
	terrain_type: TerrainType,
	marker: Option<Marker>,
}

const EMPTY_REGION: Region = Region {
	terrain_type: TerrainType::Water,
	marker: None,
};

enum Preview
{
	Gather
	{
		terrain_type: TerrainType
	},
}

pub struct Level
{
	num_regions: usize,
	region_data: [Region; MAX_NUM_REGIONS + 1],
	// TODO adjacency
	threat_level: u8,
	grain: u8,
	wood: u8,
	wine: u8,
	gold: u8,
	score: u16,
	previous_gamepad: u8,
	previous_mousebuttons: u8,
	hover_preview: Option<Preview>,
}

static MAP: Wrapper<Map> = Wrapper::new(Map::empty());

impl Level
{
	pub fn new(seed: u64) -> Level
	{
		let mut num_regions = 0;
		let mut region_data = [EMPTY_REGION; MAX_NUM_REGIONS + 1];
		let mut rng = fastrand::Rng::with_seed(seed);
		{
			// TODO do this more cleanly?
			let map = MAP.get_mut();
			map.generate(&mut rng);
			for (id, terrain_type) in map.regions()
			{
				num_regions += 1;
				region_data[id as usize] = Region {
					terrain_type,
					marker: None,
				};
			}
		}
		Level {
			num_regions,
			region_data,
			threat_level: 0,
			grain: 0,
			wood: 0,
			wine: 0,
			gold: 0,
			score: 0,
			previous_gamepad: 0,
			previous_mousebuttons: 0,
			hover_preview: None,
		}
	}

	pub fn update(&mut self) -> Option<Transition>
	{
		let gamepad = unsafe { *GAMEPAD1 };
		let mousebuttons = unsafe { *MOUSE_BUTTONS };

		// TODO do this more cleanly?
		let map = MAP.get_mut();

		self.hover_preview = None;
		let hovered_region_id = map.determine_hovered_region_id();
		if let Some(region_id) = hovered_region_id
		{
			let region = self.region_data[region_id as usize];
			if region.marker.is_none()
			{
				let preview = match region.terrain_type
				{
					TerrainType::Grass
					| TerrainType::Forest
					| TerrainType::Hill
					| TerrainType::Mountain => Some(Preview::Gather {
						terrain_type: region.terrain_type,
					}),
					TerrainType::Water => None,
				};
				self.hover_preview = preview;
			}

			if (mousebuttons & MOUSE_LEFT != 0)
				&& (self.previous_mousebuttons & MOUSE_LEFT == 0)
			{
				let marker = match self.hover_preview
				{
					Some(Preview::Gather {
						terrain_type: TerrainType::Grass,
					}) =>
					{
						self.grain += 1;
						self.score += 1;
						Some(Marker::Worker)
					}
					Some(Preview::Gather {
						terrain_type: TerrainType::Forest,
					}) =>
					{
						self.wood += 1;
						self.score += 1;
						Some(Marker::Worker)
					}
					Some(Preview::Gather {
						terrain_type: TerrainType::Hill,
					}) =>
					{
						self.wine += 1;
						self.score += 1;
						Some(Marker::Worker)
					}
					Some(Preview::Gather {
						terrain_type: TerrainType::Mountain,
					}) =>
					{
						self.score += 1;
						Some(Marker::Worker)
					}
					_ => None,
				};
				self.region_data[region_id as usize].marker = marker;
				map.set_marker_in_region(region_id, marker);
			}
		}

		self.previous_gamepad = gamepad;
		self.previous_mousebuttons = mousebuttons;
		None
	}

	pub fn draw(&mut self)
	{
		{
			let palette = match self.hover_preview
			{
				Some(Preview::Gather { terrain_type }) => match terrain_type
				{
					TerrainType::Water => palette::DEFAULT,
					TerrainType::Mountain => palette::SNOW,
					TerrainType::Hill => palette::WINE,
					TerrainType::Forest => palette::NATURE,
					TerrainType::Grass => palette::GOLD,
				},
				None => palette::DEFAULT,
			};
			unsafe { *PALETTE = palette };
		}

		{
			// TODO do this more cleanly?
			let map = MAP.get_mut();
			map.draw();
		}

		{
			unsafe { *DRAW_COLORS = 0x11 };
			rect(0, 0, 160, 10);
			unsafe { *DRAW_COLORS = 3 };
			hline(0, 9, 160);

			let mut x = -2;
			unsafe { *DRAW_COLORS = 0x3210 };
			sprites::draw_score_icon(x, 0);
			x += 9;
			unsafe { *DRAW_COLORS = 3 };
			draw_score(self.score, x, 1);
			x += 3 * 8 + 4;
			self.draw_backfill_if_gathering(TerrainType::Grass, x, 0);
			unsafe { *DRAW_COLORS = 0x3210 };
			sprites::draw_grain_icon(x, 0);
			x += 8;
			unsafe { *DRAW_COLORS = 3 };
			draw_resource_value(self.grain, x, 1);
			x += 2 * 8 + 2;
			self.draw_backfill_if_gathering(TerrainType::Forest, x, 0);
			unsafe { *DRAW_COLORS = 0x3210 };
			sprites::draw_wood_icon(x, 0);
			x += 8;
			unsafe { *DRAW_COLORS = 3 };
			draw_resource_value(self.wood, x, 1);
			x += 2 * 8 + 2;
			self.draw_backfill_if_gathering(TerrainType::Hill, x, 0);
			unsafe { *DRAW_COLORS = 0x3210 };
			sprites::draw_wine_icon(x, 0);
			x += 8;
			unsafe { *DRAW_COLORS = 3 };
			draw_resource_value(self.wine, x, 1);
			x += 2 * 8 + 2;
			self.draw_backfill_if_gathering(TerrainType::Mountain, x, 0);
			unsafe { *DRAW_COLORS = 0x3210 };
			sprites::draw_gold_icon(x, 0);
			x += 8;
			unsafe { *DRAW_COLORS = 3 };
			draw_resource_value(self.gold, x, 1);
			x = (SCREEN_SIZE as i32) - 18;
			unsafe { *DRAW_COLORS = 0x3210 };
			sprites::draw_wreath_icon(x, 0);
			x += 9;
			unsafe { *DRAW_COLORS = 3 };
			draw_threat_value(self.threat_level, x, 1);
		}
	}

	fn draw_backfill_if_gathering(&self, target: TerrainType, x: i32, y: i32)
	{
		match self.hover_preview
		{
			Some(Preview::Gather { terrain_type: tt }) if tt == target =>
			{
				unsafe { *DRAW_COLORS = 0x40 };
				sprites::draw_backfill(x - 4, y, 0);
			}
			_ => (),
		}
	}
}

pub struct Transition
{
	pub rng_seed: u64,
}

fn draw_score(value: u16, x: i32, y: i32)
{
	draw_decimal_value::<3>(value, x, y);
}

fn draw_resource_value(value: u8, x: i32, y: i32)
{
	draw_decimal_value::<2>(value.into(), x, y);
}

fn draw_threat_value(value: u8, x: i32, y: i32)
{
	draw_decimal_value::<1>(value.into(), x, y);
}

fn draw_decimal_value<const N: usize>(value: u16, x: i32, y: i32)
{
	let mut buffer = [0u8; N];
	let mut multiplicant = 1;
	for i in (0..N).rev()
	{
		let decimal: u8 = ((value / multiplicant) % 10) as u8;
		buffer[i] = b'0' + decimal;
		multiplicant *= 10;
	}
	let txt = unsafe { std::str::from_utf8_unchecked(&buffer) };
	text(txt, x, y);
}
