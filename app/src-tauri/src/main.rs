//! Vectorify presentation: Tauri backend. Thin driving adapter — each command wraps
//! an existing `application` use case and mutates the backend-owned `Project`
//! aggregate (the single consistency boundary). Live interaction state lives in the
//! frontend; only finalized changes reach here.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dto;

use std::path::Path;
use std::sync::Mutex;

use application as uc;
use domain::sampling::{FeatureId, LayerRef, Project};
use infrastructure::{FsExportWriter, FsProjectRepository, TiffRasterReader};
use tauri::State;

use dto::{OpenResult, RasterRegistered, Snapshot};

const DEFAULT_BASEMAP: &str = "https://tile.openstreetmap.org/{z}/{x}/{y}.png";

struct AppState {
    project: Project,
    reader: TiffRasterReader,
    repo: FsProjectRepository,
    writer: FsExportWriter,
}

impl Default for AppState {
    fn default() -> Self {
        // Start with an OSM basemap so the map isn't blank on launch (as the old UI did).
        let mut project = Project::new();
        let _ = uc::add_basemap(&mut project, DEFAULT_BASEMAP);
        Self { project, reader: TiffRasterReader, repo: FsProjectRepository, writer: FsExportWriter }
    }
}

type St<'a> = State<'a, Mutex<AppState>>;

// ---- read ----
#[tauri::command]
fn project_snapshot(state: St) -> Snapshot {
    Snapshot::of(&state.lock().unwrap().project)
}

// ---- project lifecycle ----
#[tauri::command]
fn new_project(state: St) -> Snapshot {
    let mut g = state.lock().unwrap();
    g.project = Project::new();
    let _ = uc::add_basemap(&mut g.project, DEFAULT_BASEMAP);
    Snapshot::of(&g.project)
}

#[tauri::command]
fn save_project(state: St, path: String) -> Result<(), String> {
    let g = state.lock().unwrap();
    uc::save_project(&g.repo, &g.project, Path::new(&path)).map(|_| ())
}

#[tauri::command]
fn open_project(state: St, path: String) -> Result<OpenResult, String> {
    let mut g = state.lock().unwrap();
    let opened = uc::open_project(&g.repo, &g.reader, Path::new(&path))?;
    g.project = opened.project;
    Ok(OpenResult { snapshot: Snapshot::of(&g.project), missing: opened.missing })
}

// ---- layers ----
#[tauri::command]
fn register_geotiff(state: St, path: String) -> RasterRegistered {
    let mut g = state.lock().unwrap();
    uc::register_geotiff(&mut g.project, Path::new(&path));
    RasterRegistered { snapshot: Snapshot::of(&g.project), key: path.clone(), path }
}

#[tauri::command]
fn add_basemap(state: St, url: String) -> Result<Snapshot, String> {
    let mut g = state.lock().unwrap();
    uc::add_basemap(&mut g.project, &url)?;
    Ok(Snapshot::of(&g.project))
}

#[tauri::command]
fn remove_layer(state: St, index: usize) -> Result<Snapshot, String> {
    let mut g = state.lock().unwrap();
    g.project.remove_layer(index)?;
    Ok(Snapshot::of(&g.project))
}

#[tauri::command]
fn reorder_layer(state: St, index: usize, delta: i32) -> Snapshot {
    let mut g = state.lock().unwrap();
    g.project.reorder_layer(index, delta as isize);
    Snapshot::of(&g.project)
}

// ---- features (each returns nothing or the new id; the frontend refreshes the snapshot) ----
#[tauri::command]
fn add_polygon(state: St, coords: Vec<[f64; 2]>, category: String) -> Result<u64, String> {
    let mut g = state.lock().unwrap();
    uc::draw_polygon(&mut g.project, coords, &category).map(|(id, _)| id.0)
}

#[tauri::command]
fn add_point(state: St, coord: [f64; 2], category: String) -> Result<u64, String> {
    let mut g = state.lock().unwrap();
    uc::draw_point(&mut g.project, coord, &category).map(|(id, _)| id.0)
}

#[tauri::command]
fn reshape_polygon(state: St, id: u64, coords: Vec<[f64; 2]>) -> Result<(), String> {
    let mut g = state.lock().unwrap();
    uc::reshape_polygon(&mut g.project, FeatureId(id), coords).map(|_| ())
}

/// Move a single vertex by index — used for point features (index 0); polygons use
/// `reshape_polygon` so OpenLayers vertex insert/delete is representable.
#[tauri::command]
fn move_vertex(state: St, id: u64, vertex: usize, coord: [f64; 2]) -> Result<(), String> {
    let mut g = state.lock().unwrap();
    uc::move_vertex(&mut g.project, FeatureId(id), vertex, coord).map(|_| ())
}

#[tauri::command]
fn set_category(state: St, id: u64, category: String) -> Result<(), String> {
    let mut g = state.lock().unwrap();
    uc::set_category(&mut g.project, FeatureId(id), &category).map(|_| ())
}

#[tauri::command]
fn delete_feature(state: St, id: u64) -> Result<(), String> {
    let mut g = state.lock().unwrap();
    uc::delete_feature(&mut g.project, FeatureId(id)).map(|_| ())
}

// ---- export ----
#[tauri::command]
fn export_samples(state: St, dir: String) -> Result<(), String> {
    let g = state.lock().unwrap();
    let raster = g.project.layers().iter().find_map(|l| match l {
        LayerRef::Raster { path } => Some(path.clone()),
        _ => None,
    });
    uc::export_samples(&g.writer, &g.project, Path::new(&dir), raster.as_deref()).map(|_| ())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            project_snapshot,
            new_project,
            save_project,
            open_project,
            register_geotiff,
            add_basemap,
            remove_layer,
            reorder_layer,
            add_polygon,
            add_point,
            reshape_polygon,
            move_vertex,
            set_category,
            delete_feature,
            export_samples,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vectorify");
}
