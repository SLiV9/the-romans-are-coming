//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

mod wasm4;

#[cfg(feature = "buddy-alloc")]
mod alloc;

mod decree;
mod global_state;
mod level;
mod map;
mod menu;
mod palette;
mod sprites;

use global_state::Wrapper;
use level::Level;
use menu::Menu;

static GAME: Wrapper<Game> = Wrapper::new(Game::Loading);

const QUICK_TEST: bool = true;

enum Game
{
	Loading,
	Menu(Menu),
	Level(Level),
}

enum Progress
{
	Menu,
	Level(level::Transition),
}

#[no_mangle]
fn update()
{
	let game = GAME.get_mut();
	let transition = match game
	{
		Game::Loading =>
		{
			setup();
			if QUICK_TEST
			{
				Some(Progress::Level(level::Transition { rng_seed: 0 }))
			}
			else
			{
				Some(Progress::Menu)
			}
		}
		Game::Menu(menu) =>
		{
			let transition = menu.update();
			match transition
			{
				Some(menu::Transition::Start { rng_seed }) =>
				{
					let data = level::Transition { rng_seed };
					Some(Progress::Level(data))
				}
				None => None,
			}
		}
		Game::Level(level) =>
		{
			let transition = level.update();
			match transition
			{
				Some(data) => Some(Progress::Level(data)),
				None => None,
			}
		}
	};
	match transition
	{
		Some(Progress::Menu) =>
		{
			let menu = Menu::new();
			*game = Game::Menu(menu);
		}
		Some(Progress::Level(data)) =>
		{
			let level = Level::new(data.rng_seed);
			*game = Game::Level(level);
		}
		None => (),
	}

	match game
	{
		Game::Loading => (),
		Game::Menu(menu) => menu.draw(),
		Game::Level(level) => level.draw(),
	}
}

fn setup()
{
	palette::setup();
}
