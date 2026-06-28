//! Imagery supporting domain: projection math, bounds, tiles, raster pixel data.

pub mod geo;

pub use geo::{Bounds, LonLat, Mercator, Projection, RasterData, TileCoord, Zoom};
