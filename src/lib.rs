#![feature(async_closure)]
use bevy::{prelude::*, render::render_graph::RenderGraph};

mod loader;
mod map;
mod tiled_map;
pub use map::*;
mod pipeline;
pub use pipeline::*;
mod tile_map;
pub use tile_map::*;

/// Adds support for GLTF file loading to Apps
#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<map::Map>()
            .init_asset_loader::<loader::TiledMapLoader>()
            .add_system(process_loaded_tile_maps);

        let resources = app.resources();
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
        render_graph.add_tile_map_graph(resources);
    }
}
