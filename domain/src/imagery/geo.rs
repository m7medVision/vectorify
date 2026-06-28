//! Web-Mercator projection, value objects, and the raster-data carrier type.
//! ponytail: raw f64 math functions for hot rendering paths; validating value
//! objects (`LonLat`, `Zoom`) guard the construction boundaries.

pub const R: f64 = 6_378_137.0; // web-mercator earth radius (m)
pub const ORIGIN: f64 = std::f64::consts::PI * R; // half world span in meters

/// Geographic coordinate (WGS84), validated on construction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LonLat {
    pub lon: f64,
    pub lat: f64,
}

impl LonLat {
    pub fn new(lon: f64, lat: f64) -> Result<Self, String> {
        if !lon.is_finite() || !lat.is_finite() || lon.abs() > 180.0 || lat.abs() > 90.0 {
            return Err(format!("invalid lon/lat: {lon}, {lat}"));
        }
        Ok(Self { lon, lat })
    }
}

/// Projected coordinate (EPSG:3857 meters). No validation needed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mercator {
    pub x: f64,
    pub y: f64,
}

/// Bounds in web-mercator meters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Bounds {
    pub fn center(&self) -> (f64, f64) {
        ((self.min_x + self.max_x) / 2.0, (self.min_y + self.max_y) / 2.0)
    }
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }
}

/// Slippy-map zoom level, clamped to a valid range on construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zoom(pub u32);

impl Zoom {
    pub fn new(z: u32) -> Self {
        Zoom(z.min(19))
    }
    pub fn get(self) -> u32 {
        self.0
    }
}

/// A slippy-map tile address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub z: u32,
    pub x: u32,
    pub y: u32,
}

/// Decoded raster pixels + mercator footprint; carried from the `RasterReader` port.
pub struct RasterData {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>, // width*height*4
    pub bounds: Bounds,
}

/// Projection domain service: pure mercator/tile math.
pub struct Projection;

impl Projection {
    /// lon/lat (deg) -> web-mercator meters.
    pub fn lonlat_to_merc(lon: f64, lat: f64) -> (f64, f64) {
        let x = lon.to_radians() * R;
        let y = ((lat.to_radians() / 2.0 + std::f64::consts::FRAC_PI_4).tan()).ln() * R;
        (x, y)
    }

    /// web-mercator meters -> lon/lat (deg).
    pub fn merc_to_lonlat(x: f64, y: f64) -> (f64, f64) {
        let lon = (x / R).to_degrees();
        let lat = (2.0 * (y / R).exp().atan() - std::f64::consts::FRAC_PI_2).to_degrees();
        (lon, lat)
    }

    /// Mercator bounds of a tile.
    pub fn tile_bounds(t: TileCoord) -> Bounds {
        let n = (1u64 << t.z) as f64;
        let span = 2.0 * ORIGIN;
        let tile = span / n;
        let min_x = -ORIGIN + t.x as f64 * tile;
        let max_y = ORIGIN - t.y as f64 * tile;
        Bounds { min_x, min_y: max_y - tile, max_x: min_x + tile, max_y }
    }

    /// Pick a zoom so one tile ~= 256 screen px at the given meters/pixel.
    pub fn zoom_for_mpp(mpp: f64) -> Zoom {
        let world_px = (2.0 * ORIGIN) / mpp;
        let z = (world_px / 256.0).log2().round().clamp(0.0, 19.0) as u32;
        Zoom::new(z)
    }

    /// Tile x/y range covering a mercator view window at zoom z. Returns (tx0,tx1,ty0,ty1).
    pub fn tile_range(z: Zoom, view: &Bounds) -> (u32, u32, u32, u32) {
        let n = 1u64 << z.get();
        let span = 2.0 * ORIGIN;
        let tox = |v: f64| (((v + ORIGIN) / span) * n as f64).floor();
        let tx0 = tox(view.min_x).clamp(0.0, (n - 1) as f64) as u32;
        let tx1 = tox(view.max_x).clamp(0.0, (n - 1) as f64) as u32;
        // y axis flipped (north = small index)
        let toy = |v: f64| (((ORIGIN - v) / span) * n as f64).floor();
        let ty0 = toy(view.max_y).clamp(0.0, (n - 1) as f64) as u32;
        let ty1 = toy(view.min_y).clamp(0.0, (n - 1) as f64) as u32;
        (tx0, tx1, ty0, ty1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merc_roundtrip() {
        for &(lon, lat) in &[(0.0, 0.0), (12.5, 41.9), (-122.4, 37.8), (139.7, 35.7)] {
            let (x, y) = Projection::lonlat_to_merc(lon, lat);
            let (lo, la) = Projection::merc_to_lonlat(x, y);
            assert!((lo - lon).abs() < 1e-6, "lon {lon} -> {lo}");
            assert!((la - lat).abs() < 1e-6, "lat {lat} -> {la}");
        }
    }

    #[test]
    fn tile_zero_is_whole_world() {
        let b = Projection::tile_bounds(TileCoord { z: 0, x: 0, y: 0 });
        assert!((b.min_x + ORIGIN).abs() < 1.0);
        assert!((b.max_x - ORIGIN).abs() < 1.0);
    }

    #[test]
    fn lonlat_rejects_out_of_range() {
        assert!(LonLat::new(0.0, 91.0).is_err());
        assert!(LonLat::new(181.0, 0.0).is_err());
        assert!(LonLat::new(10.0, 45.0).is_ok());
    }
}
