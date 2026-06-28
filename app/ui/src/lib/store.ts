import { writable } from "svelte/store";
import type { Snapshot, FeatureDto } from "./api";

export type Tool = "pan" | "polygon" | "point";

export const snapshot = writable<Snapshot>({ layers: [], features: [] });
export const tool = writable<Tool>("pan");
export const status = writable<string>(
  "Loaded OpenStreetMap basemap. Open a GeoTIFF or draw to start."
);
export const cursor = writable<{ lon: number; lat: number } | null>(null);
export const theme = writable<"dark" | "light">("dark");
/** layer key -> visible (absent = visible). Frontend-only presentation state. */
export const visibility = writable<Record<string, boolean>>({});
export const selected = writable<FeatureDto | null>(null);
export const missing = writable<string[]>([]);
/** live vertex count while a polygon draft is in progress (0 = not drawing). */
export const drawCount = writable<number>(0);

export function setStatus(msg: string) {
  status.set(msg);
}
