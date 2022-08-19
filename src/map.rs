//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

use crate::wasm4::*;

use crate::level::Marker;
use crate::level::TerrainType;
use crate::level::MAX_NUM_REGIONS;
use crate::sprites;

use bitmaps::Bitmap;
use fastrand;
use perlin2d::PerlinNoise2D;

pub const MAP_SIZE: usize = 160;
pub const BITMAP_SIZE: usize = MAP_SIZE * MAP_SIZE / 8;
pub const GRID_SIZE: usize = 16;
pub const GRID_CELL_SIZE: usize = MAP_SIZE / GRID_SIZE;
pub const PROP_GRID_SIZE: usize = 40;
pub const PROP_GRID_CELL_SIZE: usize = MAP_SIZE / PROP_GRID_SIZE;
pub const PROPMAP_SIZE: usize = PROP_GRID_SIZE * PROP_GRID_SIZE / 2;

const MIN_NUM_REGIONS: usize = 25;
const MIN_DISTANCE_BETWEEN_REGIONS: i32 = 20;
const MAX_DISTANCE_BETWEEN_REGIONS: i32 = 30;
const DBR_BBOX_RADIUS: i32 = 4;
const VILLAGE_RADIUS: i32 = 15;

const NOISE_OCTAVES: i32 = 5;
const NOISE_AMPLITUDE_ELEVATION: f64 = 50.0;
const NOISE_AMPLITUDE_ELEVATION_MAGIC: f64 = 25.0;
const NOISE_AMPLITUDE_FOREST: f64 = 50.0;
const NOISE_AMPLITUDE_OCCUPATION: f64 = 0.25;
const NOISE_FREQUENCY_ELEVATION: f64 = 1.0;
const NOISE_FREQUENCY_FOREST: f64 = 1.0;
const NOISE_FREQUENCY_OCCUPATION: f64 = 2.0;
const NOISE_PERSISTENCE_ELEVATION: f64 = 1.0;
const NOISE_PERSISTENCE_FOREST: f64 = 2.0;
const NOISE_PERSISTENCE_OCCUPATION: f64 = 2.0;
const NOISE_LACUNARITY: f64 = 2.0;
const NOISE_SCALE: (f64, f64) = (MAP_SIZE as f64, MAP_SIZE as f64);
const NOISE_BIAS_ELEVATION: f64 = 35.0;
const NOISE_BIAS_FOREST: f64 = 0.0;
const NOISE_BIAS_OCCUPATION: f64 = 0.0;

const ELEVATION_THRESHOLD_MOUNTAIN: f64 = 70.0;
const ELEVATION_THRESHOLD_HILL: f64 = 50.0;
const ELEVATION_THRESHOLD_WATER: f64 = 0.0;
const FOREST_THRESHOLD_ON_HILL: f64 = 10.0;
const FOREST_THRESHOLD_ON_GRASS: f64 = 200.0;

const MAP_X: i32 = 2;
const MAP_Y: i32 = 7;

pub struct Map
{
	water_bitmap: [u8; BITMAP_SIZE],
	mountain_bitmap: [u8; BITMAP_SIZE],
	surface_bitmap: [u8; BITMAP_SIZE],
	ink_bitmap: [u8; BITMAP_SIZE],
	occupation_bitmap: [u8; BITMAP_SIZE],
	propmap: [u8; PROPMAP_SIZE],
	surface_propmap: [u8; PROPMAP_SIZE],
	prop_region_map: [[i8; PROP_GRID_SIZE]; PROP_GRID_SIZE],
	cells: [[Cell; GRID_SIZE]; GRID_SIZE],
	occupation_noise: Option<PerlinNoise2D>,
}

impl Map
{
	pub const fn empty() -> Self
	{
		Self {
			water_bitmap: [0; BITMAP_SIZE],
			mountain_bitmap: [0; BITMAP_SIZE],
			surface_bitmap: [0; BITMAP_SIZE],
			ink_bitmap: [0; BITMAP_SIZE],
			occupation_bitmap: [0; BITMAP_SIZE],
			propmap: [0; PROPMAP_SIZE],
			surface_propmap: [0; PROPMAP_SIZE],
			prop_region_map: [[0; PROP_GRID_SIZE]; PROP_GRID_SIZE],
			cells: [[EMPTY_CELL; GRID_SIZE]; GRID_SIZE],
			occupation_noise: None,
		}
	}

	pub fn generate(&mut self, rng: &mut fastrand::Rng)
	{
		let seed = rng.u16(..) as i32;
		let elevation = generate_elevation_noise(seed);
		let seed = rng.u16(..) as i32;
		let forest = PerlinNoise2D::new(
			NOISE_OCTAVES,
			NOISE_AMPLITUDE_FOREST,
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
			NOISE_AMPLITUDE_OCCUPATION,
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
				let has_surface = match terrain_type
				{
					TerrainType::Forest =>
					{
						((x + y) % 2 == 0)
							&& (x % 4 == 0) && ((x / 2 + y) % 4 == 0)
					}
					TerrainType::Hill =>
					{
						((x / 2 + y) % 2 == 0)
							&& ((x / 2 + y) % 4 == 0) && (y % 2 == 0)
					}
					TerrainType::Mountain => ((x + y) % 2 == 0),
					_ => false,
				};
				if has_surface
				{
					draw_on_bitmap(&mut self.surface_bitmap, x, y);
				}
				else
				{
					erase_on_bitmap(&mut self.surface_bitmap, x, y);
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
		// Merge cells to determine candidates for region clearings.
		for i in 0..GRID_SIZE
		{
			self.merge_edge_cell(i, 0, rng);
			self.merge_edge_cell(i, GRID_SIZE - 1, rng);
			self.merge_edge_cell(0, i, rng);
			self.merge_edge_cell(GRID_SIZE - 1, i, rng);
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
		// Remove the cells with the highest badness until we have between
		// 25 and 35 regions.
		let mut cell_badness = [0i8; GRID_SIZE * GRID_SIZE];
		let mut num_candidates = 0;
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				match self.cells[r][c].contents
				{
					Contents::Unmerged { .. } =>
					{
						let badness = self.calculate_cell_badness(r, c);
						cell_badness[r * GRID_SIZE + c] = badness;
						num_candidates += 1;
					}
					_ => (),
				}
			}
		}
		while num_candidates > MIN_NUM_REGIONS
		{
			let worst = cell_badness
				.iter()
				.enumerate()
				.map(|(i, badness)| (i, *badness))
				.max_by_key(|(_i, badness)| *badness);
			if let Some((offset, badness)) = worst
			{
				if badness < 16 && num_candidates <= MAX_NUM_REGIONS
				{
					break;
				}

				let r = offset / GRID_SIZE;
				let c = offset % GRID_SIZE;
				self.force_merge_cell(r, c);
				cell_badness[offset] = 0;
				num_candidates -= 1;

				for dr in -DBR_BBOX_RADIUS..=DBR_BBOX_RADIUS
				{
					for dc in -DBR_BBOX_RADIUS..=DBR_BBOX_RADIUS
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
						match other.contents
						{
							Contents::Unmerged { .. } => (),
							_ => continue,
						}
						let b = self.calculate_cell_badness(rr, cc);
						cell_badness[rr * GRID_SIZE + cc] = b;
					}
				}
			}
			else
			{
				break;
			}
		}
		// We have found our regions.
		let mut num_regions = 0;
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				match cell.contents
				{
					Contents::Unmerged {
						terrain_type,
						merge_importance: _,
					} =>
					{
						cell.contents = Contents::Region {
							terrain_type,
							region_id: num_regions,
							marker: None,
							occupation_percentage: 0,
						};
						num_regions += 1;
					}
					_ => (),
				}
			}
		}
		// Match merged cells with their regions.
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				match self.cells[r][c].contents
				{
					Contents::Merged {
						parent_row,
						parent_col,
						parent_terrain_type,
					} =>
					{
						let region_id = self.find_parent_region_id(
							parent_row as usize,
							parent_col as usize,
						);

						let cell = &mut self.cells[r][c];
						if let Some(parent_region_id) = region_id
						{
							cell.contents = Contents::Subregion {
								parent_region_id,
								parent_terrain_type,
								occupation_percentage: 0,
							};
						}
						else
						{
							cell.contents = Contents::Culled {
								occupation_percentage: 0,
							};
						}
					}
					_ => (),
				}
			}
		}
		// Adjust region clearing positions.
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				let terrain_type = match cell.contents
				{
					Contents::Region { terrain_type, .. } => terrain_type,
					_ => continue,
				};
				let (te, tf) = match terrain_type
				{
					TerrainType::Water => (-100.0, 0.0),
					TerrainType::Mountain => (1000.0, 0.0),
					TerrainType::Hill =>
					{
						(ELEVATION_THRESHOLD_HILL + 10.0, -200.0)
					}
					TerrainType::Forest => (20.0, 400.0),
					TerrainType::Grass => (10.0, -200.0),
					TerrainType::Village => (10.0, -200.0),
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
		// Draw region clearings.
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &self.cells[r][c];
				let x = cell.centroid_x as usize;
				let y = cell.centroid_y as usize;
				match cell.contents
				{
					Contents::Region {
						terrain_type: TerrainType::Water,
						..
					} => (),
					Contents::Region {
						terrain_type: _, ..
					} =>
					{
						draw_on_bitmap(&mut self.ink_bitmap, x, y);
						draw_on_bitmap(&mut self.ink_bitmap, x + 1, y);
						draw_on_bitmap(&mut self.ink_bitmap, x, y + 1);
						draw_on_bitmap(&mut self.ink_bitmap, x + 1, y + 1);
						draw_on_bitmap(&mut self.ink_bitmap, x - 1, y);
						draw_on_bitmap(&mut self.ink_bitmap, x, y + 2);
						draw_on_bitmap(&mut self.ink_bitmap, x - 1, y + 2);
						draw_on_bitmap(&mut self.ink_bitmap, x + 1, y + 2);
						draw_on_bitmap(&mut self.ink_bitmap, x - 1, y + 1);
					}
					Contents::Subregion { .. } if false =>
					{
						draw_on_bitmap(&mut self.ink_bitmap, x, y);
					}
					Contents::Culled { .. } if false =>
					{
						draw_on_bitmap(&mut self.ink_bitmap, x, y);
					}
					_ => (),
				}
			}
		}
		// Determine props.
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
					Some(TerrainType::Mountain)
				}
				else if e > ELEVATION_THRESHOLD_HILL
				{
					if f > FOREST_THRESHOLD_ON_HILL
					{
						Some(TerrainType::Forest)
					}
					else
					{
						Some(TerrainType::Hill)
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
				let prop_type = match terrain_type
				{
					Some(TerrainType::Mountain) | Some(TerrainType::Hill) =>
					{
						if (u + v) % 2 == 0
						{
							terrain_type
						}
						else
						{
							None
						}
					}
					_ => terrain_type,
				};
				set_on_propmap(&mut self.propmap, u, v, prop_type);
				set_on_propmap(&mut self.surface_propmap, u, v, terrain_type);
				self.prop_region_map[v][u] = -1;
				if terrain_type.is_none()
				{
					continue;
				}
				let (r, c, _distance) =
					self.closest_rc_to_xy(x as i32, y as i32);

				match self.cells[r][c].contents
				{
					Contents::Region {
						region_id,
						terrain_type: tt,
						..
					}
					| Contents::Subregion {
						parent_region_id: region_id,
						parent_terrain_type: tt,
						..
					} =>
					{
						if Some(tt) == terrain_type
						{
							self.prop_region_map[v][u] = region_id;
						}
					}
					_ => (),
				}
			}
		}
		// Remove props around clearings.
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &mut self.cells[r][c];
				match cell.contents
				{
					Contents::Region { .. } => (),
					_ => continue,
				}
				let x = cell.centroid_x as usize;
				let y = cell.centroid_y as usize;
				let u = x / PROP_GRID_CELL_SIZE;
				let v = y / PROP_GRID_CELL_SIZE;
				for (dv, dus) in [(0, -1..=1), (1, -1..=2), (2, -1..=2)]
				{
					for du in dus
					{
						let uu = (u as i32 + du) as usize;
						let vv = (v as i32 + dv) as usize;
						if uu < PROP_GRID_SIZE && vv < PROP_GRID_SIZE
						{
							set_on_propmap(&mut self.propmap, uu, vv, None);
						}
					}
				}
			}
		}
		// Merge abandoned props on the edges of regions.
		for _i in 0..5
		{
			let mut any = false;
			for v in 0..PROP_GRID_SIZE
			{
				for u in 0..PROP_GRID_SIZE
				{
					if self.prop_region_map[v][u] >= 0
					{
						continue;
					}
					let terrain_type =
						get_from_propmap(&self.surface_propmap, u, v);
					if terrain_type.is_none()
					{
						continue;
					}
					let mut adjacents = [(1, 0), (-1, 0), (0, 1), (0, -1)];
					rng.shuffle(&mut adjacents);
					for (dv, du) in adjacents
					{
						if (v == 0 && dv < 0)
							|| (u == 0 && du < 0) || (v + 1 == PROP_GRID_SIZE
							&& dv > 0) || (u + 1 == PROP_GRID_SIZE && du > 0)
						{
							continue;
						};
						let uu = ((u as i32) + du) as usize;
						let vv = ((v as i32) + dv) as usize;
						if self.prop_region_map[vv][uu] < 0
						{
							continue;
						}
						if get_from_propmap(&self.surface_propmap, uu, vv)
							== terrain_type
						{
							self.prop_region_map[v][u] =
								self.prop_region_map[vv][uu];
							any = true;
						}
					}
				}
			}
			if !any
			{
				break;
			}
		}
	}

	pub fn update_occupation_map(&mut self, percentage: u8)
	{
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
					else
					{
						let p = self.cells[r][c]
							.update_occupation_percentage(percentage);
						let min_distance = if p >= 100
						{
							4.0
						}
						else
						{
							4.0 * 0.01 * (p as f64)
						};
						if distance < min_distance
						{
							true
						}
						else if distance < min_distance + 20.0
						{
							if let Some(gen) = &self.occupation_noise
							{
								let noise = gen
									.get_noise(x as f64 + 0.5, y as f64 + 0.5)
									.clamp(-2.0, 2.0);
								distance < min_distance + noise
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

	pub fn regions(&self) -> impl Iterator<Item = (i8, TerrainType)> + '_
	{
		(0..(GRID_SIZE * GRID_SIZE))
			.map(|i| (i / GRID_SIZE, i % GRID_SIZE))
			.filter_map(|(r, c)| match self.cells[r][c].contents
			{
				Contents::Region {
					region_id,
					terrain_type,
					marker: _,
					occupation_percentage: _,
				} => Some((region_id, terrain_type)),
				Contents::Subregion { .. } => None,
				Contents::Culled { .. } => None,
				_ => None,
			})
	}

	pub fn fill_adjacency(
		&self,
		adjacency: &mut [Bitmap<MAX_NUM_REGIONS>; MAX_NUM_REGIONS],
		border_adjacency: &mut Bitmap<MAX_NUM_REGIONS>,
	)
	{
		for v in 0..PROP_GRID_SIZE
		{
			for u in 0..PROP_GRID_SIZE
			{
				let i = self.prop_region_map[v][u];
				if i < 0
				{
					continue;
				}
				if u > 0
				{
					let j = self.prop_region_map[v][u - 1];
					if i != j && j >= 0
					{
						adjacency[i as usize].set(j as usize, true);
						adjacency[j as usize].set(i as usize, true);
					}
				}
				if v > 0
				{
					let j = self.prop_region_map[v - 1][u];
					if i != j && j >= 0
					{
						adjacency[i as usize].set(j as usize, true);
						adjacency[j as usize].set(i as usize, true);
					}
				}
				if u == 0
					|| v == 0 || u == PROP_GRID_SIZE - 1
					|| v == PROP_GRID_SIZE - 1
				{
					border_adjacency.set(i as usize, true);
				}
			}
		}
	}

	pub fn set_marker_in_region(
		&mut self,
		region_id: i8,
		marker: Option<Marker>,
	)
	{
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				match &mut self.cells[r][c].contents
				{
					Contents::Region {
						region_id: r,
						marker: m,
						..
					} if *r == region_id =>
					{
						*m = marker;
						return;
					}
					_ => (),
				}
			}
		}
	}

	pub fn occupy_region(&mut self, region_id: i8)
	{
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				match &mut self.cells[r][c].contents
				{
					Contents::Region {
						region_id: r,
						occupation_percentage,
						..
					} if *r == region_id =>
					{
						*occupation_percentage = 1;
					}
					Contents::Subregion {
						parent_region_id: r,
						occupation_percentage,
						..
					} if *r == region_id =>
					{
						*occupation_percentage = 1;
					}
					_ => (),
				}
			}
		}
	}

	pub fn place_village(&mut self, region_id: i8)
	{
		let mut centerx = 0;
		let mut centery = 0;
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				match &mut self.cells[r][c].contents
				{
					Contents::Region {
						region_id: rid,
						terrain_type: tt,
						..
					} if *rid == region_id =>
					{
						*tt = TerrainType::Village;
						centerx = self.cells[r][c].centroid_x as i32;
						centery = self.cells[r][c].centroid_y as i32;
					}
					Contents::Subregion {
						parent_region_id: rid,
						parent_terrain_type: tt,
						..
					} if *rid == region_id =>
					{
						*tt = TerrainType::Village;
					}
					_ => (),
				}
			}
		}
		for v in 0..PROP_GRID_SIZE
		{
			for u in 0..PROP_GRID_SIZE
			{
				if self.prop_region_map[v][u] != region_id
				{
					continue;
				}
				if get_from_propmap(&self.propmap, u, v).is_none()
				{
					continue;
				}
				let x = (u * PROP_GRID_CELL_SIZE) as i32;
				let y = (v * PROP_GRID_CELL_SIZE) as i32;
				let dx = x - centerx;
				let dy = y - centery;
				if dx * dx + dy * dy > VILLAGE_RADIUS * VILLAGE_RADIUS
				{
					continue;
				}
				let prop = if (u + v) % 2 == 0
				{
					Some(TerrainType::Village)
				}
				else
				{
					None
				};
				set_on_propmap(&mut self.propmap, u, v, prop);
			}
		}
	}

	pub fn draw(
		&self,
		hovered_region_id: Option<i8>,
		highlighted_terrain_type: Option<TerrainType>,
		kill_preview: Bitmap<MAX_NUM_REGIONS>,
		attack_preview: Bitmap<MAX_NUM_REGIONS>,
		support_preview: Bitmap<MAX_NUM_REGIONS>,
		gather_preview: Bitmap<MAX_NUM_REGIONS>,
	)
	{
		let mut is_empty = [false; MAX_NUM_REGIONS];
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				match self.cells[r][c].contents
				{
					Contents::Region {
						region_id,
						marker,
						occupation_percentage,
						terrain_type: _,
					} =>
					{
						is_empty[region_id as usize] =
							marker.is_none() && occupation_percentage == 0;
					}
					_ => (),
				}
			}
		}

		unsafe { *DRAW_COLORS = 0x20 };
		blit(
			&self.surface_bitmap,
			MAP_X,
			MAP_Y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);

		unsafe { *DRAW_COLORS = 0x40 };
		if hovered_region_id.is_some() || highlighted_terrain_type.is_some()
		{
			for v in 0..PROP_GRID_SIZE
			{
				for u in 0..PROP_GRID_SIZE
				{
					let region_id = self.prop_region_map[v][u];
					if region_id < 0
						|| !is_empty[region_id as usize]
						|| (highlighted_terrain_type.is_none()
							&& hovered_region_id != Some(region_id))
					{
						continue;
					}
					let x = MAP_X + (u * PROP_GRID_CELL_SIZE) as i32;
					let y = MAP_Y + (v * PROP_GRID_CELL_SIZE) as i32;
					let alt = ((23 * u + 71 * v + 59 * (u + v)) % 97) as u8;
					let tt = get_from_propmap(&self.surface_propmap, u, v);
					if hovered_region_id == Some(region_id)
						|| tt == highlighted_terrain_type
					{
						match tt
						{
							Some(TerrainType::Mountain)
							| Some(TerrainType::Hill)
							| Some(TerrainType::Grass) =>
							{
								sprites::draw_surface(x, y, alt);
							}
							_ => (),
						}
					}
				}
			}
		}

		unsafe { *DRAW_COLORS = 0x20 };
		blit(
			&self.water_bitmap,
			MAP_X,
			MAP_Y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x10 };
		blit(
			&self.occupation_bitmap,
			MAP_X,
			MAP_Y,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x20 };
		blit(
			&self.occupation_bitmap,
			MAP_X,
			MAP_Y + 3,
			MAP_SIZE as u32,
			MAP_SIZE as u32,
			BLIT_1BPP,
		);
		unsafe { *DRAW_COLORS = 0x30 };
		blit(
			&self.ink_bitmap,
			MAP_X,
			MAP_Y,
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
					let x0 = MAP_X + self.cells[r][c].centroid_x as i32;
					let y0 = MAP_Y + self.cells[r][c].centroid_y as i32;
					if r > 0
					{
						let xx = MAP_X + self.cells[r - 1][c].centroid_x as i32;
						let yy = MAP_Y + self.cells[r - 1][c].centroid_y as i32;
						line(x0, y0, xx, yy);
					}
					if c > 0
					{
						let xx = MAP_X + self.cells[r][c - 1].centroid_x as i32;
						let yy = MAP_Y + self.cells[r][c - 1].centroid_y as i32;
						line(x0, y0, xx, yy);
					}
				}
			}
		}

		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &self.cells[r][c];
				match cell.contents
				{
					Contents::Region {
						terrain_type: TerrainType::Water,
						..
					} =>
					{
						unsafe { *DRAW_COLORS = 0x1320 };
						sprites::draw_boat(
							MAP_X + (cell.centroid_x as i32),
							MAP_Y + (cell.centroid_y as i32),
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
				let region_id = self.prop_region_map[v][u];
				let terrain_type = get_from_propmap(&self.propmap, u, v);
				if region_id >= 0
					&& is_empty[region_id as usize]
					&& (hovered_region_id == Some(region_id)
						|| highlighted_terrain_type == terrain_type)
				{
					unsafe { *DRAW_COLORS = 0x4320 };
				}
				else
				{
					unsafe { *DRAW_COLORS = 0x1320 };
				}
				let x = MAP_X + (u * PROP_GRID_CELL_SIZE) as i32;
				let y =
					MAP_Y + (v * PROP_GRID_CELL_SIZE) as i32 + (u % 2) as i32;
				let alt = ((23 * u + 71 * v + 59 * (u + v)) % 97) as u8;
				match terrain_type
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
					Some(TerrainType::Village) =>
					{
						sprites::draw_house(x - 1, y, alt);
					}
					Some(TerrainType::Grass) =>
					{
						let is_palette_bloody =
							unsafe { *PALETTE == crate::palette::BLOOD }
								|| unsafe { *PALETTE == crate::palette::ROMAN };
						let is_placing_blood = hovered_region_id
							== Some(region_id)
							&& is_palette_bloody;
						if alt < 25 && (u + v) % 2 > 0 && !is_placing_blood
						{
							sprites::draw_grass(x, y, alt);
						}
					}
					_ => (),
				}
			}
		}
		for r in 0..GRID_SIZE
		{
			for c in 0..GRID_SIZE
			{
				let cell = &self.cells[r][c];
				match cell.contents
				{
					Contents::Region {
						region_id,
						marker: Some(marker),
						..
					} =>
					{
						let is_killed = kill_preview.get(region_id as usize);
						let flag = match marker
						{
							Marker::Roman if is_killed => 1,
							Marker::Roman => 0,
							Marker::DeadRoman => 1,
							Marker::Worker if is_killed => 3,
							Marker::Worker => 2,
							Marker::DeadWorker => 3,
							Marker::Occupied => 0,
						};
						let x = MAP_X + cell.centroid_x as i32;
						let y = MAP_Y + cell.centroid_y as i32;
						unsafe { *DRAW_COLORS = 0x3210 };
						sprites::draw_flag(x, y, flag);
						unsafe { *DRAW_COLORS = 0x2310 };
						if is_killed
						{
							sprites::draw_killed_town(x, y);
						}
						else if attack_preview.get(region_id as usize)
						{
							sprites::draw_attacking_town(x, y);
						}
						else if support_preview.get(region_id as usize)
						{
							sprites::draw_supporting_town(x, y);
						}
						else if gather_preview.get(region_id as usize)
						{
							sprites::draw_gathering_town(x, y);
						}
					}
					Contents::Region {
						region_id,
						marker: None,
						..
					} if hovered_region_id == Some(region_id) =>
					{
						let x = MAP_X + cell.centroid_x as i32;
						let y = MAP_Y + cell.centroid_y as i32;
						unsafe { *DRAW_COLORS = 0x4310 };
						if kill_preview.get(region_id as usize)
						{
							unsafe { *DRAW_COLORS = 0x2310 };
							sprites::draw_killed_town(x, y);
						}
						else if attack_preview.get(region_id as usize)
						{
							sprites::draw_hovered_town(x, y);
							unsafe { *DRAW_COLORS = 0x2310 };
							sprites::draw_attacking_town(x + 6, y);
						}
						else
						{
							sprites::draw_hovered_town(x, y);
						}
					}
					_ => (),
				}
			}
		}
	}

	pub fn determine_hovered_region_id(&self) -> Option<i8>
	{
		let (mouse_x, mouse_y): (i16, i16) = unsafe { (*MOUSE_X, *MOUSE_Y) };
		let x = (mouse_x as i32) - MAP_X;
		let y = (mouse_y as i32) - MAP_Y;
		let (r, c, distance) = self.closest_rc_to_xy(x, y);
		if r == 0 || distance > 1000.0
		{
			return None;
		}
		match self.cells[r][c].contents
		{
			Contents::Region { region_id, .. } => Some(region_id),
			Contents::Subregion {
				parent_region_id, ..
			} => Some(parent_region_id),
			_ => None,
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
		let is_enclosed = self.cells[r0][c0].is_occupied()
			&& self.cells[r0][c0 + 1].is_occupied()
			&& self.cells[r0 + 1][c0].is_occupied()
			&& self.cells[r0 + 1][c0 + 1].is_occupied();
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
				if is_enclosed
				{
					(r, c, (sqdis as f64).sqrt().min(3.99))
				}
				else if self.cells[r][c].is_occupied()
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
		let terrain_type = match self.cells[r][c].contents
		{
			Contents::Unmerged { terrain_type, .. } => terrain_type,
			_ => return,
		};
		if self.cells[r][c].is_crucial()
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
			match self.cells[rr][cc].contents
			{
				Contents::Unmerged {
					terrain_type: tt, ..
				} =>
				{
					if tt != terrain_type
					{
						continue;
					}
				}
				_ => continue,
			}
			// Merge either cell into the other.
			if self.cells[rr][cc].is_crucial()
			{
				self.cells[r][c].contents = Contents::Merged {
					parent_row: rr as u8,
					parent_col: cc as u8,
					parent_terrain_type: terrain_type,
				};
			}
			else if rng.bool()
			{
				self.cells[rr][cc].contents = Contents::Merged {
					parent_row: r as u8,
					parent_col: c as u8,
					parent_terrain_type: terrain_type,
				};
				self.cells[r][c].make_more_important();
			}
			else
			{
				self.cells[r][c].contents = Contents::Merged {
					parent_row: rr as u8,
					parent_col: cc as u8,
					parent_terrain_type: terrain_type,
				};
				self.cells[rr][cc].make_more_important();
			}
			break;
		}
	}

	fn merge_edge_cell(&mut self, r: usize, c: usize, rng: &mut fastrand::Rng)
	{
		let terrain_type = match self.cells[r][c].contents
		{
			Contents::Unmerged { terrain_type, .. } => terrain_type,
			_ => return,
		};
		if self.cells[r][c].is_crucial()
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
			match self.cells[rr][cc].contents
			{
				Contents::Unmerged {
					terrain_type: tt, ..
				} =>
				{
					if tt != terrain_type
					{
						continue;
					}
				}
				_ => continue,
			}
			// Merge into the other cell.
			self.cells[r][c].contents = Contents::Merged {
				parent_row: rr as u8,
				parent_col: cc as u8,
				parent_terrain_type: terrain_type,
			};
			self.cells[rr][cc].make_more_important();
			break;
		}
	}

	fn force_merge_cell(&mut self, r: usize, c: usize)
	{
		let terrain_type = match self.cells[r][c].contents
		{
			Contents::Unmerged { terrain_type, .. } => terrain_type,
			_ => return,
		};

		let mut closest_parent = None;
		let mut closest_sqdis =
			2 * MAX_DISTANCE_BETWEEN_REGIONS * MAX_DISTANCE_BETWEEN_REGIONS;
		for dr in -DBR_BBOX_RADIUS..=DBR_BBOX_RADIUS
		{
			for dc in -DBR_BBOX_RADIUS..=DBR_BBOX_RADIUS
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
				let cell = &self.cells[r][c];
				let other = &self.cells[rr][cc];
				let other_terrain_type = match other.contents
				{
					Contents::Unmerged {
						terrain_type: tt, ..
					} => tt,
					_ => continue,
				};
				if other_terrain_type != terrain_type
				{
					continue;
				}
				let dx = (other.centroid_x as i32) - (cell.centroid_x as i32);
				let dy = (other.centroid_y as i32) - (cell.centroid_y as i32);
				let sqdis = dx * dx + dy * dy;
				if sqdis
					> MAX_DISTANCE_BETWEEN_REGIONS
						* MAX_DISTANCE_BETWEEN_REGIONS
				{
					continue;
				}
				if sqdis < closest_sqdis
				{
					closest_parent = Some((rr, cc));
					closest_sqdis = sqdis;
				}
			}
		}

		if let Some((rr, cc)) = closest_parent
		{
			// Merge into the other cell, even if it is already merged.
			self.cells[r][c].contents = Contents::Merged {
				parent_row: rr as u8,
				parent_col: cc as u8,
				parent_terrain_type: terrain_type,
			};
		}
		else
		{
			// If we could not merge, cull it.
			self.cells[r][c].contents = Contents::Culled {
				occupation_percentage: 0,
			};
		}
	}

	fn calculate_cell_badness(&self, r: usize, c: usize) -> i8
	{
		let cell = &self.cells[r][c];
		let terrain_type = match cell.contents
		{
			Contents::Unmerged { terrain_type, .. } => terrain_type,
			_ =>
			{
				return 0;
			}
		};
		let mut num_too_close: usize = 0;
		let mut num_close_similar: usize = 0;
		let mut num_close_different: usize = 0;
		for dr in -DBR_BBOX_RADIUS..=DBR_BBOX_RADIUS
		{
			for dc in -DBR_BBOX_RADIUS..=DBR_BBOX_RADIUS
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
				let other_terrain_type = match other.contents
				{
					Contents::Unmerged {
						terrain_type: tt, ..
					} => tt,
					_ => continue,
				};
				let dx = (other.centroid_x as i32) - (cell.centroid_x as i32);
				let dy = (other.centroid_y as i32) - (cell.centroid_y as i32);
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
				if other_terrain_type == terrain_type
				{
					num_close_similar += 1;
				}
				else
				{
					num_close_different += 1;
				}
			}
		}
		let mut badness: i8 = 10;
		if r == 0 || c == 0 || r == GRID_SIZE - 1 || c == GRID_SIZE - 1
		{
			badness += 25;
		}
		badness -= std::cmp::min(num_close_different, 4) as i8;
		badness += std::cmp::min(num_close_similar, 4) as i8;
		badness += 10 * (std::cmp::min(num_too_close, 8) as i8);
		badness
	}

	fn find_parent_region_id(&self, r: usize, c: usize) -> Option<i8>
	{
		let mut rr = r;
		let mut cc = c;
		for _i in 0..10
		{
			match self.cells[rr][cc].contents
			{
				Contents::Region { region_id, .. } =>
				{
					return Some(region_id);
				}
				Contents::Subregion {
					parent_region_id, ..
				} =>
				{
					return Some(parent_region_id);
				}
				Contents::Merged {
					parent_row,
					parent_col,
					parent_terrain_type: _,
				} =>
				{
					rr = parent_row as usize;
					cc = parent_col as usize;
				}
				_ => return None,
			}
		}
		trace("circular parents");
		return None;
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
	Unmerged
	{
		terrain_type: TerrainType,
		merge_importance: u8,
	},
	Merged
	{
		parent_row: u8,
		parent_col: u8,
		parent_terrain_type: TerrainType,
	},
	Culled
	{
		occupation_percentage: u8,
	},
	Region
	{
		region_id: i8,
		terrain_type: TerrainType,
		marker: Option<Marker>,
		occupation_percentage: u8,
	},
	Subregion
	{
		parent_region_id: i8,
		parent_terrain_type: TerrainType,
		occupation_percentage: u8,
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
	fn is_occupied(&self) -> bool
	{
		match self.contents
		{
			Contents::Culled {
				occupation_percentage,
				..
			}
			| Contents::Region {
				occupation_percentage,
				..
			}
			| Contents::Subregion {
				occupation_percentage,
				..
			} => occupation_percentage > 0,
			_ => false,
		}
	}

	fn update_occupation_percentage(&mut self, percentage: u8) -> u8
	{
		match &mut self.contents
		{
			Contents::Culled {
				occupation_percentage,
				..
			}
			| Contents::Region {
				occupation_percentage,
				..
			}
			| Contents::Subregion {
				occupation_percentage,
				..
			} =>
			{
				if *occupation_percentage < percentage
				{
					*occupation_percentage = percentage;
					percentage
				}
				else
				{
					*occupation_percentage
				}
			}
			_ => 0,
		}
	}

	fn make_more_important(&mut self)
	{
		match &mut self.contents
		{
			Contents::Unmerged {
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
			Contents::Unmerged {
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
				TerrainType::Village => (),
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
			self.contents = Contents::Unmerged {
				terrain_type,
				merge_importance: 0,
			}
		}
		else
		{
			self.contents = Contents::Culled {
				occupation_percentage: 0,
			};
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
		Some(TerrainType::Village) => 3,
		Some(TerrainType::Grass) => 1,
		Some(TerrainType::Forest) => 5,
		Some(TerrainType::Hill) => 10,
		Some(TerrainType::Mountain) => 12,
		Some(TerrainType::Water) => 0,
	};
	let offset = v * PROP_GRID_SIZE + u;
	let byte_offset = offset / 2;
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
	let bit_shift = 4 - 4 * (offset % 2);
	let value = (propmap[byte_offset] >> bit_shift) & 0b1111;
	match value
	{
		0 => None,
		1..=2 => Some(TerrainType::Grass),
		3..=4 => Some(TerrainType::Village),
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
	let x = (x / PROP_GRID_CELL_SIZE) * PROP_GRID_CELL_SIZE
		+ PROP_GRID_CELL_SIZE / 2;
	let y = (y / PROP_GRID_CELL_SIZE) * PROP_GRID_CELL_SIZE
		+ PROP_GRID_CELL_SIZE / 2;
	(x, y)
}

fn generate_elevation_noise(seed: i32) -> PerlinNoise2D
{
	let mut elevation = PerlinNoise2D::new(
		NOISE_OCTAVES,
		NOISE_AMPLITUDE_ELEVATION,
		NOISE_FREQUENCY_ELEVATION,
		NOISE_PERSISTENCE_ELEVATION,
		NOISE_LACUNARITY,
		NOISE_SCALE,
		NOISE_BIAS_ELEVATION,
		seed,
	);
	let mut total = 0.0;
	for r in 0..GRID_SIZE
	{
		for c in 0..GRID_SIZE
		{
			let x = c * GRID_CELL_SIZE + GRID_CELL_SIZE / 2;
			let y = r * GRID_CELL_SIZE + GRID_CELL_SIZE / 2;
			let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
			total += e;
		}
	}
	let n = (GRID_SIZE * GRID_SIZE) as f64;
	let average = total / n;
	let mut spread = 0.0;
	for r in 0..GRID_SIZE
	{
		for c in 0..GRID_SIZE
		{
			let x = c * GRID_CELL_SIZE + GRID_CELL_SIZE / 2;
			let y = r * GRID_CELL_SIZE + GRID_CELL_SIZE / 2;
			let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
			let difference = e - average;
			spread += difference * difference;
		}
	}
	let variance = spread / n;
	let standard_deviation = variance.sqrt();
	// Adjust the amplitude to make sure the variance is "nice".
	elevation.set_amplitude(
		NOISE_AMPLITUDE_ELEVATION * NOISE_AMPLITUDE_ELEVATION_MAGIC
			/ standard_deviation,
	);
	// Recalculate the average because we changed the amplitude and the average
	// wasn't centered on 0.
	total = 0.0;
	for r in 0..GRID_SIZE
	{
		for c in 0..GRID_SIZE
		{
			let x = c * GRID_CELL_SIZE + GRID_CELL_SIZE / 2;
			let y = r * GRID_CELL_SIZE + GRID_CELL_SIZE / 2;
			let e = elevation.get_noise(x as f64 + 0.5, y as f64 + 0.5);
			total += e;
		}
	}
	let average = total / n;
	elevation.set_bias(2.0 * NOISE_BIAS_ELEVATION - average);
	elevation
}
