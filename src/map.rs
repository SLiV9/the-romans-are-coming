//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use fastrand;
use perlin2d::PerlinNoise2D;

pub const MAP_SIZE: usize = 160;
pub const BITMAP_SIZE: usize = MAP_SIZE * MAP_SIZE / 8;
pub const QUADMAP_SIZE: usize = MAP_SIZE * MAP_SIZE / 4;
pub const GRID_SIZE: usize = 13;
pub const GRID_CELL_SIZE: usize = MAP_SIZE / GRID_SIZE;

const NOISE_OCTAVES: i32 = 5;
const NOISE_AMPLITUDE: f64 = 50.0;
const NOISE_FREQUENCY: f64 = 1.0;
const NOISE_FREQUENCY_FOREST: f64 = 1.0;
const NOISE_PERSISTENCE: f64 = 1.0;
const NOISE_PERSISTENCE_FOREST: f64 = 2.0;
const NOISE_LACUNARITY: f64 = 2.0;
const NOISE_SCALE: (f64, f64) = (MAP_SIZE as f64, MAP_SIZE as f64);
const NOISE_BIAS_ELEVATION: f64 = 45.0;
const NOISE_BIAS_FOREST: f64 = 0.0;

pub struct Map
{
	water_bitmap: [u8; BITMAP_SIZE],
	shade_bitmap: [u8; BITMAP_SIZE],
	ink_bitmap: [u8; BITMAP_SIZE],
	cells: [[Cell; GRID_SIZE]; GRID_SIZE],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerrainType
{
	Grass,
	Forest,
	Hill,
	Mountain,
	Water,
}

#[derive(Debug, Clone, Copy, Default)]
struct Cell
{
	centroid_x: u8,
	centroid_y: u8,
	n_water: u8,
	n_mountain: u8,
	n_hill: u8,
	n_forest: u8,
	n_grass: u8,
}

const EMPTY_CELL: Cell = Cell {
	centroid_x: 0,
	centroid_y: 0,
	n_water: 0,
	n_mountain: 0,
	n_hill: 0,
	n_forest: 0,
	n_grass: 0,
};

impl Cell
{
	fn terrain_type(&self) -> Option<TerrainType>
	{
		if self.n_water > 0
		{
			Some(TerrainType::Water)
		}
		else if self.n_mountain > 0
		{
			Some(TerrainType::Mountain)
		}
		else if self.n_hill > 0
		{
			Some(TerrainType::Hill)
		}
		else if self.n_forest > 0
		{
			Some(TerrainType::Forest)
		}
		else if self.n_grass > 0
		{
			Some(TerrainType::Grass)
		}
		else
		{
			None
		}
	}

	fn clear_terrain(&mut self)
	{
		self.n_water = 0;
		self.n_mountain = 0;
		self.n_hill = 0;
		self.n_forest = 0;
		self.n_grass = 0;
	}

	fn make_more_important(&mut self)
	{
		if !self.is_crucial()
		{
			self.n_water *= 2;
			self.n_mountain *= 2;
			self.n_hill *= 2;
			self.n_forest *= 2;
			self.n_grass *= 2;
		}
	}

	fn is_crucial(&self) -> bool
	{
		self.n_water >= 8
			|| self.n_mountain >= 8
			|| self.n_hill >= 8
			|| self.n_forest >= 8
			|| self.n_grass >= 8
	}
}

impl Map
{
	pub const fn empty() -> Self
	{
		Self {
			water_bitmap: [0; BITMAP_SIZE],
			shade_bitmap: [0; BITMAP_SIZE],
			ink_bitmap: [0; BITMAP_SIZE],
			cells: [[EMPTY_CELL; GRID_SIZE]; GRID_SIZE],
		}
	}

	pub fn generate(&mut self, rng: &mut fastrand::Rng)
	{
		let seed = rng.u16(..) as i32;
		let elevation = PerlinNoise2D::new(
			NOISE_OCTAVES,
			NOISE_AMPLITUDE,
			NOISE_FREQUENCY,
			NOISE_PERSISTENCE,
			NOISE_LACUNARITY,
			NOISE_SCALE,
			NOISE_BIAS_ELEVATION,
			seed,
		);
		let seed = rng.u16(..) as i32;
		let forest = PerlinNoise2D::new(
			NOISE_OCTAVES,
			NOISE_AMPLITUDE,
			NOISE_FREQUENCY_FOREST,
			NOISE_PERSISTENCE_FOREST,
			NOISE_LACUNARITY,
			NOISE_SCALE,
			NOISE_BIAS_FOREST,
			seed,
		);
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				let (x, y) = pick_random_centroid_xy_at_rc(r, c, rng);
				cell.centroid_x = x as u8;
				cell.centroid_y = y as u8;
				cell.clear_terrain();
			}
		}
		for y in 0..MAP_SIZE
		{
			for x in 0..MAP_SIZE
			{
				let (r, c, distance) =
					self.closest_rc_to_xy(x as i32, y as i32);
				let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let f = forest.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let terrain_type = if e > 70.0
				{
					TerrainType::Mountain
				}
				else if e > 50.0
				{
					if f > 10.0
					{
						TerrainType::Forest
					}
					else
					{
						TerrainType::Hill
					}
				}
				else if e > 0.0
				{
					if f > 200.0
					{
						TerrainType::Forest
					}
					else
					{
						TerrainType::Grass
					}
				}
				else
				{
					TerrainType::Water
				};
				if terrain_type == TerrainType::Water
				{
					draw_on_bitmap(&mut self.water_bitmap, x, y);
				}
				else
				{
					erase_on_bitmap(&mut self.water_bitmap, x, y);
				}
				if terrain_type == TerrainType::Mountain
				{
					draw_on_bitmap(&mut self.ink_bitmap, x, y);
				}
				else
				{
					erase_on_bitmap(&mut self.ink_bitmap, x, y);
				}
				if terrain_type == TerrainType::Forest
					|| terrain_type == TerrainType::Hill && ((x + y) % 2 == 1)
				{
					draw_on_bitmap(&mut self.shade_bitmap, x, y);
				}
				else
				{
					erase_on_bitmap(&mut self.shade_bitmap, x, y);
				}
				let cell = &mut self.cells[r][c];
				match terrain_type
				{
					TerrainType::Grass =>
					{
						if cell.n_grass < 250
						{
							cell.n_grass += 1;
						}
					}
					TerrainType::Forest =>
					{
						if cell.n_forest < 250
						{
							cell.n_forest += 1;
						}
					}
					TerrainType::Hill =>
					{
						if cell.n_hill < 250
						{
							cell.n_hill += 1;
						}
					}
					TerrainType::Mountain =>
					{
						if cell.n_mountain < 250
						{
							cell.n_mountain += 1;
						}
					}
					TerrainType::Water =>
					{
						if cell.n_water < 250
						{
							cell.n_water += 1;
						}
					}
				}
			}
		}
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				let n_total = (cell.n_water as u16)
					+ (cell.n_mountain as u16)
					+ (cell.n_hill as u16)
					+ (cell.n_forest as u16)
					+ (cell.n_grass as u16);
				let n_total = std::cmp::min(n_total, 250) as u8;
				let mut te = 0.0;
				let mut tf = 0.0;
				let terrain_type = if cell.n_water > n_total / 2
				{
					te = -100.0;
					TerrainType::Water
				}
				else if cell.n_mountain > n_total / 2
				{
					te = 1000.0;
					TerrainType::Mountain
				}
				else if cell.n_hill > n_total / 2
				{
					te = 60.0;
					tf = -200.0;
					TerrainType::Hill
				}
				else if cell.n_forest > n_total / 2
				{
					te = 20.0;
					tf = 400.0;
					TerrainType::Forest
				}
				else if cell.n_grass > n_total / 2
				{
					te = 10.0;
					tf = -200.0;
					TerrainType::Grass
				}
				else
				{
					// This is a bad cell.
					cell.clear_terrain();
					continue;
				};
				let x = cell.centroid_x as usize;
				let y = cell.centroid_y as usize;
				let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let f = forest.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let mut badness = (te - e).abs() + (tf - f).abs();
				for _i in 0..10
				{
					if badness < 2.0
					{
						break;
					}
					let (x, y) = pick_random_centroid_xy_at_rc(r, c, rng);
					let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
					let f = forest.get_noise(x as f64 + 0.5, y as f64 + 0.5);
					let b = (te - e).abs() + (tf - f).abs();
					if b + 1.0 < badness
					{
						cell.centroid_x = x as u8;
						cell.centroid_y = y as u8;
						badness = b;
					}
				}
				cell.clear_terrain();
				match terrain_type
				{
					TerrainType::Grass => cell.n_grass = 1,
					TerrainType::Forest => cell.n_forest = 1,
					TerrainType::Hill => cell.n_hill = 1,
					TerrainType::Mountain => cell.n_mountain = 1,
					TerrainType::Water => cell.n_water = 1,
				}
			}
		}
		{
			self.cells[0][0].clear_terrain();
			self.cells[GRID_SIZE - 1][0].clear_terrain();
			self.cells[0][GRID_SIZE - 1].clear_terrain();
			self.cells[GRID_SIZE - 1][GRID_SIZE - 1].clear_terrain();
		}
		for r in (1..(GRID_SIZE - 1)).step_by(2)
		{
			for c in 1..(GRID_SIZE - 1)
			{
				self.merge_cell(r, c, rng);
			}
		}
		for r in (2..(GRID_SIZE - 2)).step_by(2)
		{
			for c in (1..(GRID_SIZE - 1)).step_by(2)
			{
				self.merge_cell(r, c, rng);
			}
		}
		for i in 0..GRID_SIZE
		{
			self.merge_cell(i, 0, rng);
			self.merge_cell(i, GRID_SIZE - 1, rng);
			self.merge_cell(0, i, rng);
			self.merge_cell(GRID_SIZE - 1, i, rng);
		}
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &self.cells[r][c];
				let x = cell.centroid_x as usize;
				let y = cell.centroid_y as usize;
				if cell.terrain_type().is_some()
				{
					draw_on_bitmap(&mut self.ink_bitmap, x, y);
					draw_on_bitmap(&mut self.ink_bitmap, x + 1, y);
					draw_on_bitmap(&mut self.ink_bitmap, x, y + 1);
					draw_on_bitmap(&mut self.ink_bitmap, x + 1, y + 1);
					if cell.n_forest > 0 || cell.n_water > 0
					{
						erase_on_bitmap(&mut self.ink_bitmap, x + 1, y + 1);
					}
					if cell.n_mountain > 0 || cell.n_water > 0
					{
						erase_on_bitmap(&mut self.ink_bitmap, x, y + 1);
					}
					if cell.n_hill > 0
					{
						erase_on_bitmap(&mut self.ink_bitmap, x + 1, y);
					}
				}
				else
				{
					draw_on_bitmap(&mut self.ink_bitmap, x, y);
				}
			}
		}
	}

	pub fn draw(&self)
	{
		let x = 0;
		let y = 0;
		unsafe { *DRAW_COLORS = 0x20 };
		blit(
			&self.shade_bitmap,
			x,
			y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x40 };
		blit(
			&self.water_bitmap,
			x,
			y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x30 };
		blit(
			&self.ink_bitmap,
			x,
			y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
	}

	fn closest_rc_to_xy(&self, x: i32, y: i32) -> (usize, usize, f64)
	{
		let xx = std::cmp::max(0, x) as usize;
		let yy = std::cmp::max(0, y) as usize;
		let r0 = std::cmp::min(yy / GRID_CELL_SIZE, GRID_SIZE - 2);
		let c0 = std::cmp::min(xx / GRID_CELL_SIZE, GRID_SIZE - 2);
		(0..4)
			.map(|i| {
				let r = r0 + 1 - i / 2;
				let c = c0 + 1 - i % 2;
				let cell = &self.cells[r][c];
				let ddx = cell.centroid_x as i32 - x;
				let ddy = cell.centroid_y as i32 - y;
				let sqdis: i32 = ddx * ddx + ddy * ddy;
				(r, c, sqdis)
			})
			.min_by_key(|(_r, _c, sqdis)| *sqdis)
			.map(|(r, c, sqdis)| (r, c, (sqdis as f64).sqrt()))
			.unwrap()
	}

	fn merge_cell(&mut self, r: usize, c: usize, rng: &mut fastrand::Rng)
	{
		let tt = self.cells[r][c].terrain_type();
		if tt.is_none()
		{
			return;
		}
		else if self.cells[r][c].is_crucial()
		{
			return;
		}
		let mut adjacents = [(1, 0), (-1, 0), (0, 1), (0, -1)];
		rng.shuffle(&mut adjacents);
		for (dr, dc) in adjacents
		{
			if (r == 0 && dr < 0)
				|| (c == 0 && dc < 0)
				|| (r + 1 == GRID_SIZE && dr > 0)
				|| (c + 1 == GRID_SIZE && dc > 0)
			{
				continue;
			};
			let rr = ((r as i32) + dr) as usize;
			let cc = ((c as i32) + dc) as usize;
			if self.cells[rr][cc].terrain_type() == tt
			{
				if self.cells[rr][cc].is_crucial()
				{
					self.cells[r][c].clear_terrain();
				}
				else if rng.bool()
				{
					self.cells[rr][cc].clear_terrain();
					self.cells[r][c].make_more_important();
				}
				else
				{
					self.cells[r][c].clear_terrain();
					self.cells[rr][cc].make_more_important();
				}
				break;
			}
		}
	}
}

fn draw_on_bitmap(bitmap: &mut [u8; BITMAP_SIZE], x: usize, y: usize)
{
	let offset = y * MAP_SIZE + x;
	let byte_offset = offset / 8;
	assert!(byte_offset < BITMAP_SIZE);
	let bit_shift = offset % 8;
	bitmap[byte_offset] |= 0b10000000 >> bit_shift;
}

fn erase_on_bitmap(bitmap: &mut [u8; BITMAP_SIZE], x: usize, y: usize)
{
	let offset = y * MAP_SIZE + x;
	let byte_offset = offset / 8;
	assert!(byte_offset < BITMAP_SIZE);
	let bit_shift = offset % 8;
	bitmap[byte_offset] &= !(0b10000000 >> bit_shift);
}

fn draw_on_quadmap(
	quadmap: &mut [u8; QUADMAP_SIZE],
	x: usize,
	y: usize,
	value: u8,
)
{
	let offset = y * MAP_SIZE + x;
	let byte_offset = offset / 4;
	assert!(byte_offset < QUADMAP_SIZE);
	let bit_shift = 2 * (offset % 4);
	quadmap[byte_offset] &= !(0b11000000 >> bit_shift);
	let value = value & 0b11;
	quadmap[byte_offset] |= (value << 6) >> bit_shift;
}

fn pick_random_centroid_xy_at_rc(
	r: usize,
	c: usize,
	rng: &mut fastrand::Rng,
) -> (usize, usize)
{
	let padding = 3;
	let inner_x = padding + rng.usize(0..(GRID_CELL_SIZE - 2 * padding));
	let inner_y = padding + rng.usize(0..(GRID_CELL_SIZE - 2 * padding));
	let x = c * GRID_CELL_SIZE + inner_x;
	let y = r * GRID_CELL_SIZE + inner_y;
	let x = std::cmp::min(
		std::cmp::max(GRID_CELL_SIZE / 2 + padding, x),
		MAP_SIZE - 1 - padding - GRID_CELL_SIZE / 2,
	);
	let y = std::cmp::min(
		std::cmp::max(GRID_CELL_SIZE / 2 + padding, y),
		MAP_SIZE - 1 - padding - GRID_CELL_SIZE / 2,
	);
	(x, y)
}
