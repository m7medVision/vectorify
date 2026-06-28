//! Layer references and the validated XYZ template value object.

use serde::{Deserialize, Serialize};

/// Validated XYZ tile URL template (must contain {z}/{x}/{y}).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct XyzTemplate(String);

impl XyzTemplate {
    pub fn parse(s: impl Into<String>) -> Result<Self, String> {
        let s = s.into();
        if s.contains("{z}") && s.contains("{x}") && s.contains("{y}") {
            Ok(XyzTemplate(s))
        } else {
            Err("XYZ URL must contain {z}/{x}/{y}".into())
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A logical layer in the project: a local raster file or an XYZ basemap.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerRef {
    Raster { path: String },
    Basemap { url: String },
}
