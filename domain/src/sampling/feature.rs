//! Feature entity and its value objects.

use serde::{Deserialize, Serialize};

/// Opaque feature identity. Minted only by the `Project` aggregate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeatureId(pub u64);

/// Non-empty category label (value object, validated on construction).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category(String);

impl Category {
    pub fn new(s: impl Into<String>) -> Result<Self, String> {
        let s = s.into();
        if s.trim().is_empty() {
            return Err("category must not be empty".into());
        }
        Ok(Category(s))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Polygon,
    Point,
}

/// A drawn sample feature. Coordinates are WGS84 lon/lat.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feature {
    pub id: FeatureId,
    pub kind: Kind,
    pub coords: Vec<[f64; 2]>,
    pub category: Category,
}

impl Feature {
    /// Invariant check for a feature's geometry given its kind.
    pub(crate) fn geometry_ok(kind: Kind, coords: &[[f64; 2]]) -> Result<(), String> {
        match kind {
            Kind::Point => {
                if coords.len() != 1 {
                    return Err("a point must have exactly one coordinate".into());
                }
            }
            Kind::Polygon => {
                if coords.len() < 3 {
                    return Err("a polygon needs at least 3 vertices".into());
                }
            }
        }
        Ok(())
    }
}
