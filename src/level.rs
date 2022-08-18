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

pub struct Level
{
	num_regions: usize,
	region_data: [Region; MAX_NUM_REGIONS + 1],
	// TODO adjacency
	previous_gamepad: u8,
	previous_mousebuttons: u8,
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
			previous_gamepad: 0,
			previous_mousebuttons: 0,
		}
	}

	pub fn update(&mut self) -> Option<Transition>
	{
		let gamepad = unsafe { *GAMEPAD1 };
		let mousebuttons = unsafe { *MOUSE_BUTTONS };

		// TODO do this more cleanly?
		let map = MAP.get_mut();

		if (mousebuttons & MOUSE_LEFT != 0)
			&& (self.previous_mousebuttons & MOUSE_LEFT == 0)
		{
			let hovered_region_id = map.determine_hovered_region_id();
			if let Some(region_id) = hovered_region_id
			{
				let region = self.region_data[region_id as usize];
				if region.marker.is_none()
				{
					// TODO game logic
					let marker = match region.terrain_type
					{
						TerrainType::Grass => Some(Marker::Roman),
						TerrainType::Hill => Some(Marker::DeadRoman),
						TerrainType::Forest => Some(Marker::Worker),
						TerrainType::Mountain => Some(Marker::DeadWorker),
						TerrainType::Water => None,
					};
					self.region_data[region_id as usize].marker = marker;
					map.set_marker_in_region(region_id, marker);
				}
			}
		}

		self.previous_gamepad = gamepad;
		self.previous_mousebuttons = mousebuttons;
		None
	}

	pub fn draw(&mut self)
	{
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
