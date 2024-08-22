# Solar4X

This is a game in active development stage, inspired by 4X space games.

It is designed to be lightweight, modular and hopefully fun! (coming soon)

The idea was proposed by [FrancoisBrucker](https://github.com/FrancoisBrucker), and development started as an internship project by [ben3dninja](https://github.com/ben3dninja).

The game itself is built with the [Bevy](https://bevyengine.org) game engine, and all the menus are displayed in the terminal using the [ratatui](https://ratatui.rs/) crate.

## Building the game from source
1. Install the latest stable version of [Rust](https://www.rust-lang.org/)
2. Install the OS dependencies for Bevy following their [instructions](https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies)
3. Clone the repository and `cd` inside the directory
4. Run `cargo build` and let it compile (compiling Bevy takes time)

Optionnally, you can clone the repository first and use the provided Nix shell to install Rust and the Bevy OS dependencies automatically using [Nix](https://nixos.org/) magic!

## Running the game
Run `cargo run --bin client` or `cargo run --bin server` depending on which binary you want to run.

## Keybindings
All the keybindings are described in a `keymap.toml` file which can be changed you like.
As for mouse inputs, panning is done by holding the left button, and selecting by left clicking on a body/prediction.
