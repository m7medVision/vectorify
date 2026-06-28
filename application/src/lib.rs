//! Application use cases: orchestrate the domain through ports. Plain functions —
//! the presentation owns the concrete adapters and the `Project` and calls these.

use std::path::Path;

use domain::events::DomainEvent;
use domain::imagery::RasterData;
use domain::ports::{ExportWriter, ProjectRepository, RasterReader};
use domain::sampling::{Category, FeatureId, Project, XyzTemplate};

fn category(name: &str) -> Result<Category, String> {
    Category::new(name)
}

/// Read a GeoTIFF and register it as a raster layer. Returns the decoded data so
/// the presentation can build a texture.
pub fn open_geotiff(
    reader: &dyn RasterReader,
    project: &mut Project,
    path: &Path,
) -> Result<(RasterData, Vec<DomainEvent>), String> {
    let data = reader.read(path)?;
    let events = project.add_raster_layer(path.to_string_lossy().into_owned());
    Ok((data, events))
}

/// Register a GeoTIFF as a raster layer WITHOUT decoding it — the webview renders
/// the file for display. Use `open_geotiff` only when Rust needs the pixels (export).
pub fn register_geotiff(project: &mut Project, path: &Path) -> Vec<DomainEvent> {
    project.add_raster_layer(path.to_string_lossy().into_owned())
}

/// Add an XYZ basemap layer from a URL template.
pub fn add_basemap(project: &mut Project, url: &str) -> Result<Vec<DomainEvent>, String> {
    let template = XyzTemplate::parse(url)?;
    Ok(project.add_basemap_layer(template))
}

pub fn draw_polygon(
    project: &mut Project,
    coords: Vec<[f64; 2]>,
    cat: &str,
) -> Result<(FeatureId, Vec<DomainEvent>), String> {
    project.add_polygon(coords, category(cat)?)
}

pub fn draw_point(
    project: &mut Project,
    coord: [f64; 2],
    cat: &str,
) -> Result<(FeatureId, Vec<DomainEvent>), String> {
    project.add_point(coord, category(cat)?)
}

pub fn move_vertex(
    project: &mut Project,
    id: FeatureId,
    vertex: usize,
    coord: [f64; 2],
) -> Result<Vec<DomainEvent>, String> {
    project.move_vertex(id, vertex, coord)
}

pub fn reshape_polygon(
    project: &mut Project,
    id: FeatureId,
    coords: Vec<[f64; 2]>,
) -> Result<Vec<DomainEvent>, String> {
    project.reshape_polygon(id, coords)
}

pub fn delete_feature(project: &mut Project, id: FeatureId) -> Result<Vec<DomainEvent>, String> {
    project.delete_feature(id)
}

pub fn set_category(
    project: &mut Project,
    id: FeatureId,
    cat: &str,
) -> Result<Vec<DomainEvent>, String> {
    project.set_category(id, category(cat)?)
}

/// Export samples (GeoJSON + notebook + README) into a directory via the writer port.
pub fn export_samples(
    writer: &dyn ExportWriter,
    project: &Project,
    dir: &Path,
    raster_path: Option<&str>,
) -> Result<Vec<DomainEvent>, String> {
    let files = domain::sampling::SampleExport::build(project.features(), raster_path)?;
    writer.write_all(dir, &files)?;
    Ok(vec![DomainEvent::ProjectExported])
}

pub fn save_project(
    repo: &dyn ProjectRepository,
    project: &Project,
    path: &Path,
) -> Result<Vec<DomainEvent>, String> {
    repo.save(path, project)?;
    Ok(vec![DomainEvent::ProjectSaved])
}

/// Result of opening a project: the aggregate plus the rasters that loaded and the
/// references that are missing on disk.
pub struct OpenedProject {
    pub project: Project,
    pub rasters: Vec<(String, RasterData)>,
    pub missing: Vec<String>,
    pub events: Vec<DomainEvent>,
}

pub fn open_project(
    repo: &dyn ProjectRepository,
    reader: &dyn RasterReader,
    path: &Path,
) -> Result<OpenedProject, String> {
    use domain::sampling::LayerRef;
    let project = repo.load(path)?;
    let mut rasters = Vec::new();
    let mut missing = Vec::new();
    for layer in project.layers() {
        if let LayerRef::Raster { path } = layer {
            match reader.read(Path::new(path)) {
                Ok(data) => rasters.push((path.clone(), data)),
                Err(_) => missing.push(path.clone()),
            }
        }
    }
    Ok(OpenedProject { project, rasters, missing, events: vec![DomainEvent::ProjectOpened] })
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ports::RasterReader;
    use std::path::Path;

    // In-memory fake adapter — proves use cases run with no real file/GUI/network.
    struct FakeReader;
    impl RasterReader for FakeReader {
        fn read(&self, _path: &Path) -> Result<RasterData, String> {
            Ok(RasterData {
                width: 1,
                height: 1,
                rgba: vec![0, 0, 0, 255],
                bounds: domain::imagery::Bounds { min_x: 0.0, min_y: 0.0, max_x: 1.0, max_y: 1.0 },
            })
        }
    }

    #[test]
    fn open_geotiff_against_fake_adds_layer() {
        let mut p = Project::new();
        let (data, events) = open_geotiff(&FakeReader, &mut p, Path::new("/x.tif")).unwrap();
        assert_eq!(data.width, 1);
        assert_eq!(events, vec![DomainEvent::LayerAdded]);
        assert_eq!(p.layers().len(), 1);
    }

    #[test]
    fn add_basemap_validates_template() {
        let mut p = Project::new();
        assert!(add_basemap(&mut p, "https://no-placeholders").is_err());
        assert!(add_basemap(&mut p, "https://t/{z}/{x}/{y}.png").is_ok());
    }
}
