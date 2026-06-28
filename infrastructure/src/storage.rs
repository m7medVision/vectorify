//! Filesystem adapters: project repository and export writer. (File dialogs are
//! handled by the Tauri dialog plugin in the presentation layer.)

use std::path::Path;

use domain::ports::{ExportWriter, ProjectRepository};
use domain::sampling::Project;

/// Saves/loads the project aggregate as a `.vproj` JSON file.
pub struct FsProjectRepository;

impl ProjectRepository for FsProjectRepository {
    fn save(&self, path: &Path, project: &Project) -> Result<(), String> {
        let json = serde_json::to_string_pretty(project).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    fn load(&self, path: &Path) -> Result<Project, String> {
        let s = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut project: Project = serde_json::from_str(&s).map_err(|e| e.to_string())?;
        project.validate()?; // re-check invariants — deserialization bypasses constructors
        Ok(project)
    }
}

/// Writes export artifacts into a directory.
pub struct FsExportWriter;

impl ExportWriter for FsExportWriter {
    fn write_all(&self, dir: &Path, files: &[(String, String)]) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        for (name, contents) in files {
            std::fs::write(dir.join(name), contents).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
