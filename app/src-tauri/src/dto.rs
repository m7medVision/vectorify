//! App-layer IPC DTOs. Deliberately distinct from the domain's own `serde` impls
//! (which back the `.vproj` persistence format) so the wire contract and the
//! aggregate can evolve independently and internals like `next_id` never leak.

use serde::Serialize;

use domain::sampling::{Feature, Kind, LayerRef, Project};

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum LayerDto {
    Raster { key: String, path: String },
    Basemap { key: String, url: String },
}

impl From<&LayerRef> for LayerDto {
    fn from(l: &LayerRef) -> Self {
        match l {
            LayerRef::Raster { path } => LayerDto::Raster { key: path.clone(), path: path.clone() },
            LayerRef::Basemap { url } => LayerDto::Basemap { key: url.clone(), url: url.clone() },
        }
    }
}

#[derive(Serialize)]
pub struct FeatureDto {
    pub id: u64,
    pub kind: &'static str,
    pub coords: Vec<[f64; 2]>,
    pub category: String,
}

impl From<&Feature> for FeatureDto {
    fn from(f: &Feature) -> Self {
        FeatureDto {
            id: f.id.0,
            kind: match f.kind {
                Kind::Polygon => "polygon",
                Kind::Point => "point",
            },
            coords: f.coords.clone(),
            category: f.category.as_str().to_string(),
        }
    }
}

/// Full read-model the frontend reconciles against (keyed upsert by feature id).
#[derive(Serialize)]
pub struct Snapshot {
    pub layers: Vec<LayerDto>,
    pub features: Vec<FeatureDto>,
}

impl Snapshot {
    pub fn of(p: &Project) -> Self {
        Snapshot {
            layers: p.layers().iter().map(LayerDto::from).collect(),
            features: p.features().iter().map(FeatureDto::from).collect(),
        }
    }
}

/// Result of opening a project: the snapshot plus rasters that failed to resolve on
/// disk, so the layer panel can mark them instead of silently dropping them.
#[derive(Serialize)]
pub struct OpenResult {
    pub snapshot: Snapshot,
    pub missing: Vec<String>,
}

/// Result of registering a GeoTIFF: the snapshot plus the file path the webview loads.
#[derive(Serialize)]
pub struct RasterRegistered {
    pub snapshot: Snapshot,
    pub key: String,
    pub path: String,
}
