// OpenLayers map controller: owns the map, reconciles layers/features from backend
// snapshots (keyed upsert by FeatureId), and runs the Draw/Modify/Snap/Select
// interactions, syncing only *finalized* geometry to the Rust Project.
import OlMap from "ol/Map";
import View from "ol/View";
import TileLayer from "ol/layer/Tile";
import WebGLTileLayer from "ol/layer/WebGLTile";
import VectorLayer from "ol/layer/Vector";
import type BaseLayer from "ol/layer/Base";
import XYZ from "ol/source/XYZ";
import GeoTIFF from "ol/source/GeoTIFF";
import VectorSource from "ol/source/Vector";
import Feature from "ol/Feature";
import { Polygon, Point } from "ol/geom";
import type Geometry from "ol/geom/Geometry";
import { Draw, Modify, Snap, Select } from "ol/interaction";
import type Interaction from "ol/interaction/Interaction";
import { click } from "ol/events/condition";
import { Style, Stroke, Fill, Circle as CircleStyle } from "ol/style";
import { fromLonLat, toLonLat } from "ol/proj";
import { convertFileSrc } from "@tauri-apps/api/core";

import { api, type Snapshot, type FeatureDto, type LonLat } from "./api";
import { categoryColor } from "./colors";
import {
  snapshot as snapshotStore,
  selected,
  cursor,
  drawCount,
  setStatus,
  type Tool,
} from "./store";

const VECTOR_Z = 1000;
const DRAFT_Z = 1001;

let map: OlMap;
const finalizedSource = new VectorSource();
const draftSource = new VectorSource();
const layerMap = new Map<string, BaseLayer>(); // backend layer key -> OL layer

let currentVis: Record<string, boolean> = {};
let selectedId: number | null = null;
let interactions: Interaction[] = [];
let drawCleanup: (() => void) | null = null;

// ---- styling ----
function styleFor(feature: Feature<Geometry>): Style {
  const cat = (feature.get("category") as string) || "sample";
  const [r, g, b] = categoryColor(cat);
  const sel = feature.getId() === selectedId;
  const line = `rgb(${r},${g},${b})`;
  const geom = feature.getGeometry();
  if (geom instanceof Point) {
    return new Style({
      image: new CircleStyle({
        radius: 5,
        fill: new Fill({ color: line }),
        stroke: sel ? new Stroke({ color: "#fff", width: 2 }) : undefined,
      }),
    });
  }
  return new Style({
    fill: new Fill({ color: `rgba(${r},${g},${b},0.25)` }),
    stroke: new Stroke({ color: line, width: sel ? 3 : 1.5 }),
  });
}

const finalizedLayer = new VectorLayer({ source: finalizedSource, style: styleFor as any, zIndex: VECTOR_Z });
const draftLayer = new VectorLayer({
  source: draftSource,
  zIndex: DRAFT_Z,
  style: new Style({
    fill: new Fill({ color: "rgba(255,255,0,0.15)" }),
    stroke: new Stroke({ color: "rgb(255,255,0)", width: 2 }),
    image: new CircleStyle({ radius: 4, fill: new Fill({ color: "rgb(255,255,0)" }) }),
  }),
});

// ---- geometry <-> domain coords (domain stores open rings in WGS84) ----
function ringToLonLat(poly: Polygon): LonLat[] {
  const ring = poly.getCoordinates()[0].map((c) => {
    const l = toLonLat(c);
    return [l[0], l[1]] as LonLat;
  });
  if (ring.length > 1) {
    const a = ring[0];
    const b = ring[ring.length - 1];
    if (a[0] === b[0] && a[1] === b[1]) ring.pop(); // drop OL's closing duplicate
  }
  return ring;
}

function geomOf(f: FeatureDto): Geometry {
  if (f.kind === "polygon") {
    const ring = f.coords.map((c) => fromLonLat(c));
    if (ring.length) ring.push(ring[0]); // close for OL
    return new Polygon([ring]);
  }
  return new Point(fromLonLat(f.coords[0]));
}

// ---- snapshot reconciliation ----
function syncLayers(snap: Snapshot) {
  const present = new Set<string>();
  snap.layers.forEach((l, i) => {
    present.add(l.key);
    let layer = layerMap.get(l.key);
    if (!layer) {
      layer =
        l.kind === "basemap"
          ? new TileLayer({ source: new XYZ({ url: l.url }) })
          : new WebGLTileLayer({
              // ponytail: default projection; web-mercator COGs align. Exotic CRS
              // GeoTIFFs are a known risk (see design) — validate per task 4.2.
              source: new GeoTIFF({ sources: [{ url: convertFileSrc(l.path) }], normalize: true }),
            });
      layerMap.set(l.key, layer);
      map.addLayer(layer);
    }
    layer.setZIndex(i); // snapshot order = bottom -> top
    layer.setVisible(currentVis[l.key] !== false);
  });
  for (const [key, layer] of layerMap) {
    if (!present.has(key)) {
      map.removeLayer(layer);
      layerMap.delete(key);
    }
  }
}

function syncFinalized(snap: Snapshot) {
  const present = new Set<number>();
  for (const f of snap.features) {
    present.add(f.id);
    const existing = finalizedSource.getFeatureById(f.id) as Feature<Geometry> | null;
    if (existing) {
      existing.setGeometry(geomOf(f));
      existing.set("category", f.category);
    } else {
      const feat = new Feature({ geometry: geomOf(f) });
      feat.setId(f.id);
      feat.set("category", f.category);
      feat.set("kind", f.kind);
      finalizedSource.addFeature(feat);
    }
  }
  for (const feat of finalizedSource.getFeatures()) {
    if (!present.has(feat.getId() as number)) finalizedSource.removeFeature(feat);
  }
}

export function applySnapshot(snap: Snapshot) {
  snapshotStore.set(snap);
  syncLayers(snap);
  syncFinalized(snap);
  finalizedLayer.changed();
}

export async function refresh() {
  applySnapshot(await api.snapshot());
}

export function setVisibility(vis: Record<string, boolean>) {
  currentVis = vis;
  for (const [key, layer] of layerMap) layer.setVisible(vis[key] !== false);
}

/** Fit the view to a just-registered GeoTIFF's extent. */
export function fitRaster(key: string) {
  const layer = layerMap.get(key) as WebGLTileLayer | undefined;
  const src = layer?.getSource() as GeoTIFF | undefined;
  if (!src) return;
  src
    .getView()
    .then((v) => {
      if (v.center) map.getView().setCenter(v.center as [number, number]);
      if (v.zoom != null) map.getView().setZoom(v.zoom as number);
    })
    .catch(() => setStatus("GeoTIFF registered but could not auto-fit the view."));
}

// ---- interactions per tool ----
function clearInteractions() {
  for (const i of interactions) map.removeInteraction(i);
  interactions = [];
  if (drawCleanup) {
    drawCleanup();
    drawCleanup = null;
  }
}

function addDraw(type: "Polygon" | "Point") {
  const draw = new Draw({ source: draftSource, type });
  draw.on("drawstart", (e) => {
    const geom = e.feature.getGeometry();
    if (geom instanceof Polygon) {
      const update = () => drawCount.set(geom.getCoordinates()[0].length - 1);
      update();
      geom.on("change", update);
    }
  });
  draw.on("drawend", async (e) => {
    const geom = e.feature.getGeometry();
    try {
      if (geom instanceof Polygon) {
        await api.addPolygon(ringToLonLat(geom), "sample");
        setStatus("Polygon added.");
      } else if (geom instanceof Point) {
        const c = toLonLat(geom.getCoordinates());
        await api.addPoint([c[0], c[1]], "sample");
        setStatus("Point added.");
      }
      await refresh();
    } catch (err) {
      setStatus(`Rejected: ${err}`);
    } finally {
      drawCount.set(0);
      // finalized features come from the snapshot; discard the optimistic draft.
      setTimeout(() => draftSource.clear(), 0);
    }
  });
  const snap = new Snap({ source: finalizedSource });
  const snapDraft = new Snap({ source: draftSource });
  interactions = [draw, snap, snapDraft];
  interactions.forEach((i) => map.addInteraction(i));

  // Undo last vertex (Backspace / right-click) and cancel (Esc) — OL Draw has no
  // default bindings for these, but the vector-drawing spec requires them.
  const onKey = (ev: KeyboardEvent) => {
    if (ev.key === "Backspace") {
      ev.preventDefault();
      draw.removeLastPoint();
    } else if (ev.key === "Escape") {
      draw.abortDrawing();
      draftSource.clear();
      drawCount.set(0);
    }
  };
  const target = map.getTargetElement();
  const onCtx = (ev: MouseEvent) => {
    ev.preventDefault();
    draw.removeLastPoint();
  };
  window.addEventListener("keydown", onKey);
  target?.addEventListener("contextmenu", onCtx);
  drawCleanup = () => {
    window.removeEventListener("keydown", onKey);
    target?.removeEventListener("contextmenu", onCtx);
  };
}

function addPanEdit() {
  const select = new Select({ condition: click, layers: [finalizedLayer], style: null });
  select.on("select", () => {
    const feat = select.getFeatures().item(0) as Feature<Geometry> | undefined;
    selectedId = (feat?.getId() as number) ?? null;
    selected.set(
      feat
        ? {
            id: feat.getId() as number,
            kind: (feat.get("kind") as "polygon" | "point") ?? "polygon",
            category: (feat.get("category") as string) ?? "sample",
            coords: [],
          }
        : null
    );
    finalizedLayer.changed();
  });

  const modify = new Modify({ source: finalizedSource });
  modify.on("modifyend", async (e) => {
    for (const feat of e.features.getArray() as Feature<Geometry>[]) {
      const id = feat.getId() as number;
      const geom = feat.getGeometry();
      try {
        if (geom instanceof Polygon) await api.reshapePolygon(id, ringToLonLat(geom));
        else if (geom instanceof Point) {
          const c = toLonLat(geom.getCoordinates());
          await api.moveVertex(id, 0, [c[0], c[1]]);
        }
      } catch (err) {
        setStatus(`Edit rejected: ${err}`);
      }
    }
    await refresh();
  });

  interactions = [select, modify];
  interactions.forEach((i) => map.addInteraction(i));
}

export function setTool(t: Tool) {
  clearInteractions();
  draftSource.clear();
  drawCount.set(0);
  if (t === "polygon") addDraw("Polygon");
  else if (t === "point") addDraw("Point");
  else addPanEdit();
}

export function clearSelection() {
  selectedId = null;
  selected.set(null);
  finalizedLayer.changed();
}

export function init(target: HTMLElement) {
  map = new OlMap({
    target,
    layers: [finalizedLayer, draftLayer],
    view: new View({ center: fromLonLat([0, 20]), zoom: 2 }),
  });
  map.on("pointermove", (evt) => {
    const [lon, lat] = toLonLat(evt.coordinate);
    cursor.set({ lon, lat });
  });
  setTool("pan");
}
