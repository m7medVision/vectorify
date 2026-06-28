//! `SampleExport` domain service: pure builders for the export artifacts.
//! Returns (filename, contents) pairs; writing them to disk is a port concern.

use super::feature::{Feature, Kind};

pub struct SampleExport;

impl SampleExport {
    /// Build all export files. Errors if there is nothing to export.
    pub fn build(
        features: &[Feature],
        raster_path: Option<&str>,
    ) -> Result<Vec<(String, String)>, String> {
        if features.is_empty() {
            return Err("nothing to export — draw at least one feature first".into());
        }
        Ok(vec![
            ("samples.geojson".into(), Self::geojson(features)),
            ("sample.ipynb".into(), Self::notebook(raster_path)),
            ("README.md".into(), Self::readme()),
        ])
    }

    /// GeoJSON FeatureCollection (WGS84) from features.
    pub fn geojson(features: &[Feature]) -> String {
        let feats: Vec<serde_json::Value> = features
            .iter()
            .map(|f| {
                let geom = match f.kind {
                    Kind::Point => serde_json::json!({
                        "type": "Point",
                        "coordinates": f.coords.first().copied().unwrap_or([0.0, 0.0]),
                    }),
                    Kind::Polygon => {
                        let mut ring: Vec<[f64; 2]> = f.coords.clone();
                        if ring.first() != ring.last() {
                            if let Some(&first) = ring.first() {
                                ring.push(first); // close the ring
                            }
                        }
                        serde_json::json!({ "type": "Polygon", "coordinates": [ring] })
                    }
                };
                serde_json::json!({
                    "type": "Feature",
                    "properties": { "id": f.id.0, "category": f.category.as_str() },
                    "geometry": geom,
                })
            })
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "type": "FeatureCollection",
            "features": feats,
        }))
        .unwrap_or_default()
    }

    /// Starter notebook: geopandas loads the GeoJSON, rasterio clips a chip per polygon.
    pub fn notebook(raster_path: Option<&str>) -> String {
        let raster = raster_path.unwrap_or("PUT_YOUR_GEOTIFF_PATH_HERE.tif");
        let cells = [
            "import geopandas as gpd\ngdf = gpd.read_file('samples.geojson')\ngdf".to_string(),
            format!(
                "import os, rasterio\nfrom rasterio.mask import mask\n\nRASTER = {raster:?}\nos.makedirs('chips', exist_ok=True)\npolys = gdf[gdf.geometry.type == 'Polygon']\nwith rasterio.open(RASTER) as src:\n    polys = polys.to_crs(src.crs)\n    for _, row in polys.iterrows():\n        img, tr = mask(src, [row.geometry], crop=True)\n        meta = src.meta.copy()\n        meta.update(height=img.shape[1], width=img.shape[2], transform=tr)\n        with rasterio.open(f\"chips/{{int(row.id)}}.tif\", 'w', **meta) as dst:\n            dst.write(img)\nprint('wrote', len(polys), 'chips')"
            ),
        ];
        let json_cells: Vec<serde_json::Value> = cells
            .iter()
            .map(|src| {
                serde_json::json!({
                    "cell_type": "code",
                    "execution_count": null,
                    "metadata": {},
                    "outputs": [],
                    "source": src,
                })
            })
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "cells": json_cells,
            "metadata": { "kernelspec": { "name": "python3", "display_name": "Python 3" } },
            "nbformat": 4,
            "nbformat_minor": 5,
        }))
        .unwrap_or_default()
    }

    fn readme() -> String {
        "# Vectorify sample export\n\n- `samples.geojson` — drawn features (WGS84).\n- `sample.ipynb` — loads them and clips a raster chip per polygon into `chips/`.\n\nRun: `pip install geopandas rasterio` then open the notebook.\n".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampling::{Category, FeatureId};

    #[test]
    fn geojson_closes_polygon_ring() {
        let f = Feature {
            id: FeatureId(1),
            kind: Kind::Polygon,
            coords: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            category: Category::new("x").unwrap(),
        };
        let v: serde_json::Value = serde_json::from_str(&SampleExport::geojson(&[f])).unwrap();
        let ring = &v["features"][0]["geometry"]["coordinates"][0];
        assert_eq!(ring.as_array().unwrap().len(), 4, "ring must be closed");
    }

    #[test]
    fn build_rejects_empty() {
        assert!(SampleExport::build(&[], None).is_err());
    }
}
