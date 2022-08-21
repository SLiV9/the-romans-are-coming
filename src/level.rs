//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use crate::decree::Decree;
use crate::decree::Tutorial;
use crate::decree::{AllOrNone, InOrNear};
use crate::global_state::Wrapper;
use crate::map::Map;
use crate::palette;
use crate::sprites;

use bitmaps::Bitmap;
use fastrand;

pub const MAX_NUM_REGIONS: usize = 35;
pub const MAX_NUM_CARDS: usize = 20;
pub const MAX_NUM_DECREES: usize = 6;
pub const TOTAL_NUM_DECREES: usize = 23;

const MAX_THREAT_LEVEL: u8 = 10;
const MAX_TRIBUTE: u8 = 8;

const VILLAGE_WOOD_COST: u8 = 10;
const VILLAGE_GOLD_COST: u8 = 5;
const MAX_STORED_GRAIN: u8 = 20;
const MAX_STORED_WOOD: u8 = 20;
const MAX_STORED_WINE: u8 = 50;
const MAX_STORED_GOLD: u8 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType
{
	Village,
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
	Occupied,
	FogOfWar,
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
enum State
{
	Setup,
	NewObjectives,
	NewDecrees,
	Placement,
	Shuffling,
	Resolution,
	Occupation,
	Cleanup,
	DecreeViolated
	{
		decree_offset: u8,
	},
	TributePaid,
	TributeFailed,
	TributeSkipped,
	GameOver,
}

#[derive(Debug, Clone, Copy)]
enum Preview
{
	HoverObjectives,
	HoverDecrees,
	HoverResource
	{
		terrain_type: TerrainType,
	},
	PlaceWorker
	{
		region_id: i8,
		terrain_type: TerrainType,
	},
	PlaceRoman
	{
		region_id: i8,
	},
	CannotPlaceRoman,
}

const UI_X_GRAIN: i32 = 35;
const UI_X_WOOD: i32 = 61;
const UI_X_WINE: i32 = 113;
const UI_X_GOLD: i32 = 87;

pub struct Level
{
	region_data: [Region; MAX_NUM_REGIONS],
	adjacency: [Bitmap<MAX_NUM_REGIONS>; MAX_NUM_REGIONS],
	border_adjacency: Bitmap<MAX_NUM_REGIONS>,
	kill_preview: Bitmap<MAX_NUM_REGIONS>,
	attack_preview: Bitmap<MAX_NUM_REGIONS>,
	support_preview: Bitmap<MAX_NUM_REGIONS>,
	gather_preview: Bitmap<MAX_NUM_REGIONS>,
	card_deck: [Card; MAX_NUM_CARDS],
	decree_data: [Decree; TOTAL_NUM_DECREES],
	num_regions: u8,
	num_cards: u8,
	num_decrees: u8,
	card_offset: u8,
	threat_level: u8,
	tribute: u8,
	grain: u8,
	wood: u8,
	wine: u8,
	gold: u8,
	score: u16,
	ticks_in_4sec: u8,
	previous_gamepad: u8,
	previous_mousebuttons: u8,
	state: State,
	tutorial: Option<Tutorial>,
	hover_preview: Option<Preview>,
	cursor: Cursor,
	rng: fastrand::Rng,
}

static MAP: Wrapper<Map> = Wrapper::new(Map::empty());

impl Level
{
	pub fn new(seed: u64) -> Level
	{
		let decree_data = [
			Decree::Dummy,
			Decree::NoRomansInAmbush,
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Roman,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Water,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Roman,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Grass,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Roman,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Forest,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Roman,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Hill,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Roman,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Mountain,
			},
			Decree::Regional {
				all_or_none: AllOrNone::None,
				marker: Marker::Roman,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Water,
			},
			Decree::Regional {
				all_or_none: AllOrNone::None,
				marker: Marker::Roman,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Forest,
			},
			Decree::Regional {
				all_or_none: AllOrNone::None,
				marker: Marker::Roman,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Hill,
			},
			Decree::Regional {
				all_or_none: AllOrNone::None,
				marker: Marker::Roman,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Mountain,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Worker,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Water,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Worker,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Grass,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Worker,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Forest,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Worker,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Hill,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Worker,
				in_or_near: InOrNear::Near,
				terrain_type: TerrainType::Mountain,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Roman,
				in_or_near: InOrNear::In,
				terrain_type: TerrainType::Grass,
			},
			Decree::Regional {
				all_or_none: AllOrNone::All,
				marker: Marker::Worker,
				in_or_near: InOrNear::In,
				terrain_type: TerrainType::Forest,
			},
			Decree::Regional {
				all_or_none: AllOrNone::None,
				marker: Marker::Worker,
				in_or_near: InOrNear::In,
				terrain_type: TerrainType::Hill,
			},
			Decree::Regional {
				all_or_none: AllOrNone::None,
				marker: Marker::Worker,
				in_or_near: InOrNear::In,
				terrain_type: TerrainType::Mountain,
			},
			Decree::Regional {
				all_or_none: AllOrNone::None,
				marker: Marker::Roman,
				in_or_near: InOrNear::In,
				terrain_type: TerrainType::Forest,
			},
			Decree::Regional {
				all_or_none: AllOrNone::None,
				marker: Marker::Roman,
				in_or_near: InOrNear::In,
				terrain_type: TerrainType::Hill,
			},
			Decree::Regional {
				all_or_none: AllOrNone::None,
				marker: Marker::Roman,
				in_or_near: InOrNear::In,
				terrain_type: TerrainType::Mountain,
			},
		];
		trace(format!("seed = {}", seed));
		let mut num_regions = 0;
		let mut region_data = [EMPTY_REGION; MAX_NUM_REGIONS];
		let mut adjacency = [Bitmap::default(); MAX_NUM_REGIONS];
		let mut border_adjacency = Bitmap::default();
		let mut rng = fastrand::Rng::with_seed(seed);
		{
			let map = MAP.get_mut();
			map.generate(&mut rng);
			for (id, terrain_type) in map.regions()
			{
				region_data[id as usize] = Region {
					terrain_type,
					marker: None,
				};
				num_regions += 1;
			}
			map.fill_adjacency(&mut adjacency, &mut border_adjacency);
			let roman_spawn = match seed
			{
				1 => None,
				_ => (0..(num_regions as usize))
					.rev()
					.filter(|i| border_adjacency.get(*i as usize))
					.find(|i| match region_data[*i].terrain_type
					{
						TerrainType::Water => false,
						TerrainType::Mountain => false,
						_ => true,
					})
					.map(|i| i as i8),
			};
			if let Some(region_id) = roman_spawn
			{
				map.occupy_region(region_id);
				map.update_occupation_map(100);
				let marker = Marker::Occupied;
				region_data[region_id as usize].marker = Some(marker);
				map.set_marker_in_region(region_id, Some(marker));
			}
			match seed
			{
				1 =>
				{
					for region_id in [
						0, 1, 2, 3, 4, 6, 8, 9, 10, 12, 13, 14, 15, 19, 20, 22,
						23, 24, 25, 26, 27, 29, 30, 31,
					]
					{
						let marker = Marker::FogOfWar;
						region_data[region_id as usize].marker = Some(marker);
						map.set_marker_in_region(region_id, Some(marker));
					}
				}
				_ => (),
			}
		}
		let tutorial = match seed
		{
			1 => Some(Tutorial::PlaceBanners),
			202 => Some(Tutorial::Village),
			_ => None,
		};
		let threat_level = 0;
		let tribute = match seed
		{
			1 => 0,
			202 => 1,
			_ => 2,
		};
		let starting_grain = match seed
		{
			_ => 24,
		};
		let starting_wood = match seed
		{
			1 => 0,
			202 => 15,
			_ => 10,
		};
		let starting_gold = match seed
		{
			1 => 0,
			_ => 10,
		};
		Level {
			num_regions,
			region_data,
			adjacency,
			border_adjacency,
			kill_preview: Bitmap::default(),
			attack_preview: Bitmap::default(),
			support_preview: Bitmap::default(),
			gather_preview: Bitmap::default(),
			num_cards: 0,
			card_offset: 0,
			card_deck: [Card::Worker; MAX_NUM_CARDS],
			decree_data,
			num_decrees: 1,
			threat_level,
			tribute,
			grain: starting_grain,
			wood: starting_wood,
			wine: 0,
			gold: starting_gold,
			score: 0,
			ticks_in_4sec: 0,
			previous_gamepad: 0,
			previous_mousebuttons: 0,
			hover_preview: None,
			state: State::Setup,
			tutorial,
			cursor: Cursor {
				row: 8,
				col: 8,
				mouse_x: -1,
				mouse_y: -1,
			},
			rng,
		}
	}

	pub fn update(&mut self) -> Option<Transition>
	{
		let gamepad = unsafe { *GAMEPAD1 };
		let mousebuttons = unsafe { *MOUSE_BUTTONS };

		let map = MAP.get_mut();

		let active_card = if self.card_offset < self.num_cards
		{
			Some(self.card_deck[self.card_offset as usize])
		}
		else
		{
			None
		};

		if unsafe { *GAMEPAD1 } & BUTTON_2 != 0
		{
			self.threat_level = MAX_THREAT_LEVEL;
			self.num_decrees = 0;
			self.num_cards = 0;
		}

		self.hover_preview = None;
		self.kill_preview = Bitmap::new();
		self.attack_preview = Bitmap::new();
		self.support_preview = Bitmap::new();
		self.gather_preview = Bitmap::new();
		let hovered_region_id = {
			let (mouse_x, mouse_y): (i16, i16) =
				unsafe { (*MOUSE_X, *MOUSE_Y) };
			let mouse_x = mouse_x as i32;
			let mouse_y = mouse_y as i32;
			if mouse_x == self.cursor.mouse_x && mouse_y == self.cursor.mouse_y
			{
			}
			else
			{
				self.cursor.mouse_x = mouse_x;
				self.cursor.mouse_y = mouse_y;
			}
			let arrow_mask =
				BUTTON_UP | BUTTON_DOWN | BUTTON_LEFT | BUTTON_RIGHT;
			if (gamepad & arrow_mask != 0)
				&& (self.previous_gamepad & arrow_mask == 0)
			{
				let (dc, dr) = if gamepad & BUTTON_UP != 0
				{
					(0, -1)
				}
				else if gamepad & BUTTON_DOWN != 0
				{
					(0, 1)
				}
				else if gamepad & BUTTON_LEFT != 0
				{
					(-1, 0)
				}
				else
				{
					(1, 0)
				};
			}
			map.determine_hovered_region_id(&self.cursor)
		};
		if self.num_decrees == 0
		{
			self.num_cards = 0;
			self.pick_decrees();

			if self.decree_data[0] == Decree::Dummy
			{
				self.state = State::Shuffling;
			}
			else if self.threat_level < MAX_THREAT_LEVEL
			{
				self.state = State::NewDecrees;
			}
			else
			{
				for i in 0..self.num_regions
				{
					map.occupy_region(i as i8);
				}
				self.state = State::Occupation;
			}
			self.ticks_in_4sec = 0;
		}
		else if active_card.is_none()
		{
			match self.state
			{
				State::Setup =>
				{
					self.pick_decrees();
					if self.tutorial.is_some()
					{
						if self.tutorial == Some(Tutorial::Village)
						{
							self.tutorial = None;
						}
						self.state = State::NewObjectives;
					}
					else
					{
						self.state = State::Shuffling;
					}
					self.ticks_in_4sec = 0;
				}
				State::NewObjectives =>
				{
					// Wait for user to finish reading.
					self.hover_preview = Some(Preview::HoverObjectives);
				}
				State::Placement =>
				{
					if self.count_remaining_spaces() == 0
					{
						if self.tutorial == Some(Tutorial::FeedWorkers)
						{
							self.tutorial = Some(Tutorial::Harvest);
							self.state = State::NewObjectives;
						}
						else if self.tutorial
							== Some(Tutorial::RomansHaveCome)
						{
							self.tribute = 15;
							self.tutorial = Some(Tutorial::Tribute);
							self.state = State::NewObjectives;
						}
						else
						{
							self.state = State::Resolution;
						}
					}
					else if self.tutorial == Some(Tutorial::PlaceBanners)
					{
						self.tutorial = Some(Tutorial::FeedWorkers);
						self.state = State::NewObjectives;
					}
					else
					{
						self.state = State::Shuffling;
					}
					self.ticks_in_4sec = 0;
				}
				State::Shuffling =>
				{
					if self.ticks_in_4sec == 5
					{
						self.shuffle();
						self.state = State::Placement;
					}
				}
				State::Resolution =>
				{
					if self.ticks_in_4sec == 20
					{
						let survivor =
							(0..(self.num_regions as usize)).find(|i| {
								self.region_data[*i].marker
									== Some(Marker::Worker)
							});
						if let Some(i) = survivor
						{
							match self.region_data[i].terrain_type
							{
								TerrainType::Village => self.grain += 1,
								TerrainType::Grass => self.grain += 1,
								TerrainType::Forest => self.wood += 1,
								TerrainType::Hill => self.wine += 1,
								TerrainType::Mountain => self.gold += 1,
								TerrainType::Water => (),
							}
							self.region_data[i].marker = None;
							map.set_marker_in_region(i as i8, None);
						}
						else
						{
							self.state = State::Occupation;
						}
						self.ticks_in_4sec = 0;
					}
				}
				State::Occupation =>
				{
					if self.ticks_in_4sec == 1
					{
						let mut any = false;
						for i in 0..(self.num_regions as usize)
						{
							if self.region_data[i].marker == Some(Marker::Roman)
							{
								let region_id = i as i8;
								map.occupy_region(region_id);
								let marker = Marker::Occupied;
								self.region_data[i].marker = Some(marker);
								map.set_marker_in_region(
									region_id,
									Some(marker),
								);
								any = true;
							}
						}
						if !any
						{
							// Find a battlefield (empty squares are either
							// water or had workers on them) on the border.
							let roman_spawn = (0..(self.num_regions as usize))
								.rev()
								.filter(|i| self.border_adjacency.get(*i))
								.find(|i| match self.region_data[*i].marker
								{
									Some(Marker::DeadRoman) => true,
									Some(Marker::DeadWorker) => true,
									_ => false,
								});
							if let Some(i) = roman_spawn
							{
								let region_id = i as i8;
								map.occupy_region(region_id);
								let marker = Marker::Occupied;
								self.region_data[i].marker = Some(marker);
								map.set_marker_in_region(
									region_id,
									Some(marker),
								);
							}
						}
					}
					else if self.ticks_in_4sec >= 30
						&& self.ticks_in_4sec <= 50
						&& self.ticks_in_4sec % 2 == 0
					{
						let percentage = 5 * (self.ticks_in_4sec - 30);
						map.update_occupation_map(percentage);
					}
					else if self.ticks_in_4sec >= 80
					{
						map.update_occupation_map(100);
						if self.threat_level < MAX_THREAT_LEVEL
						{
							self.state = State::Cleanup;
						}
						else
						{
							self.state = State::GameOver;
						}
						self.ticks_in_4sec = 0;
					}
				}
				State::Cleanup =>
				{
					if self.ticks_in_4sec == 5
					{
						let trash = (0..(self.num_regions as usize)).find(
							|i| match self.region_data[*i].marker
							{
								Some(Marker::Occupied) => false,
								Some(Marker::FogOfWar) => false,
								Some(Marker::Roman) => true,
								Some(_) => true,
								None => false,
							},
						);
						if let Some(i) = trash
						{
							self.region_data[i].marker = None;
							map.set_marker_in_region(i as i8, None);
						}
						else
						{
							if self.tutorial == Some(Tutorial::FirstKill)
							{
								self.tutorial = Some(Tutorial::RomansHaveCome);
								let marker = Marker::Roman;
								self.region_data[27].marker = Some(marker);
								map.set_marker_in_region(27, Some(marker));
								self.state = State::Occupation;
							}
							else if self.tribute == 0
							{
								self.state = State::TributeSkipped;
							}
							else if self.wine >= self.tribute
							{
								self.wine -= self.tribute;
								self.tribute += 2;
								if self.tribute > MAX_TRIBUTE
								{
									self.tribute = MAX_TRIBUTE;
								}
								self.state = State::TributePaid;
							}
							else
							{
								self.tribute += 1;
								if self.tutorial == Some(Tutorial::Tribute)
								{
									self.tribute = 1;
								}
								if self.tribute > MAX_TRIBUTE
								{
									self.tribute = MAX_TRIBUTE;
								}
								if self.threat_level < MAX_THREAT_LEVEL
								{
									self.threat_level += 1;
								}
								else
								{
									self.num_decrees = 0;
								}
								self.state = State::TributeFailed;
							}
							if self.grain > MAX_STORED_GRAIN
							{
								self.grain = MAX_STORED_GRAIN;
							}
							if self.wood > MAX_STORED_WOOD
							{
								self.wood = MAX_STORED_WOOD;
							}
							if self.wine > MAX_STORED_WINE
							{
								self.wine = MAX_STORED_WINE;
							}
							if self.gold > MAX_STORED_GOLD
							{
								self.gold = MAX_STORED_GOLD;
							}
						}
						self.ticks_in_4sec = 0;
					}
				}
				State::NewDecrees =>
				{
					// Wait for user to finish reading.
					self.hover_preview = Some(Preview::HoverDecrees);
				}
				State::DecreeViolated { .. } =>
				{
					// Wait for user to finish reading.
				}
				State::TributePaid | State::TributeSkipped =>
				{
					if self.ticks_in_4sec >= 90
					{
						if self.tutorial == Some(Tutorial::Harvest)
						{
							let marker = Marker::Roman;
							self.region_data[22].marker = Some(marker);
							map.set_marker_in_region(22, Some(marker));
						}

						self.state = State::Shuffling;
						self.ticks_in_4sec = 0;

						if self.tutorial == Some(Tutorial::RomansHaveCome)
						{
							self.state = State::NewObjectives;
						}
					}
				}
				State::TributeFailed =>
				{
					// Wait for user to finish reading.
				}
				State::GameOver =>
				{
					// Wait for user to finish reading.
				}
			}
		}
		else if let Some(region_id) = hovered_region_id
		{
			let region = self.region_data[region_id as usize];
			if region.marker.is_none()
			{
				let can_place = match region.terrain_type
				{
					TerrainType::Village => true,
					TerrainType::Grass => true,
					TerrainType::Forest => true,
					TerrainType::Hill => true,
					TerrainType::Mountain => true,
					_ => false,
				};
				let preview = if active_card == Some(Card::Roman)
				{
					if can_place
					{
						self.figure_out_combat(region_id, Card::Roman);
						Some(Preview::PlaceRoman { region_id })
					}
					else
					{
						None
					}
				}
				else
				{
					if can_place
					{
						self.figure_out_combat(region_id, Card::Worker);
						Some(Preview::PlaceWorker {
							region_id,
							terrain_type: region.terrain_type,
						})
					}
					else
					{
						None
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
				if mouse_x <= 30
				{
					self.hover_preview = Some(Preview::HoverObjectives);
				}
				else if mouse_x >= (SCREEN_SIZE as i16) - 24
				{
					self.hover_preview = Some(Preview::HoverDecrees);
				}
			}
		}

		if active_card == Some(Card::Roman) && self.hover_preview.is_none()
		{
			self.hover_preview = Some(Preview::CannotPlaceRoman);
		}

		if (mousebuttons & MOUSE_LEFT != 0)
			&& (self.previous_mousebuttons & MOUSE_LEFT == 0)
			|| (gamepad & BUTTON_1 != 0)
				&& (self.previous_gamepad & BUTTON_1 == 0)
		{
			match self.state
			{
				State::NewObjectives =>
				{
					self.num_decrees = 0;
					self.state = State::Shuffling;
					self.ticks_in_4sec = 0;
				}
				State::Placement =>
				{
					self.place_marker(map);

					if self.tutorial == Some(Tutorial::Harvest)
					{
						if self.region_data[0..(self.num_regions as usize)]
							.iter()
							.any(|region| {
								region.marker == Some(Marker::DeadRoman)
							})
						{
							self.tutorial = Some(Tutorial::FirstKill);
							self.state = State::NewObjectives;
							self.num_cards = 0;
						}
					}
				}
				State::NewDecrees =>
				{
					self.state = State::Shuffling;
					self.ticks_in_4sec = 0;
				}
				State::TributePaid | State::TributeSkipped =>
				{
					self.ticks_in_4sec = 100;
				}
				State::DecreeViolated { .. } =>
				{
					self.num_decrees = 0;
				}
				State::TributeFailed =>
				{
					if self.tutorial == Some(Tutorial::Tribute)
					{
						self.num_decrees = 0;
					}
					self.state = State::Shuffling;
					self.ticks_in_4sec = 0;
				}
				State::GameOver =>
				{
					if self.tutorial.is_some()
					{
						return Some(Transition { rng_seed: 202 });
					}
					else
					{
						return Some(Transition {
							rng_seed: self.ticks_in_4sec as u64,
						});
					}
				}
				_ => (),
			}
		}

		self.ticks_in_4sec += 1;
		if self.ticks_in_4sec == 240
		{
			self.ticks_in_4sec = 0;
		}
		self.previous_gamepad = gamepad;
		self.previous_mousebuttons = mousebuttons;
		None
	}

	fn figure_out_combat(&mut self, region_id: i8, card: Card)
	{
		let mut supporters = Bitmap::<MAX_NUM_REGIONS>::default();
		let mut enemies = Bitmap::<MAX_NUM_REGIONS>::default();
		let mut occupants = Bitmap::<MAX_NUM_REGIONS>::default();
		let mut num_supporters = 0;
		let mut num_enemies = 0;
		let mut num_occupants = 0;
		for i in (0..(self.num_regions as usize))
			.filter(|i| *i as i8 != region_id)
			.filter(|i| self.adjacency[region_id as usize].get(*i))
		{
			match (self.region_data[i].marker, card)
			{
				(Some(Marker::Worker), Card::Worker) =>
				{
					if self.region_data[i].terrain_type != TerrainType::Village
					{
						supporters.set(i, true);
						num_supporters += 1;
					}
				}
				(Some(Marker::Roman), Card::Worker) =>
				{
					enemies.set(i, true);
					num_enemies += 1;
				}
				(Some(Marker::Occupied), Card::Worker) =>
				{
					occupants.set(i, true);
					num_occupants += 1;
				}
				(Some(Marker::Roman), Card::Roman) =>
				{
					supporters.set(i, true);
					num_supporters += 1;
				}
				(Some(Marker::Worker), Card::Roman) =>
				{
					enemies.set(i, true);
					num_enemies += 1;
				}
				(Some(Marker::Occupied), Card::Roman) =>
				{
					supporters.set(i, true);
					num_supporters += 1;
				}
				(Some(Marker::DeadRoman), _) => (),
				(Some(Marker::DeadWorker), _) => (),
				(Some(Marker::FogOfWar), _) => (),
				(None, _) => (),
			}
		}
		if num_enemies == 1 && num_occupants == 0
		{
			self.kill_preview = enemies;
			self.attack_preview = Bitmap::new();
			if num_supporters == 0
			{
				self.kill_preview.set(region_id as usize, true);
			}
			else
			{
				self.attack_preview.set(region_id as usize, true);
			}
			self.support_preview = supporters;
		}
		else if num_enemies + num_occupants > 0
		{
			if num_supporters < num_enemies + num_occupants
			{
				self.kill_preview = Bitmap::new();
				self.kill_preview.set(region_id as usize, true);
				self.attack_preview = enemies | occupants;
			}
			else
			{
				self.kill_preview = enemies;
				self.attack_preview = Bitmap::new();
				if num_occupants > 0
				{
					self.kill_preview.set(region_id as usize, true);
				}
				else if num_enemies > 0
				{
					self.attack_preview.set(region_id as usize, true);
				}
			}
			self.support_preview = supporters;
		}
		else
		{
			self.gather_preview = supporters;
		}
	}

	fn count_remaining_spaces(&self) -> usize
	{
		self.region_data[0..(self.num_regions as usize)]
			.iter()
			.filter(|region| region.marker.is_none())
			.filter(|region| region.terrain_type != TerrainType::Water)
			.count()
	}

	fn shuffle(&mut self)
	{
		let num_villages = self.region_data[0..(self.num_regions as usize)]
			.iter()
			.filter(|region| region.terrain_type == TerrainType::Village)
			.filter(|region| region.marker == Some(Marker::Worker))
			.count();
		let num_workers = 4 + num_villages as u8;
		if self.grain >= num_workers
		{
			self.grain -= num_workers;
		}
		else if self.grain + self.wine >= num_workers
		{
			self.wine += self.grain;
			self.wine -= num_workers;
			self.grain = 0;
		}
		else
		{
			self.grain = 0;
			self.wine = 0;
		}
		let num_remaining_spaces = self.count_remaining_spaces();
		self.num_cards = std::cmp::min(
			num_workers + self.threat_level,
			num_remaining_spaces as u8,
		);
		for i in 0..self.num_cards
		{
			let card = if 2 * (i / 3) >= num_workers
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
	}

	fn pick_decrees(&mut self)
	{
		self.num_decrees = 0;
		if self.threat_level == 0
		{
			if self.tutorial.is_some()
			{
				self.decree_data[self.num_decrees as usize] = Decree::Dummy;
				self.num_decrees += 1;
			}
			else
			{
				let decree = Decree::NoWorkersAdjacent;
				self.decree_data[self.num_decrees as usize] = decree;
				self.num_decrees += 1;
			}
			return;
		}
		self.decree_data[self.num_decrees as usize] = Decree::AllRomansAdjacent;
		self.num_decrees += 1;
		if self.tutorial.is_some()
		{
			if self.threat_level >= 4
			{
				let decree = Decree::Regional {
					all_or_none: AllOrNone::All,
					marker: Marker::Roman,
					in_or_near: InOrNear::In,
					terrain_type: TerrainType::Grass,
				};
				self.decree_data[self.num_decrees as usize] = decree;
				self.num_decrees += 1;
			}
			if self.threat_level >= 9
			{
				let decree = Decree::Regional {
					all_or_none: AllOrNone::All,
					marker: Marker::Roman,
					in_or_near: InOrNear::Near,
					terrain_type: TerrainType::Forest,
				};
				self.decree_data[self.num_decrees as usize] = decree;
				self.num_decrees += 1;
			}
			self.decree_data[self.num_decrees as usize] =
				Decree::NoRomansInAmbush;
			self.num_decrees += 1;
		}
		else
		{
			let difficulty_level = match self.threat_level
			{
				0..=1 => 0,
				2..=3 => 1,
				4..=5 => 2,
				6 => 3,
				7.. => 4,
			};
			self.rng.shuffle(
				&mut self.decree_data
					[(self.num_decrees as usize)..TOTAL_NUM_DECREES],
			);
			self.num_decrees += difficulty_level;
			if let Some(offset) = self
				.decree_data
				.iter()
				.position(|decree| *decree == Decree::NoRomansInAmbush)
			{
				if offset != self.num_decrees as usize
				{
					self.decree_data[offset] =
						self.decree_data[self.num_decrees as usize];
					self.decree_data[self.num_decrees as usize] =
						Decree::NoRomansInAmbush;
				}
			}
			self.num_decrees += 1;
		}
	}

	fn place_marker(&mut self, map: &mut Map)
	{
		let (region_id, marker) = match self.hover_preview
		{
			Some(Preview::PlaceWorker {
				region_id,
				terrain_type,
			}) =>
			{
				self.score += 1;
				if self.kill_preview.get(region_id as usize)
				{
					(region_id, Marker::DeadWorker)
				}
				else if terrain_type == TerrainType::Grass
					&& self.gather_preview.len() >= 2
					&& self.wood >= VILLAGE_WOOD_COST
					&& self.gold >= VILLAGE_GOLD_COST
				{
					self.score += 9;
					self.grain += 1;
					self.wood -= VILLAGE_WOOD_COST;
					self.gold -= VILLAGE_GOLD_COST;
					for j in self.gather_preview.into_iter()
					{
						if self.region_data[j].marker != Some(Marker::Worker)
						{
							continue;
						}
						match self.region_data[j].terrain_type
						{
							TerrainType::Village => (),
							TerrainType::Grass => self.grain += 1,
							TerrainType::Forest => self.wood += 1,
							TerrainType::Hill => self.wine += 1,
							TerrainType::Mountain => self.gold += 1,
							TerrainType::Water => (),
						}
					}
					map.place_village(region_id);
					self.region_data[region_id as usize].terrain_type =
						TerrainType::Village;
					(region_id, Marker::Worker)
				}
				else
				{
					match terrain_type
					{
						TerrainType::Village =>
						{
							self.grain += 1;
							for j in self.gather_preview.into_iter()
							{
								if self.region_data[j].marker
									!= Some(Marker::Worker)
								{
									continue;
								}
								match self.region_data[j].terrain_type
								{
									TerrainType::Village => (),
									TerrainType::Grass => self.grain += 1,
									TerrainType::Forest => self.wood += 1,
									TerrainType::Hill => self.wine += 1,
									TerrainType::Mountain => self.gold += 1,
									TerrainType::Water => (),
								}
							}
						}
						TerrainType::Grass => self.grain += 1,
						TerrainType::Forest => self.wood += 1,
						TerrainType::Hill => self.wine += 1,
						TerrainType::Mountain => self.gold += 1,
						TerrainType::Water => (),
					}
					(region_id, Marker::Worker)
				}
			}
			Some(Preview::PlaceRoman { region_id }) =>
			{
				if self.kill_preview.get(region_id as usize)
				{
					(region_id, Marker::DeadRoman)
				}
				else
				{
					(region_id, Marker::Roman)
				}
			}
			_ => return,
		};
		trace(format!("clicked region {}", region_id));
		self.region_data[region_id as usize].marker = Some(marker);
		map.set_marker_in_region(region_id, Some(marker));
		for i in self.kill_preview.into_iter()
		{
			if i != region_id as usize
			{
				let killed = match self.region_data[i].marker
				{
					Some(Marker::Roman) => Some(Marker::DeadRoman),
					Some(Marker::Worker) => Some(Marker::DeadWorker),
					_ => None,
				};
				self.region_data[i].marker = killed;
				map.set_marker_in_region(i as i8, killed);
			}
		}
		let alive_marker = match marker
		{
			Marker::DeadRoman => Marker::Roman,
			Marker::DeadWorker => Marker::Worker,
			x => x,
		};
		let violated_decree_offset =
			(0..(self.num_decrees as usize)).find(|offset| {
				match self.decree_data[*offset]
				{
					Decree::Regional { .. }
						if self.region_data[region_id as usize]
							.terrain_type == TerrainType::Village =>
					{
						false
					}
					Decree::Regional { marker: m, .. } if m != alive_marker =>
					{
						false
					}
					Decree::Regional {
						all_or_none,
						marker: _,
						in_or_near,
						terrain_type,
					} =>
					{
						let matches = match in_or_near
						{
							InOrNear::In =>
							{
								self.region_data[region_id as usize]
									.terrain_type == terrain_type
							}
							InOrNear::Near => self.adjacency
								[region_id as usize]
								.into_iter()
								.any(|adj_id| {
									self.region_data[adj_id].terrain_type
										== terrain_type
								}),
						};
						match all_or_none
						{
							AllOrNone::All => !matches,
							AllOrNone::None => matches,
						}
					}
					Decree::NoWorkersAdjacent =>
					{
						alive_marker == Marker::Worker
							&& (!self.support_preview.is_empty()
								|| !self.gather_preview.is_empty())
					}
					Decree::AllRomansAdjacent =>
					{
						alive_marker == Marker::Roman
							&& self.support_preview.is_empty()
							&& self.gather_preview.is_empty()
					}
					Decree::NoRomansInAmbush => marker == Marker::DeadRoman,
					Decree::Dummy => false,
				}
			});
		if let Some(offset) = violated_decree_offset
		{
			self.num_cards = 0;
			if self.threat_level < MAX_THREAT_LEVEL
			{
				self.threat_level += 1;
			}
			self.state = State::DecreeViolated {
				decree_offset: offset as u8,
			};
		}
		else
		{
			self.card_offset += 1;
		}
	}

	pub fn draw(&mut self)
	{
		{
			let palette = match self.hover_preview
			{
				Some(Preview::HoverResource { terrain_type }) =>
				{
					match terrain_type
					{
						TerrainType::Water => palette::DEFAULT,
						TerrainType::Mountain => palette::SNOW,
						TerrainType::Hill => palette::WINE,
						TerrainType::Forest => palette::NATURE,
						TerrainType::Grass => palette::GOLD,
						TerrainType::Village => palette::GOLD,
					}
				}
				Some(Preview::PlaceWorker {
					region_id,
					terrain_type,
				}) =>
				{
					if self.kill_preview.get(region_id as usize)
					{
						palette::BLOOD
					}
					else
					{
						match terrain_type
						{
							TerrainType::Water => palette::DEFAULT,
							TerrainType::Mountain => palette::SNOW,
							TerrainType::Hill => palette::WINE,
							TerrainType::Forest => palette::NATURE,
							TerrainType::Grass => palette::GOLD,
							TerrainType::Village => palette::GOLD,
						}
					}
				}
				Some(Preview::PlaceRoman { region_id: _ }) => palette::ROMAN,
				Some(Preview::CannotPlaceRoman) => palette::ROMAN,
				Some(Preview::HoverDecrees) => palette::ROMAN,
				Some(Preview::HoverObjectives) => palette::WATER,
				None => match self.state
				{
					State::DecreeViolated { .. } => palette::ROMAN,
					State::TributeFailed => palette::ROMAN,
					State::GameOver => palette::ROMAN,
					_ => palette::DEFAULT,
				},
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

			let map = MAP.get_mut();
			map.draw(
				region_id,
				highlighted_terrain,
				self.kill_preview,
				self.attack_preview,
				self.support_preview,
				self.gather_preview,
			);
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
			Some(Preview::HoverResource { terrain_type }) =>
			{
				let x = match terrain_type
				{
					TerrainType::Village => 0,
					TerrainType::Grass => UI_X_GRAIN,
					TerrainType::Forest => UI_X_WOOD,
					TerrainType::Hill => UI_X_WINE,
					TerrainType::Mountain => UI_X_GOLD,
					TerrainType::Water => 0,
				};
				sprites::draw_backfill(x - 4, 0, 0);
			}
			Some(Preview::PlaceWorker {
				region_id,
				terrain_type,
			}) =>
			{
				if !self.kill_preview.get(region_id as usize)
				{
					let x = match terrain_type
					{
						TerrainType::Village => UI_X_GRAIN,
						TerrainType::Grass => UI_X_GRAIN,
						TerrainType::Forest => UI_X_WOOD,
						TerrainType::Hill => UI_X_WINE,
						TerrainType::Mountain => UI_X_GOLD,
						TerrainType::Water => 0,
					};
					sprites::draw_backfill(x - 4, 0, 0);
				}
			}
			Some(Preview::HoverDecrees)
			| Some(Preview::PlaceRoman { region_id: _ })
			| Some(Preview::CannotPlaceRoman) =>
			{
				sprites::draw_backfill((SCREEN_SIZE as i32) - 24, 0, 0);
			}
			Some(Preview::HoverObjectives) =>
			{
				sprites::draw_backfill(0, 0, 0);
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
		if self.threat_level < MAX_THREAT_LEVEL
		{
			draw_threat_value(self.threat_level, (SCREEN_SIZE as i32) - 8, 1);
		}
		else
		{
			text("X", (SCREEN_SIZE as i32) - 8, 1);
		}

		match self.hover_preview
		{
			Some(Preview::HoverObjectives) if self.tutorial.is_some() =>
			{
				unsafe { *DRAW_COLORS = 0x31 };
				rect(20, 20, 120, 130);

				if let Some(tutorial) = self.tutorial
				{
					tutorial.draw(25, 25);
				}
			}
			Some(Preview::HoverObjectives) =>
			{
				unsafe { *DRAW_COLORS = 0x31 };
				rect(20, 20, 120, 120);

				unsafe { *DRAW_COLORS = 0x03 };
				let x = 25;
				let mut y = 25;
				text("Banner", x, y);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_score_icon(x + 102, y - 1);
				unsafe { *DRAW_COLORS = 0x03 };
				text("1", x + 94, y);
				y += 10;
				text("Cost:  1", x, y);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_grain_icon(x + 66, y - 1);
				unsafe { *DRAW_COLORS = 0x03 };
				text("/ 1", x + 78, y);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_wine_icon(x + 104, y - 1);
				unsafe { *DRAW_COLORS = 0x03 };
				y += 22;
				text("Village", x, y);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_score_icon(x + 102, y - 1);
				unsafe { *DRAW_COLORS = 0x03 };
				text("10", x + 86, y);
				y += 10;
				text("Cost: 10", x, y);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_wood_icon(x + 66, y - 1);
				unsafe { *DRAW_COLORS = 0x03 };
				text("+ 5", x + 78, y);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_gold_icon(x + 104, y - 1);
				unsafe { *DRAW_COLORS = 0x03 };
				y += 10;
				text("Place", x, y);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_flag(x + 46 + 2, y + 9, 2);
				unsafe { *DRAW_COLORS = 0x03 };
				text("in", x + 56, y);
				unsafe { *DRAW_COLORS = 0x1320 };
				sprites::draw_grass(x + 76, y + 1, 0);
				sprites::draw_grass(x + 76 + 4, y + 6, 4);
				sprites::draw_grass(x + 76 + 8, y + 1, 5);
				unsafe { *DRAW_COLORS = 0x03 };
				y += 10;
				text("with 2", x, y);
				unsafe { *DRAW_COLORS = 0x2310 };
				sprites::draw_gathering_town(x + 54 + 2, y + 1);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_flag(x + 64 + 2, y + 9, 2);
				unsafe { *DRAW_COLORS = 0x03 };
				text(".", x + 74, y);
				y += 10;
				text("Once built,", x, y);
				y += 10;
				text("gain ", x, y);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_grain_icon(x + 36, y - 1);
				sprites::draw_wood_icon(x + 44, y - 1);
				sprites::draw_gold_icon(x + 52, y - 1);
				sprites::draw_wine_icon(x + 60, y - 1);
				unsafe { *DRAW_COLORS = 0x03 };
				text("of", x + 72, y);
				unsafe { *DRAW_COLORS = 0x2310 };
				sprites::draw_gathering_town(x + 92 + 2, y + 1);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_flag(x + 102 + 2, y + 9, 2);
				unsafe { *DRAW_COLORS = 0x03 };
				y += 10;
				text("and +1", x, y);
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_flag(x + 52 + 2, y + 9, 2);
				unsafe { *DRAW_COLORS = 0x03 };
				text(".", x + 62, y);
				y += 10;
				text("Ignore decree.", x, y);
			}
			Some(Preview::HoverDecrees) =>
			{
				unsafe { *DRAW_COLORS = 0x31 };
				rect(60, 20, 90, 130);

				unsafe { *DRAW_COLORS = 4 };
				let x = 63;
				let mut y = 25;
				text("IMPERIAL", x + 10, y);
				y += 8;
				text("DECREE", x + 10 + 8, y);
				unsafe { *DRAW_COLORS = 3 };
				for decree in &self.decree_data[0..(self.num_decrees as usize)]
				{
					y += 15;
					decree.draw(x, y);
				}
				if self.tribute > 0
				{
					y = 139;
					unsafe { *DRAW_COLORS = 0x03 };
					text("Tribute:", x, y);
					draw_threat_value(self.tribute, x + 67, y);
					unsafe { *DRAW_COLORS = 0x3210 };
					sprites::draw_wine_icon(x + 76, y - 1);
				}
			}
			_ => match self.state
			{
				State::DecreeViolated { decree_offset } =>
				{
					unsafe { *DRAW_COLORS = 0x31 };
					rect(10, 60, 140, 58);
					unsafe { *DRAW_COLORS = 0x03 };
					let x = 15;
					let mut y = 60 + 6;
					text("Imperial decrees", x, y);
					y += 8;
					text("are absolute!", x, y);
					y += 15;
					text("(", x, y);
					let decree = &self.decree_data[decree_offset as usize];
					let w = decree.draw(x + 8, y);
					unsafe { *DRAW_COLORS = 0x03 };
					text(")", x + 4 + w, y);
					y += 15;
					unsafe { *DRAW_COLORS = 0x40 };
					sprites::draw_backfill(x + 49, y - 1, 0);
					unsafe { *DRAW_COLORS = 0x03 };
					text("+1", x + 52, y);
					unsafe { *DRAW_COLORS = 0x3210 };
					sprites::draw_wreath_icon(x + 70, y - 1);
					unsafe { *DRAW_COLORS = 0x03 };
				}
				State::TributeSkipped =>
				{
					unsafe { *DRAW_COLORS = 0x31 };
					rect(10, 60, 140, 35);
					unsafe { *DRAW_COLORS = 0x03 };
					let x = 15;
					let mut y = 60 + 6;
					text("A new year", x, y);
					y += 8;
					text("begins.", x, y);
				}
				State::TributePaid =>
				{
					unsafe { *DRAW_COLORS = 0x31 };
					rect(10, 60, 140, 35);
					unsafe { *DRAW_COLORS = 0x03 };
					let x = 15;
					let mut y = 60 + 6;
					text("Tribute paid.", x, y);
					y += 15;
					text("New tribute:", x, y);
					draw_threat_value(self.tribute, x + 114, y);
					unsafe { *DRAW_COLORS = 0x3210 };
					sprites::draw_wine_icon(x + 123, y - 1);
				}
				State::TributeFailed =>
				{
					unsafe { *DRAW_COLORS = 0x31 };
					rect(10, 60, 140, 58);
					unsafe { *DRAW_COLORS = 0x03 };
					let x = 15;
					let mut y = 60 + 6;
					text("You dare refuse", x, y);
					y += 8;
					text("to pay tribute?!", x, y);
					y += 15;
					unsafe { *DRAW_COLORS = 0x40 };
					sprites::draw_backfill(x + 49, y - 1, 0);
					unsafe { *DRAW_COLORS = 0x03 };
					text("+1", x + 52, y);
					unsafe { *DRAW_COLORS = 0x3210 };
					sprites::draw_wreath_icon(x + 70, y - 1);
					unsafe { *DRAW_COLORS = 0x03 };
					y += 15;
					text("New tribute:", x, y);
					draw_threat_value(self.tribute, x + 114, y);
					unsafe { *DRAW_COLORS = 0x3210 };
					sprites::draw_wine_icon(x + 123, y - 1);
				}
				State::GameOver =>
				{
					unsafe { *DRAW_COLORS = 0x31 };
					rect(10, 60, 140, 35);
					unsafe { *DRAW_COLORS = 0x03 };
					let x = 15;
					let mut y = 60 + 6;
					text("You have been", x, y);
					y += 8;
					text("eradicated.", x, y);
				}
				_ => (),
			},
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

struct Cursor
{
	row: u8,
	col: u8,
	mouse_x: i16,
	mouse_y: i16,
}
