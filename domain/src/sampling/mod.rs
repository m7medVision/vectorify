//! Sampling core domain: the `Project` aggregate and its parts.

pub mod export;
pub mod feature;
pub mod layer;
pub mod project;

pub use export::SampleExport;
pub use feature::{Category, Feature, FeatureId, Kind};
pub use layer::{LayerRef, XyzTemplate};
pub use project::Project;
