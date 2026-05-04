//! # Universes
//!
//! Since the engine is data-driven, a Universe represents all the "things"
//! that you can use to create your worlds.

use indexmap::IndexMap;
use serde::Deserialize;
use std::{ffi::OsStr, path::PathBuf};

pub mod example;

pub mod block_type;
pub mod data_pack;
pub mod material;
pub mod texture;
pub mod universe;

pub mod prelude {
    pub use super::block_type::*;
    pub use super::data_pack::*;
    pub use super::material::*;
    pub use super::texture::*;
    pub use super::universe::*;
}

pub trait Id {
    fn id(&self) -> String;
}

#[test]
fn test_load_universe() {
    use std::path::PathBuf;
    use universe::Universe;

    let u = Universe::from_directory(PathBuf::from("./examples/example_universe")).unwrap();
    dbg!(u);
}

pub fn extract_all_from_folder<T: 'static + Id + for<'de> Deserialize<'de>>(
    target: &mut IndexMap<String, T>,
    dir: PathBuf,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.extension() == Some(&OsStr::new("ron")) {
            let raw_block_definition = std::fs::read_to_string(entry.path())?;
            println!("\n\nReading {}\n", raw_block_definition);
            let block_definition: T =
                ron::from_str(raw_block_definition.as_str()).expect("Failed to parse ron file.");
            target.insert(block_definition.id(), block_definition);
        }
    }

    Ok(())
}
