//! `TiffRasterReader` — reads a GeoTIFF (pure-Rust `tiff` crate) into `RasterData`.
//! ponytail: only EPSG:4326/3857, north-up axis-aligned, 8/16-bit gray/rgb/rgba.

use std::path::Path;

use domain::imagery::{Bounds, Projection, RasterData};
use domain::ports::RasterReader;

pub struct TiffRasterReader;

impl RasterReader for TiffRasterReader {
    fn read(&self, path: &Path) -> Result<RasterData, String> {
        use tiff::decoder::Decoder;
        use tiff::tags::Tag;

        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut dec = Decoder::new(file).map_err(|e| e.to_string())?;

        let (w, h) = dec.dimensions().map_err(|e| e.to_string())?;
        let (w, h) = (w as usize, h as usize);

        // Georeferencing: pixel scale + tiepoint (north-up, tiepoint at pixel 0,0).
        let scale = dec
            .get_tag_f64_vec(Tag::ModelPixelScaleTag)
            .map_err(|_| "missing ModelPixelScale (not georeferenced?)".to_string())?;
        let tie = dec
            .get_tag_f64_vec(Tag::ModelTiepointTag)
            .map_err(|_| "missing ModelTiepoint (not georeferenced?)".to_string())?;
        if scale.len() < 2 || tie.len() < 6 {
            return Err("incomplete georeferencing tags".into());
        }
        let (sx, sy) = (scale[0], scale[1]);
        let (ox, oy) = (tie[3], tie[4]);
        let min_x_n = ox;
        let max_y_n = oy;
        let max_x_n = ox + (w as f64) * sx;
        let min_y_n = oy - (h as f64) * sy;

        let bounds = match read_epsg(&mut dec) {
            Some(3857) | Some(3785) => {
                Bounds { min_x: min_x_n, min_y: min_y_n, max_x: max_x_n, max_y: max_y_n }
            }
            Some(4326) => {
                let (a, b) = Projection::lonlat_to_merc(min_x_n, min_y_n);
                let (c, d) = Projection::lonlat_to_merc(max_x_n, max_y_n);
                Bounds { min_x: a, min_y: b, max_x: c, max_y: d }
            }
            Some(code) => {
                return Err(format!("unsupported CRS EPSG:{code}; reproject to 4326 or 3857 first"))
            }
            None => return Err("could not determine CRS".into()),
        };

        let img = dec.read_image().map_err(|e| e.to_string())?;
        let colortype = dec.colortype().map_err(|e| e.to_string())?;
        let rgba = to_rgba(img, colortype, w, h)?;

        // Cap display texture size — GPUs reject textures past ~8–16k px.
        let (rgba, w, h) = downscale_to(rgba, w, h, 8192);
        Ok(RasterData { width: w, height: h, rgba, bounds })
    }
}

/// Nearest-neighbor downscale RGBA so neither side exceeds `max`. No-op if small.
fn downscale_to(rgba: Vec<u8>, w: usize, h: usize, max: usize) -> (Vec<u8>, usize, usize) {
    if w <= max && h <= max {
        return (rgba, w, h);
    }
    let scale = (w.max(h) as f64 / max as f64).ceil() as usize;
    let nw = (w / scale).max(1);
    let nh = (h / scale).max(1);
    let mut out = vec![0u8; nw * nh * 4];
    for y in 0..nh {
        for x in 0..nw {
            let sx = (x * scale).min(w - 1);
            let sy = (y * scale).min(h - 1);
            let s = (sy * w + sx) * 4;
            let d = (y * nw + x) * 4;
            out[d..d + 4].copy_from_slice(&rgba[s..s + 4]);
        }
    }
    (out, nw, nh)
}

/// EPSG code from the GeoKeyDirectory (projected key 3072 or geographic key 2048).
fn read_epsg<R: std::io::Read + std::io::Seek>(dec: &mut tiff::decoder::Decoder<R>) -> Option<u16> {
    use tiff::tags::Tag;
    let keys = dec.get_tag_u16_vec(Tag::Unknown(34735)).ok()?; // GeoKeyDirectoryTag
    if keys.len() < 4 {
        return None;
    }
    let num = keys[3] as usize;
    for i in 0..num {
        let base = 4 + i * 4;
        if base + 3 >= keys.len() {
            break;
        }
        if (keys[base] == 3072 || keys[base] == 2048) && keys[base + 1] == 0 {
            return Some(keys[base + 3]);
        }
    }
    None
}

/// Normalize decoded pixels to RGBA8. 8/16-bit gray/rgb/rgba; 16-bit is >>8.
fn to_rgba(
    img: tiff::decoder::DecodingResult,
    ct: tiff::ColorType,
    w: usize,
    h: usize,
) -> Result<Vec<u8>, String> {
    use tiff::decoder::DecodingResult as D;
    use tiff::ColorType as C;
    let n = w * h;
    let mut out = vec![0u8; n * 4];
    let bytes: Vec<u8> = match img {
        D::U8(v) => v,
        D::U16(v) => v.iter().map(|&x| (x >> 8) as u8).collect(),
        _ => return Err("unsupported pixel type (use 8/16-bit)".into()),
    };
    let chans = match ct {
        C::Gray(_) => 1,
        C::RGB(_) => 3,
        C::RGBA(_) => 4,
        _ => return Err("unsupported color type (use gray/rgb/rgba)".into()),
    };
    for i in 0..n {
        let s = i * chans;
        let (r, g, b, a) = match chans {
            1 => (bytes[s], bytes[s], bytes[s], 255),
            3 => (bytes[s], bytes[s + 1], bytes[s + 2], 255),
            _ => (bytes[s], bytes[s + 1], bytes[s + 2], bytes[s + 3]),
        };
        out[i * 4] = r;
        out[i * 4 + 1] = g;
        out[i * 4 + 2] = b;
        out[i * 4 + 3] = a;
    }
    Ok(out)
}
