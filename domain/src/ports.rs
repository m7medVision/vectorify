//! Ports: the domain's outbound interfaces. Adapters in `infrastructure` implement
//! them; use cases in `application` depend on these traits, never the adapters.

use std::path::Path;

use crate::imagery::RasterData;
use crate::sampling::Project;

/// Reads a GeoTIFF into decoded pixels + mercator bounds. Used for export sampling;
/// display is handled by the webview (OpenLayers), not this port.
pub trait RasterReader {
    fn read(&self, path: &Path) -> Result<RasterData, String>;
}

/// Loads/saves the project aggregate to a local file.
pub trait ProjectRepository {
    fn save(&self, path: &Path, project: &Project) -> Result<(), String>;
    fn load(&self, path: &Path) -> Result<Project, String>;
}

/// Writes export artifacts (filename, contents) into a directory.
pub trait ExportWriter {
    fn write_all(&self, dir: &Path, files: &[(String, String)]) -> Result<(), String>;
}
