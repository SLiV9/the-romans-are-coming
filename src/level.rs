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

pub const MAX_NUM_CARDS: usize = 20;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Card
{
	Worker,
	Roman,
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

#[derive(Debug, Clone, Copy)]
enum Preview
{
	HoverResource
	{
		terrain_type: TerrainType,
	},
	PlaceWorker
	{
		region_id: u8,
		terrain_type: TerrainType,
	},
	PlaceRoman
	{
		region_id: u8,
	},
	CannotPlaceRoman,
	Shuffle,
}

const UI_X_GRAIN: i32 = 35;
const UI_X_WOOD: i32 = 61;
const UI_X_WINE: i32 = 87;
const UI_X_GOLD: i32 = 113;

pub struct Level
{
	region_data: [Region; MAX_NUM_REGIONS + 1],
	// TODO adjacency
	card_deck: [Card; MAX_NUM_CARDS],
	num_regions: u8,
	num_cards: u8,
	card_offset: u8,
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
		let threat_level = 1;
		Level {
			num_regions,
			region_data,
			num_cards: 0,
			card_offset: 0,
			card_deck: [Card::Worker; MAX_NUM_CARDS],
			threat_level,
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

		let active_card = if self.card_offset < self.num_cards
		{
			Some(self.card_deck[self.card_offset as usize])
		}
		else
		{
			None
		};

		self.hover_preview = None;
		let hovered_region_id = map.determine_hovered_region_id();
		if active_card.is_none()
		{
			self.hover_preview = Some(Preview::Shuffle);
		}
		else if let Some(region_id) = hovered_region_id
		{
			let region = self.region_data[region_id as usize];
			if region.marker.is_none()
			{
				let preview = if active_card == Some(Card::Roman)
				{
					let can_place = match region.terrain_type
					{
						TerrainType::Grass => true,
						TerrainType::Forest => true,
						TerrainType::Hill => true,
						_ => false,
					};
					if can_place
					{
						Some(Preview::PlaceRoman { region_id })
					}
					else
					{
						None
					}
				}
				else
				{
					match region.terrain_type
					{
						TerrainType::Grass
						| TerrainType::Forest
						| TerrainType::Hill
						| TerrainType::Mountain => Some(Preview::PlaceWorker {
							region_id,
							terrain_type: region.terrain_type,
						}),
						_ => None,
					}
				};
				self.hover_preview = preview;
			}
		}
		else
		{
			let (mouse_x, mouse_y): (i16, i16) =
				unsafe { (*MOUSE_X, *MOUSE_Y) };
			if mouse_y < 9
			{
				for (t, x) in [
					(TerrainType::Grass, UI_X_GRAIN),
					(TerrainType::Forest, UI_X_WOOD),
					(TerrainType::Hill, UI_X_WINE),
					(TerrainType::Mountain, UI_X_GOLD),
				]
				{
					if mouse_x as i32 >= x - 3 && mouse_x as i32 <= x + 28
					{
						self.hover_preview =
							Some(Preview::HoverResource { terrain_type: t });
					}
				}
			}
		}

		if active_card == Some(Card::Roman) && self.hover_preview.is_none()
		{
			self.hover_preview = Some(Preview::CannotPlaceRoman);
		}

		if (mousebuttons & MOUSE_LEFT != 0)
			&& (self.previous_mousebuttons & MOUSE_LEFT == 0)
		{
			self.handle_click(map);
		}

		self.previous_gamepad = gamepad;
		self.previous_mousebuttons = mousebuttons;
		None
	}

	fn handle_click(&mut self, map: &mut Map)
	{
		let placed = match self.hover_preview
		{
			Some(Preview::Shuffle) =>
			{
				let num_workers = 5;
				self.num_cards = num_workers + self.threat_level;
				for i in 0..self.num_cards
				{
					let card = if i / 3 >= num_workers
					{
						Card::Roman
					}
					else if i / 3 >= self.threat_level
					{
						Card::Worker
					}
					else if i % 3 == 2
					{
						Card::Roman
					}
					else
					{
						Card::Worker
					};
					self.card_deck[i as usize] = card;
				}
				self.card_offset = 0;
				return;
			}
			Some(Preview::PlaceWorker {
				region_id,
				terrain_type: TerrainType::Grass,
			}) =>
			{
				self.grain += 1;
				self.score += 1;
				Some((region_id, Marker::Worker))
			}
			Some(Preview::PlaceWorker {
				region_id,
				terrain_type: TerrainType::Forest,
			}) =>
			{
				self.wood += 1;
				self.score += 1;
				Some((region_id, Marker::Worker))
			}
			Some(Preview::PlaceWorker {
				region_id,
				terrain_type: TerrainType::Hill,
			}) =>
			{
				self.wine += 1;
				self.score += 1;
				Some((region_id, Marker::Worker))
			}
			Some(Preview::PlaceWorker {
				region_id,
				terrain_type: TerrainType::Mountain,
			}) =>
			{
				self.score += 1;
				Some((region_id, Marker::Worker))
			}
			Some(Preview::PlaceRoman { region_id }) =>
			{
				self.score += 1;
				Some((region_id, Marker::Roman))
			}
			_ => None,
		};
		if let Some((region_id, marker)) = placed
		{
			self.region_data[region_id as usize].marker = Some(marker);
			map.set_marker_in_region(region_id, Some(marker));
			self.card_offset += 1;
		}
	}

	pub fn draw(&mut self)
	{
		{
			let palette = match self.hover_preview
			{
				Some(Preview::HoverResource { terrain_type })
				| Some(Preview::PlaceWorker {
					region_id: _,
					terrain_type,
				}) => match terrain_type
				{
					TerrainType::Water => palette::DEFAULT,
					TerrainType::Mountain => palette::SNOW,
					TerrainType::Hill => palette::WINE,
					TerrainType::Forest => palette::NATURE,
					TerrainType::Grass => palette::GOLD,
				},
				Some(Preview::PlaceRoman { region_id: _ }) => palette::BLOOD,
				Some(Preview::CannotPlaceRoman) => palette::BLOOD,
				Some(Preview::Shuffle) => palette::DEFAULT,
				None => palette::DEFAULT,
			};
			unsafe { *PALETTE = palette };
		}

		{
			let (region_id, highlighted_terrain) = match self.hover_preview
			{
				Some(Preview::HoverResource { terrain_type }) =>
				{
					(None, Some(terrain_type))
				}
				Some(Preview::PlaceWorker {
					region_id,
					terrain_type: _,
				}) => (Some(region_id), None),
				Some(Preview::PlaceRoman { region_id }) =>
				{
					(Some(region_id), None)
				}
				_ => (None, None),
			};

			// TODO do this more cleanly?
			let map = MAP.get_mut();
			map.draw(region_id, highlighted_terrain);
		}

		if self.card_offset < self.num_cards
		{
			unsafe { *DRAW_COLORS = 0x31 };
			rect(-1, 9, 9, 3 + 7 * (self.num_cards as u32));
			unsafe { *DRAW_COLORS = 3 };
			hline(0, 17, 9);

			match self.hover_preview
			{
				Some(Preview::PlaceRoman { .. })
				| Some(Preview::CannotPlaceRoman) =>
				{
					unsafe { *DRAW_COLORS = 0x44 };
				}
				_ =>
				{
					unsafe { *DRAW_COLORS = 0x22 };
				}
			}
			rect(0, 10, 7, 7);

			unsafe { *DRAW_COLORS = 0x4320 };
			let remaining_card_offsets =
				(self.card_offset as usize)..(self.num_cards as usize);
			for (i, card) in
				self.card_deck[remaining_card_offsets].iter().enumerate()
			{
				let y = 11 + 1 * ((i > 0) as i32) + 7 * (i as i32);
				let alt = match card
				{
					Card::Worker => 0,
					Card::Roman => 1,
				};
				sprites::draw_card_icon(1, y, alt);
			}
		}

		unsafe { *DRAW_COLORS = 0x11 };
		rect(0, 0, 160, 10);
		unsafe { *DRAW_COLORS = 3 };
		hline(0, 9, 160);

		unsafe { *DRAW_COLORS = 0x40 };
		match self.hover_preview
		{
			Some(Preview::HoverResource {
				terrain_type: TerrainType::Grass,
			})
			| Some(Preview::PlaceWorker {
				region_id: _,
				terrain_type: TerrainType::Grass,
			}) =>
			{
				sprites::draw_backfill(UI_X_GRAIN - 4, 0, 0);
			}
			Some(Preview::HoverResource {
				terrain_type: TerrainType::Forest,
			})
			| Some(Preview::PlaceWorker {
				region_id: _,
				terrain_type: TerrainType::Forest,
			}) =>
			{
				sprites::draw_backfill(UI_X_WOOD - 4, 0, 0);
			}
			Some(Preview::HoverResource {
				terrain_type: TerrainType::Hill,
			})
			| Some(Preview::PlaceWorker {
				region_id: _,
				terrain_type: TerrainType::Hill,
			}) =>
			{
				sprites::draw_backfill(UI_X_WINE - 4, 0, 0);
			}
			Some(Preview::HoverResource {
				terrain_type: TerrainType::Mountain,
			})
			| Some(Preview::PlaceWorker {
				region_id: _,
				terrain_type: TerrainType::Mountain,
			}) =>
			{
				sprites::draw_backfill(UI_X_GOLD - 4, 0, 0);
			}
			Some(Preview::PlaceRoman { region_id: _ })
			| Some(Preview::CannotPlaceRoman) =>
			{
				sprites::draw_backfill((SCREEN_SIZE as i32) - 24, 0, 0);
			}
			_ => (),
		}

		unsafe { *DRAW_COLORS = 0x3210 };
		sprites::draw_score_icon(-3, 0);
		unsafe { *DRAW_COLORS = 3 };
		draw_score(self.score, 6, 1);

		unsafe { *DRAW_COLORS = 0x3210 };
		sprites::draw_grain_icon(UI_X_GRAIN, 0);
		unsafe { *DRAW_COLORS = 3 };
		draw_resource_value(self.grain, UI_X_GRAIN + 8, 1);
		unsafe { *DRAW_COLORS = 0x3210 };
		sprites::draw_wood_icon(UI_X_WOOD, 0);
		unsafe { *DRAW_COLORS = 3 };
		draw_resource_value(self.wood, UI_X_WOOD + 8, 1);
		unsafe { *DRAW_COLORS = 0x3210 };
		sprites::draw_wine_icon(UI_X_WINE, 0);
		unsafe { *DRAW_COLORS = 3 };
		draw_resource_value(self.wine, UI_X_WINE + 8, 1);
		unsafe { *DRAW_COLORS = 0x3210 };
		sprites::draw_gold_icon(UI_X_GOLD, 0);
		unsafe { *DRAW_COLORS = 3 };
		draw_resource_value(self.gold, UI_X_GOLD + 8, 1);

		unsafe { *DRAW_COLORS = 0x3210 };
		sprites::draw_wreath_icon((SCREEN_SIZE as i32) - 17, 0);
		unsafe { *DRAW_COLORS = 3 };
		draw_threat_value(self.threat_level, (SCREEN_SIZE as i32) - 8, 1);
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
