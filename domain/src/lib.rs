//! Vectorify domain core (DDD). Pure: no GUI/IO/framework dependencies.
//!
//! Bounded contexts:
//! - [`sampling`] — core domain: the `Project` aggregate, features, categories, export.
//! - [`imagery`] — supporting domain: projection math, bounds, tiles, raster data.

pub mod events;
pub mod imagery;
pub mod ports;
pub mod sampling;
