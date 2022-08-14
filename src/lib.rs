//
// Part of the-romans-are-coming
// Copyright (c) 2022 Sander in 't Veld
// License: MIT
//

mod wasm4;

#[cfg(feature = "buddy-alloc")]
mod alloc;

mod global_state;
mod palette;

use global_state::Wrapper;

static GAME: Wrapper<Game> = Wrapper::new(Game::Loading);

enum Game
{
    Loading,
    Menu,
}

enum Progress
{
    Entry,
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
            Some(Progress::Entry)
        }
        Game::Menu =>
        {
            None
        }
    };
    match transition
    {
        Some(Progress::Entry) =>
        {
            *game = Game::Menu;
        }
        None => (),
    }

    match game
    {
        Game::Loading => (),
        Game::Menu => (),
    }
}

fn setup()
{
    palette::setup();
}
