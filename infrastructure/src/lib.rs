//! Infrastructure adapters — concrete implementations of the domain ports.

mod raster;
mod storage;

pub use raster::TiffRasterReader;
pub use storage::{FsExportWriter, FsProjectRepository};
