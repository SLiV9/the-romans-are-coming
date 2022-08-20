//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use crate::level::Marker;
use crate::level::TerrainType;
use crate::sprites;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllOrNone
{
	All,
	None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InOrNear
{
	In,
	Near,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decree
{
	Regional
	{
		all_or_none: AllOrNone,
		marker: Marker,
		in_or_near: InOrNear,
		terrain_type: TerrainType,
	},
	AllRomansAdjacent,
	NoWorkersAdjacent,
	NoRomansInAmbush,
	Dummy,
}

impl Decree
{
	pub fn draw(&self, x: i32, y: i32) -> i32
	{
		draw_parts(&self.parts(), x, y)
	}

	fn parts(&self) -> [Part; 5]
	{
		match self
		{
			Decree::Regional {
				all_or_none,
				marker,
				in_or_near,
				terrain_type,
			} => [
				all_or_none.into(),
				marker.into(),
				in_or_near.into(),
				terrain_type.into(),
				Part::Period,
			],
			Decree::AllRomansAdjacent => [
				AllOrNone::All.into(),
				Marker::Roman.into(),
				InOrNear::Near.into(),
				Marker::Roman.into(),
				Part::Period,
			],
			Decree::NoWorkersAdjacent => [
				AllOrNone::None.into(),
				Marker::Worker.into(),
				InOrNear::Near.into(),
				Marker::Worker.into(),
				Part::Period,
			],
			Decree::NoRomansInAmbush => [
				AllOrNone::None.into(),
				Marker::Roman.into(),
				Part::Word("as"),
				Marker::DeadRoman.into(),
				Part::Period,
			],
			Decree::Dummy => [
				Part::Word("Place"),
				Marker::Worker.into(),
				Marker::Worker.into(),
				Marker::Worker.into(),
				Marker::Worker.into(),
			],
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Tutorial
{
	PlaceBanners,
}

impl Tutorial
{
	pub fn draw(&self, x: i32, y: i32)
	{
		match self
		{
			Tutorial::PlaceBanners =>
			{
				let parts = &[
					Part::Word("Greetings,"),
					Part::Newline,
					Part::Word("your highness!"),
					Part::Newline,
					Part::Newline,
					Part::Word("Place"),
					Marker::Worker.into(),
					Part::Newline,
					Part::Word("to score 1"),
					Icon::Score.into(),
					Part::Newline,
					Part::Word("and gather"),
					Part::Newline,
					Part::Word("1"),
					Icon::Grain.into(),
					Part::Word("/"),
					Icon::Wood.into(),
					Part::Word("/"),
					Icon::Gold.into(),
					Part::Word("/"),
					Icon::Wine.into(),
					Part::Period,
				];
				draw_parts(parts, x, y);
			}
		}
	}
}

fn draw_parts(parts: &[Part], x: i32, y: i32) -> i32
{
	let mut dx = 0;
	let mut dy = 0;
	for part in parts
	{
		dx += part.draw(x + dx, y + dy);
		match part
		{
			Part::Newline =>
			{
				dx = 0;
				dy += 10;
			}
			_ => (),
		}
	}
	dx
}

#[derive(Debug, Clone, Copy)]
enum Icon
{
	Score,
	Grain,
	Wood,
	Wine,
	Gold,
}

#[derive(Debug)]
enum Part
{
	Word(&'static str),
	Marker(Marker),
	TerrainType(TerrainType),
	Icon(Icon),
	Period,
	Newline,
}

impl From<AllOrNone> for Part
{
	fn from(x: AllOrNone) -> Part
	{
		match x
		{
			AllOrNone::All => Part::Word("All"),
			AllOrNone::None => Part::Word("No"),
		}
	}
}

impl From<InOrNear> for Part
{
	fn from(x: InOrNear) -> Part
	{
		match x
		{
			InOrNear::In => Part::Word("in"),
			InOrNear::Near => Part::Word("near"),
		}
	}
}

impl From<Marker> for Part
{
	fn from(x: Marker) -> Part
	{
		Part::Marker(x)
	}
}

impl From<TerrainType> for Part
{
	fn from(x: TerrainType) -> Part
	{
		Part::TerrainType(x)
	}
}

impl From<Icon> for Part
{
	fn from(x: Icon) -> Part
	{
		Part::Icon(x)
	}
}

impl<T: Copy + Into<Part>> From<&T> for Part
{
	fn from(x: &T) -> Part
	{
		(*x).into()
	}
}

impl Part
{
	fn draw(&self, x: i32, y: i32) -> i32
	{
		match self
		{
			Part::Word(word) =>
			{
				unsafe { *DRAW_COLORS = 0x03 };
				text(word, x, y);
				(word.len() as i32) * 8 + 4
			}
			Part::Period =>
			{
				unsafe { *DRAW_COLORS = 0x03 };
				text(".", x - 4, y);
				4
			}
			Part::Newline => 0,
			Part::Marker(marker) =>
			{
				let flag = match marker
				{
					Marker::Roman => 0,
					Marker::DeadRoman => 1,
					Marker::Worker => 2,
					Marker::DeadWorker => 3,
					Marker::Occupied => 0,
				};
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_flag(x + 2, y + 8, flag);
				10
			}
			Part::TerrainType(TerrainType::Village) =>
			{
				unsafe { *DRAW_COLORS = 0x1320 };
				sprites::draw_house(x, y + 5, 0);
				sprites::draw_house(x + 4, y + 6, 0);
				sprites::draw_house(x + 8, y + 5, 0);
				12
			}
			Part::TerrainType(TerrainType::Grass) =>
			{
				unsafe { *DRAW_COLORS = 0x1320 };
				sprites::draw_grass(x, y + 1, 0);
				sprites::draw_grass(x + 4, y + 6, 4);
				sprites::draw_grass(x + 8, y + 1, 5);
				12
			}
			Part::TerrainType(TerrainType::Forest) =>
			{
				unsafe { *DRAW_COLORS = 0x1320 };
				sprites::draw_tree(x, y + 6, 0);
				sprites::draw_tree(x + 4, y + 7, 0);
				sprites::draw_tree(x + 8, y + 6, 0);
				12
			}
			Part::TerrainType(TerrainType::Hill) =>
			{
				unsafe { *DRAW_COLORS = 0x1320 };
				sprites::draw_hill(x, y + 4, 1);
				sprites::draw_hill(x + 3, y + 8, 0);
				sprites::draw_hill(x + 8, y + 3, 2);
				12
			}
			Part::TerrainType(TerrainType::Mountain) =>
			{
				unsafe { *DRAW_COLORS = 0x1320 };
				sprites::draw_mountain(x + 2, y + 8, 0);
				12
			}
			Part::TerrainType(TerrainType::Water) =>
			{
				unsafe { *DRAW_COLORS = 0x20 };
				sprites::draw_surface(x + 4, y, 0);
				sprites::draw_surface(x + 2, y + 2, 0);
				sprites::draw_surface(x + 6, y + 2, 0);
				sprites::draw_surface(x + 4, y + 4, 0);
				sprites::draw_surface(x + 2, y + 6, 0);
				sprites::draw_surface(x + 6, y + 6, 0);
				sprites::draw_surface(x + 4, y + 8, 0);
				unsafe { *DRAW_COLORS = 0x1320 };
				sprites::draw_boat(x + 4, y + 4, 0);
				16
			}
			Part::Icon(Icon::Score) =>
			{
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_score_icon(x - 4, y - 1);
				8
			}
			Part::Icon(Icon::Grain) =>
			{
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_grain_icon(x - 2, y - 1);
				8
			}
			Part::Icon(Icon::Wood) =>
			{
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_wood_icon(x - 2, y - 1);
				8
			}
			Part::Icon(Icon::Wine) =>
			{
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_wine_icon(x - 2, y - 1);
				8
			}
			Part::Icon(Icon::Gold) =>
			{
				unsafe { *DRAW_COLORS = 0x3210 };
				sprites::draw_gold_icon(x - 2, y - 1);
				8
			}
		}
	}
}
