//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use crate::sprites;

use fastrand;
use perlin2d::PerlinNoise2D;

pub const MAP_SIZE: usize = 160;
pub const BITMAP_SIZE: usize = MAP_SIZE * MAP_SIZE / 8;
pub const GRID_SIZE: usize = 16;
pub const GRID_CELL_SIZE: usize = MAP_SIZE / GRID_SIZE;
pub const PROP_GRID_SIZE: usize = 40;
pub const PROP_GRID_CELL_SIZE: usize = MAP_SIZE / PROP_GRID_SIZE;
pub const PROPMAP_SIZE: usize = PROP_GRID_SIZE * PROP_GRID_SIZE / 2;

const MIN_DISTANCE_BETWEEN_REGIONS: i32 = 20;
const MAX_DISTANCE_BETWEEN_REGIONS: i32 = 40;

const NOISE_OCTAVES: i32 = 5;
const NOISE_AMPLITUDE: f64 = 50.0;
const NOISE_FREQUENCY_ELEVATION: f64 = 1.0;
const NOISE_FREQUENCY_FOREST: f64 = 1.0;
const NOISE_FREQUENCY_OCCUPATION: f64 = 2.0;
const NOISE_PERSISTENCE_ELEVATION: f64 = 1.0;
const NOISE_PERSISTENCE_FOREST: f64 = 2.0;
const NOISE_PERSISTENCE_OCCUPATION: f64 = 2.0;
const NOISE_LACUNARITY: f64 = 2.0;
const NOISE_SCALE: (f64, f64) = (MAP_SIZE as f64, MAP_SIZE as f64);
const NOISE_BIAS_ELEVATION: f64 = 45.0;
const NOISE_BIAS_FOREST: f64 = 0.0;
const NOISE_BIAS_OCCUPATION: f64 = 0.0;

const ELEVATION_THRESHOLD_MOUNTAIN: f64 = 70.0;
const ELEVATION_THRESHOLD_HILL: f64 = 50.0;
const ELEVATION_THRESHOLD_WATER: f64 = 0.0;
const FOREST_THRESHOLD_ON_HILL: f64 = 10.0;
const FOREST_THRESHOLD_ON_GRASS: f64 = 200.0;

pub struct Map
{
	water_bitmap: [u8; BITMAP_SIZE],
	mountain_bitmap: [u8; BITMAP_SIZE],
	debug_bitmap: [u8; BITMAP_SIZE],
	ink_bitmap: [u8; BITMAP_SIZE],
	occupation_bitmap: [u8; BITMAP_SIZE],
	propmap: [u8; PROPMAP_SIZE],
	cells: [[Cell; GRID_SIZE]; GRID_SIZE],
	occupation_noise: Option<PerlinNoise2D>,
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

impl Map
{
	pub const fn empty() -> Self
	{
		Self {
			water_bitmap: [0; BITMAP_SIZE],
			mountain_bitmap: [0; BITMAP_SIZE],
			debug_bitmap: [0; BITMAP_SIZE],
			ink_bitmap: [0; BITMAP_SIZE],
			occupation_bitmap: [0; BITMAP_SIZE],
			propmap: [0; PROPMAP_SIZE],
			cells: [[EMPTY_CELL; GRID_SIZE]; GRID_SIZE],
			occupation_noise: None,
		}
	}

	pub fn generate(&mut self, rng: &mut fastrand::Rng)
	{
		let seed = rng.u16(..) as i32;
		let elevation = PerlinNoise2D::new(
			NOISE_OCTAVES,
			NOISE_AMPLITUDE,
			NOISE_FREQUENCY_ELEVATION,
			NOISE_PERSISTENCE_ELEVATION,
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
		let seed = rng.u16(..) as i32;
		self.occupation_noise = Some(PerlinNoise2D::new(
			NOISE_OCTAVES,
			NOISE_AMPLITUDE,
			NOISE_FREQUENCY_OCCUPATION,
			NOISE_PERSISTENCE_OCCUPATION,
			NOISE_LACUNARITY,
			NOISE_SCALE,
			NOISE_BIAS_OCCUPATION,
			seed,
		));
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				let (x, y) = pick_random_centroid_xy_at_rc(r, c, rng);
				cell.centroid_x = x as u8;
				cell.centroid_y = y as u8;
				cell.contents = Contents::Tally {
					n_water: 0,
					n_mountain: 0,
					n_hill: 0,
					n_forest: 0,
					n_grass: 0,
				};
			}
		}
		for y in 0..MAP_SIZE
		{
			for x in 0..MAP_SIZE
			{
				let (r, c, _distance) =
					self.closest_rc_to_xy(x as i32, y as i32);
				let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let f = forest.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let terrain_type = if e > ELEVATION_THRESHOLD_MOUNTAIN
				{
					TerrainType::Mountain
				}
				else if e > ELEVATION_THRESHOLD_HILL
				{
					if f > FOREST_THRESHOLD_ON_HILL
					{
						TerrainType::Forest
					}
					else
					{
						TerrainType::Hill
					}
				}
				else if e > ELEVATION_THRESHOLD_WATER
				{
					if f > FOREST_THRESHOLD_ON_GRASS
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
					draw_on_bitmap(&mut self.mountain_bitmap, x, y);
				}
				else
				{
					erase_on_bitmap(&mut self.mountain_bitmap, x, y);
				}
				if (terrain_type == TerrainType::Forest
					&& ((x + y) % 2 == 0)
					&& (x % 4 == 0) && (y % 4 == 0))
					|| (terrain_type == TerrainType::Hill
						&& ((x + y) % 2 == 0) && (x % 4 == 0 || y % 4 == 0))
					|| (terrain_type == TerrainType::Mountain
						&& ((x + y) % 2 == 0))
				{
					draw_on_bitmap(&mut self.debug_bitmap, x, y);
				}
				else
				{
					erase_on_bitmap(&mut self.debug_bitmap, x, y);
				}
				self.cells[r][c].add_tally(terrain_type);
			}
		}
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				cell.realize_terrain();
			}
		}
		for i in 0..GRID_SIZE
		{
			self.force_merge_edge(i, 0, rng);
			self.force_merge_edge(i, GRID_SIZE - 1, rng);
			self.force_merge_edge(0, i, rng);
			self.force_merge_edge(GRID_SIZE - 1, i, rng);
		}
		for r in 1..(GRID_SIZE - 1)
		{
			for c in 1..(GRID_SIZE - 1)
			{
				if (r + c) % 2 == 0
				{
					self.soft_merge_cell(r, c, rng);
				}
			}
		}
		for r in (1..(GRID_SIZE - 1)).step_by(2)
		{
			for c in 1..(GRID_SIZE - 1)
			{
				self.soft_merge_cell(r, c, rng);
			}
		}
		for r in (2..(GRID_SIZE - 2)).step_by(2)
		{
			for c in (1..(GRID_SIZE - 1)).step_by(2)
			{
				self.soft_merge_cell(r, c, rng);
			}
		}
		let mut cell_goodness = [0i8; GRID_SIZE * GRID_SIZE];
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &self.cells[r][c];
				match cell.contents
				{
					Contents::Terrain { terrain_type, .. } =>
					{
						let mut num_too_close: usize = 0;
						let mut num_close_similar: usize = 0;
						let mut num_close_different: usize = 0;
						for dr in -4..=4
						{
							for dc in -4..=4
							{
								if dr == 0 && dc == 0
								{
									continue;
								}
								let rr = r as i32 + dr;
								let cc = c as i32 + dc;
								if rr < 0
									|| cc < 0 || (rr as usize > GRID_SIZE - 1)
									|| (cc as usize > GRID_SIZE - 1)
								{
									continue;
								}
								let rr = rr as usize;
								let cc = cc as usize;
								let other = &self.cells[rr][cc];
								match cell.contents
								{
									Contents::Terrain { .. } => (),
									_ => continue,
								}
								let dx = (cell.centroid_x as i32)
									- (other.centroid_x as i32);
								let dy = (cell.centroid_y as i32)
									- (other.centroid_y as i32);
								if dx * dx + dy * dy
									> MAX_DISTANCE_BETWEEN_REGIONS
										* MAX_DISTANCE_BETWEEN_REGIONS
								{
									continue;
								}
								if dx * dx + dy * dy
									< MIN_DISTANCE_BETWEEN_REGIONS
										* MIN_DISTANCE_BETWEEN_REGIONS
								{
									num_too_close += 1;
								}
								if other.terrain_type() == Some(terrain_type)
								{
									num_close_similar += 1;
								}
								else
								{
									num_close_different += 1;
								}
							}
						}
						let goodness: i8 = 1
							+ (std::cmp::min(num_close_different, 9) as i8)
							- (std::cmp::min(num_close_similar, 9) as i8)
							- 10 * (std::cmp::min(num_too_close, 10) as i8);
						cell_goodness[r * GRID_SIZE + c] = goodness;
					}
					_ => (),
				}
			}
		}
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				let (te, tf) = match cell.terrain_type()
				{
					Some(TerrainType::Water) => (-100.0, 0.0),
					Some(TerrainType::Mountain) => (1000.0, 0.0),
					Some(TerrainType::Hill) =>
					{
						(ELEVATION_THRESHOLD_HILL + 10.0, -200.0)
					}
					Some(TerrainType::Forest) => (20.0, 400.0),
					Some(TerrainType::Grass) => (10.0, -200.0),
					None =>
					{
						// This is not clickable terrain so the center does
						// not really matter.
						continue;
					}
				};
				let x = cell.centroid_x as usize;
				let y = cell.centroid_y as usize;
				let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let f = forest.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let mut badness = (te - e).abs() + (tf - f).abs();
				if (te > 0.0 && e < 2.0) || (te < 0.0 && e > -2.0)
				{
					badness += 1000.0;
				}
				for _i in 0..100
				{
					if badness < 2.0
					{
						break;
					}
					let (x, y) = pick_random_centroid_xy_at_rc(r, c, rng);
					let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
					let f = forest.get_noise(x as f64 + 0.5, y as f64 + 0.5);
					let mut b = (te - e).abs() + (tf - f).abs();
					if (te > 0.0 && e < 2.0) || (te < 0.0 && e > -2.0)
					{
						b += 1000.0;
					}
					if b + 1.0 < badness
					{
						cell.centroid_x = x as u8;
						cell.centroid_y = y as u8;
						badness = b;
					}
				}
			}
		}
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &self.cells[r][c];
				let x = cell.centroid_x as usize;
				let y = cell.centroid_y as usize;
				match cell.terrain_type()
				{
					Some(TerrainType::Water) => (),
					Some(_) =>
					{
						draw_on_bitmap(&mut self.ink_bitmap, x, y);
						draw_on_bitmap(&mut self.ink_bitmap, x + 1, y);
						draw_on_bitmap(&mut self.ink_bitmap, x, y + 1);
						draw_on_bitmap(&mut self.ink_bitmap, x + 1, y + 1);
					}
					None =>
					{
						draw_on_bitmap(&mut self.ink_bitmap, x, y);
					}
				}
			}
		}
		for v in 0..PROP_GRID_SIZE
		{
			for u in 0..PROP_GRID_SIZE
			{
				let x = PROP_GRID_CELL_SIZE * u;
				let y = PROP_GRID_CELL_SIZE * v;
				let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let f = forest.get_noise(x as f64 + 0.5, y as f64 + 0.5);
				let terrain_type = if e > ELEVATION_THRESHOLD_MOUNTAIN
				{
					if (u + v) % 2 == 0
					{
						Some(TerrainType::Mountain)
					}
					else
					{
						None
					}
				}
				else if e > ELEVATION_THRESHOLD_HILL
				{
					if f > FOREST_THRESHOLD_ON_HILL
					{
						Some(TerrainType::Forest)
					}
					else if (u + v) % 2 == 0
					{
						Some(TerrainType::Hill)
					}
					else
					{
						None
					}
				}
				else if e > ELEVATION_THRESHOLD_WATER + 5.0
				{
					if f > FOREST_THRESHOLD_ON_GRASS
					{
						Some(TerrainType::Forest)
					}
					else
					{
						Some(TerrainType::Grass)
					}
				}
				else
				{
					None
				};
				set_on_propmap(&mut self.propmap, u, v, terrain_type);
			}
		}
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				match cell.contents
				{
					Contents::Terrain { .. } =>
					{
						let x = cell.centroid_x as usize;
						let y = cell.centroid_y as usize;
						let u = x / PROP_GRID_CELL_SIZE;
						let v = y / PROP_GRID_CELL_SIZE;
						for (dv, dus) in [(0, -1..=1), (1, -2..=2), (2, -1..=1)]
						{
							for du in dus
							{
								let uu = (u as i32 + du) as usize;
								let vv = (v as i32 + dv) as usize;
								if uu < PROP_GRID_SIZE && vv < PROP_GRID_SIZE
								{
									set_on_propmap(
										&mut self.propmap,
										uu,
										vv,
										None,
									);
								}
							}
						}
					}
					_ => (),
				}
			}
		}
		for r in 0..GRID_SIZE
		{
			for c in 0..4
			{
				let cell = &mut self.cells[r][c];
				match cell.terrain_type()
				{
					Some(_) =>
					{
						if (c <= 2) || rng.bool()
						{
							cell.become_occupied();
						}
					}
					None =>
					{
						if c <= 2
						{
							cell.become_occupied();
						}
					}
				}
			}
		}
		self.update_occupation();
	}

	pub fn update_occupation(&mut self)
	{
		// TODO only update a subsection if we know where the update took place
		for y in 0..MAP_SIZE
		{
			for x in 0..MAP_SIZE
			{
				let draw = if ((x + y) % 6) > 0
				{
					false
				}
				else
				{
					let (r, c, distance) =
						self.closest_occupied_rc_to_xy(x as i32, y as i32);
					if !self.cells[r][c].is_occupied()
					{
						false
					}
					else if is_on_bitmap(&self.water_bitmap, x, y)
						|| is_on_bitmap(&self.mountain_bitmap, x, y)
					{
						false
					}
					else if distance < 4.0
					{
						true
					}
					else if distance < 4.0 + 10.0
					{
						if let Some(gen) = &self.occupation_noise
						{
							let noise =
								gen.get_noise(x as f64 + 0.5, y as f64 + 0.5);
							10.0 * (distance - 4.0) + noise < 0.0
						}
						else
						{
							false
						}
					}
					else
					{
						false
					}
				};
				if draw
				{
					draw_on_bitmap(&mut self.occupation_bitmap, x, y);
					if x > 0 && y > 0
					{
						draw_on_bitmap(&mut self.occupation_bitmap, x - 1, y);
						draw_on_bitmap(&mut self.occupation_bitmap, x, y - 1);
						draw_on_bitmap(
							&mut self.occupation_bitmap,
							x - 1,
							y - 1,
						);
					}
				}
				else
				{
					erase_on_bitmap(&mut self.occupation_bitmap, x, y);
				}
			}
		}
	}

	pub fn draw(&self)
	{
		let map_x = 0;
		let map_y = 5;
		unsafe { *DRAW_COLORS = 0x00 };
		blit(
			&self.debug_bitmap,
			map_x,
			map_y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x20 };
		blit(
			&self.water_bitmap,
			map_x,
			map_y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x20 };
		blit(
			&self.occupation_bitmap,
			map_x,
			map_y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x30 };
		blit(
			&self.ink_bitmap,
			map_x,
			map_y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		if false
		{
			unsafe { *DRAW_COLORS = 3 };
			for r in 0..GRID_SIZE
			{
				for c in 0..GRID_SIZE
				{
					let x0 = map_x + self.cells[r][c].centroid_x as i32;
					let y0 = map_y + self.cells[r][c].centroid_y as i32;
					if r > 0
					{
						let xx = map_x + self.cells[r - 1][c].centroid_x as i32;
						let yy = map_y + self.cells[r - 1][c].centroid_y as i32;
						line(x0, y0, xx, yy);
					}
					if c > 0
					{
						let xx = map_x + self.cells[r][c - 1].centroid_x as i32;
						let yy = map_y + self.cells[r][c - 1].centroid_y as i32;
						line(x0, y0, xx, yy);
					}
				}
			}
		}

		unsafe { *DRAW_COLORS = 0x4320 };
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &self.cells[r][c];
				match cell.terrain_type()
				{
					Some(TerrainType::Water) =>
					{
						sprites::draw_boat(
							map_x + (cell.centroid_x as i32),
							map_y + (cell.centroid_y as i32),
							((2 * r + 3 * c) % 17) as u8,
						);
					}
					_ => (),
				}
			}
		}
		for v in 0..PROP_GRID_SIZE
		{
			for u in 0..PROP_GRID_SIZE
			{
				let x = map_x + (u * PROP_GRID_CELL_SIZE) as i32;
				let y =
					map_y + (v * PROP_GRID_CELL_SIZE) as i32 + (u % 2) as i32;
				let alt = ((2 * u + 3 * v) % 17) as u8;
				match get_from_propmap(&self.propmap, u, v)
				{
					Some(TerrainType::Mountain) =>
					{
						sprites::draw_mountain(x, y + 4, alt);
					}
					Some(TerrainType::Hill) =>
					{
						sprites::draw_hill(x, y, alt);
					}
					Some(TerrainType::Forest) =>
					{
						sprites::draw_tree(x - 1, y, alt);
					}
					Some(TerrainType::Grass) =>
					{
						//sprites::draw_grass(x, y, alt);
					}
					_ => (),
				}
			}
		}
	}

	fn r0_c0_for_xy(&self, x: i32, y: i32) -> (usize, usize)
	{
		let xx = std::cmp::max(0, x) as usize;
		let yy = std::cmp::max(0, y) as usize;
		let r = std::cmp::min(yy / GRID_CELL_SIZE, GRID_SIZE - 1);
		let c = std::cmp::min(xx / GRID_CELL_SIZE, GRID_SIZE - 1);
		let r0 = if r > 0
			&& (r + 1 == GRID_SIZE || y < (self.cells[r][c].centroid_y as i32))
		{
			r - 1
		}
		else
		{
			r
		};
		let c0 = if c > 0
			&& (c + 1 == GRID_SIZE || x < (self.cells[r][c].centroid_x as i32))
		{
			c - 1
		}
		else
		{
			c
		};
		(r0, c0)
	}

	fn closest_rc_to_xy(&self, x: i32, y: i32) -> (usize, usize, f64)
	{
		let (r0, c0) = self.r0_c0_for_xy(x, y);
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
			.unwrap_or((0, 0, 40404.0))
	}

	fn closest_occupied_rc_to_xy(&self, x: i32, y: i32) -> (usize, usize, f64)
	{
		let (r0, c0) = self.r0_c0_for_xy(x, y);
		if self.cells[r0][c0].is_occupied()
			&& self.cells[r0][c0 + 1].is_occupied()
			&& self.cells[r0 + 1][c0].is_occupied()
			&& self.cells[r0 + 1][c0 + 1].is_occupied()
		{
			return (r0, c0, 0.0);
		}
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
			.filter(|(r, c, sqdis)| {
				self.cells[*r][*c].is_occupied() || *sqdis < 4 * 4
			})
			.min_by_key(|(_r, _c, sqdis)| *sqdis)
			.map(|(r, c, sqdis)| {
				if self.cells[r][c].is_occupied()
				{
					(r, c, (sqdis as f64).sqrt())
				}
				else
				{
					(r, c, 1000.0)
				}
			})
			.unwrap_or((r0, c0, 1000.0))
	}

	fn soft_merge_cell(&mut self, r: usize, c: usize, rng: &mut fastrand::Rng)
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
					self.cells[r][c].contents = Contents::Merged {
						parent_row: rr as u8,
						parent_col: cc as u8,
						is_occupied: false,
					};
				}
				else if rng.bool()
				{
					self.cells[rr][cc].contents = Contents::Merged {
						parent_row: r as u8,
						parent_col: c as u8,
						is_occupied: false,
					};
					self.cells[r][c].make_more_important();
				}
				else
				{
					self.cells[r][c].contents = Contents::Merged {
						parent_row: rr as u8,
						parent_col: cc as u8,
						is_occupied: false,
					};
					self.cells[rr][cc].make_more_important();
				}
				break;
			}
		}
	}

	fn force_merge_edge(&mut self, r: usize, c: usize, rng: &mut fastrand::Rng)
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
			if (rr == 0 || rr == GRID_SIZE - 1)
				&& (cc == 0 || cc == GRID_SIZE - 1)
			{
				continue;
			}
			if self.cells[rr][cc].terrain_type() == tt
			{
				self.cells[r][c].contents = Contents::Merged {
					parent_row: rr as u8,
					parent_col: cc as u8,
					is_occupied: false,
				};
				self.cells[rr][cc].make_more_important();
				break;
			}
		}
	}
}

#[derive(Debug, Clone, Copy)]
enum Contents
{
	Empty,
	Tally
	{
		n_water: u8,
		n_mountain: u8,
		n_hill: u8,
		n_forest: u8,
		n_grass: u8,
	},
	Terrain
	{
		terrain_type: TerrainType,
		is_clickable: bool,
		is_occupied: bool,
		merge_importance: u8,
	},
	Merged
	{
		parent_row: u8,
		parent_col: u8,
		is_occupied: bool,
	},
	Culled
	{
		is_occupied: bool,
	},
}

#[derive(Debug, Clone, Copy)]
struct Cell
{
	centroid_x: u8,
	centroid_y: u8,
	contents: Contents,
}

const EMPTY_CELL: Cell = Cell {
	centroid_x: 0,
	centroid_y: 0,
	contents: Contents::Empty,
};

impl Cell
{
	fn terrain_type(&self) -> Option<TerrainType>
	{
		match self.contents
		{
			Contents::Terrain { terrain_type, .. } => Some(terrain_type),
			_ => None,
		}
	}

	fn is_occupied(&self) -> bool
	{
		match self.contents
		{
			Contents::Terrain { is_occupied, .. } => is_occupied,
			Contents::Merged { is_occupied, .. } => is_occupied,
			Contents::Culled { is_occupied, .. } => is_occupied,
			_ => false,
		}
	}

	fn become_occupied(&mut self)
	{
		match &mut self.contents
		{
			Contents::Terrain { is_occupied, .. } => *is_occupied = true,
			Contents::Merged { is_occupied, .. } => *is_occupied = true,
			Contents::Culled { is_occupied, .. } => *is_occupied = true,
			_ => (),
		}
	}

	fn make_more_important(&mut self)
	{
		match &mut self.contents
		{
			Contents::Terrain {
				merge_importance, ..
			} =>
			{
				*merge_importance += 1;
			}
			_ => (),
		}
	}

	fn is_crucial(&self) -> bool
	{
		match self.contents
		{
			Contents::Terrain {
				merge_importance, ..
			} => merge_importance >= 3,
			_ => false,
		}
	}

	fn add_tally(&mut self, terrain_type: TerrainType)
	{
		match &mut self.contents
		{
			Contents::Tally {
				n_water,
				n_mountain,
				n_hill,
				n_forest,
				n_grass,
			} => match terrain_type
			{
				TerrainType::Grass =>
				{
					if *n_grass < 250
					{
						*n_grass += 1;
					}
				}
				TerrainType::Forest =>
				{
					if *n_forest < 250
					{
						*n_forest += 1;
					}
				}
				TerrainType::Hill =>
				{
					if *n_hill < 250
					{
						*n_hill += 1;
					}
				}
				TerrainType::Mountain =>
				{
					if *n_mountain < 250
					{
						*n_mountain += 1;
					}
				}
				TerrainType::Water =>
				{
					if *n_water < 250
					{
						*n_water += 1;
					}
				}
			},
			_ => (),
		}
	}

	fn realize_terrain(&mut self)
	{
		let terrain_type = match self.contents
		{
			Contents::Tally {
				n_water,
				n_mountain,
				n_hill,
				n_forest,
				n_grass,
			} =>
			{
				let n_total = (n_water as u16)
					+ (n_mountain as u16)
					+ (n_hill as u16) + (n_forest as u16)
					+ (n_grass as u16);
				let n_total = std::cmp::min(n_total, 250) as u8;
				if n_water > (3 * (n_total as u16) / 4) as u8
				{
					Some(TerrainType::Water)
				}
				else if n_mountain > n_total / 2
				{
					Some(TerrainType::Mountain)
				}
				else if n_hill > n_total / 2
				{
					Some(TerrainType::Hill)
				}
				else if n_forest > n_total / 2
				{
					Some(TerrainType::Forest)
				}
				else if n_grass > n_total / 2
				{
					Some(TerrainType::Grass)
				}
				else
				{
					None
				}
			}
			_ => None,
		};

		if let Some(terrain_type) = terrain_type
		{
			self.contents = Contents::Terrain {
				terrain_type,
				is_clickable: false,
				is_occupied: false,
				merge_importance: 0,
			}
		}
		else
		{
			self.contents = Contents::Culled { is_occupied: false };
		}
	}
}

fn draw_on_bitmap(bitmap: &mut [u8; BITMAP_SIZE], x: usize, y: usize)
{
	let offset = y * MAP_SIZE + x;
	let byte_offset = offset / 8;
	let bit_shift = offset % 8;
	bitmap[byte_offset] |= 0b10000000 >> bit_shift;
}

fn erase_on_bitmap(bitmap: &mut [u8; BITMAP_SIZE], x: usize, y: usize)
{
	let offset = y * MAP_SIZE + x;
	let byte_offset = offset / 8;
	let bit_shift = offset % 8;
	bitmap[byte_offset] &= !(0b10000000 >> bit_shift);
}

fn is_on_bitmap(bitmap: &[u8; BITMAP_SIZE], x: usize, y: usize) -> bool
{
	let offset = y * MAP_SIZE + x;
	let byte_offset = offset / 8;
	let bit_shift = offset % 8;
	bitmap[byte_offset] & (0b10000000 >> bit_shift) > 0
}

fn set_on_propmap(
	propmap: &mut [u8; PROPMAP_SIZE],
	u: usize,
	v: usize,
	terrain_type: Option<TerrainType>,
)
{
	let value = match terrain_type
	{
		None => 0,
		Some(TerrainType::Grass) => 1,
		Some(TerrainType::Forest) => 5,
		Some(TerrainType::Hill) => 10,
		Some(TerrainType::Mountain) => 12,
		Some(TerrainType::Water) => 0,
	};
	let offset = v * PROP_GRID_SIZE + u;
	let byte_offset = offset / 2;
	assert!(byte_offset < PROPMAP_SIZE);
	let bit_shift = 4 * (offset % 2);
	propmap[byte_offset] &= !(0b11110000 >> bit_shift);
	let value = value & 0b1111;
	propmap[byte_offset] |= (value << 4) >> bit_shift;
}

fn get_from_propmap(
	propmap: &[u8; PROPMAP_SIZE],
	u: usize,
	v: usize,
) -> Option<TerrainType>
{
	let offset = v * PROP_GRID_SIZE + u;
	let byte_offset = offset / 2;
	assert!(byte_offset < PROPMAP_SIZE);
	let bit_shift = 4 - 4 * (offset % 2);
	let value = (propmap[byte_offset] >> bit_shift) & 0b1111;
	match value
	{
		0 => None,
		1..=4 => Some(TerrainType::Grass),
		5..=9 => Some(TerrainType::Forest),
		10..=11 => Some(TerrainType::Hill),
		12..=15 => Some(TerrainType::Mountain),
		16.. => None,
	}
}

fn pick_random_centroid_xy_at_rc(
	r: usize,
	c: usize,
	rng: &mut fastrand::Rng,
) -> (usize, usize)
{
	let padding = 2;
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
