// Typed wrappers over the Tauri backend commands. The frontend reaches domain
// logic ONLY through these (plus the dialog/fs plugins) — see desktop-shell spec.
import { invoke } from "@tauri-apps/api/core";

export type LonLat = [number, number];

export type LayerDto =
  | { kind: "raster"; key: string; path: string }
  | { kind: "basemap"; key: string; url: string };

export interface FeatureDto {
  id: number;
  kind: "polygon" | "point";
  coords: LonLat[];
  category: string;
}

export interface Snapshot {
  layers: LayerDto[];
  features: FeatureDto[];
}

export interface OpenResult {
  snapshot: Snapshot;
  missing: string[];
}

export interface RasterRegistered {
  snapshot: Snapshot;
  key: string;
  path: string;
}

export const api = {
  snapshot: () => invoke<Snapshot>("project_snapshot"),
  newProject: () => invoke<Snapshot>("new_project"),
  saveProject: (path: string) => invoke<void>("save_project", { path }),
  openProject: (path: string) => invoke<OpenResult>("open_project", { path }),
  registerGeotiff: (path: string) => invoke<RasterRegistered>("register_geotiff", { path }),
  addBasemap: (url: string) => invoke<Snapshot>("add_basemap", { url }),
  removeLayer: (index: number) => invoke<Snapshot>("remove_layer", { index }),
  reorderLayer: (index: number, delta: number) =>
    invoke<Snapshot>("reorder_layer", { index, delta }),
  addPolygon: (coords: LonLat[], category: string) =>
    invoke<number>("add_polygon", { coords, category }),
  addPoint: (coord: LonLat, category: string) => invoke<number>("add_point", { coord, category }),
  reshapePolygon: (id: number, coords: LonLat[]) =>
    invoke<void>("reshape_polygon", { id, coords }),
  moveVertex: (id: number, vertex: number, coord: LonLat) =>
    invoke<void>("move_vertex", { id, vertex, coord }),
  setCategory: (id: number, category: string) => invoke<void>("set_category", { id, category }),
  deleteFeature: (id: number) => invoke<void>("delete_feature", { id }),
  exportSamples: (dir: string) => invoke<void>("export_samples", { dir }),
};
