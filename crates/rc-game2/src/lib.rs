//! Game logic of Ratchet & Clank: Going Commando (R&C2), ported from the game's code like `rc-game` is for the first
//! game (docs/plan/rac2_bringup.md): the boot's front end ([`front_end`], [`gui`]) and Ratchet ([`hero`]). The engine
//! (`rc-engine`, its `rac2` modules) loads the data, plays the movies, reads the pad and draws what this crate lays
//! out. The systems that are the same code in both games (the pad record, the collision queries, Ratchet's animation
//! player) are `rc-game`'s, used as they are.

pub mod actor_1330;
pub mod init_1330;
pub mod state_1330;
pub mod camera;
pub mod collision;
pub mod controller_132d;
pub mod effects;
pub mod front_end;
pub mod front_audio;
pub mod gui;
pub mod hero;
pub mod hits;
pub mod items;
pub mod melee;
pub mod objects;
pub mod pvars;
pub mod spawn;
pub mod targeting;
pub mod teleport_actors;
pub mod teleport_world;
pub mod teleport_modifiers;
pub mod env_sample;
pub mod script_queue;
pub mod world;
