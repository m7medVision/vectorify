<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { api } from "../lib/api";
  import { applySnapshot, fitRaster, clearSelection } from "../lib/mapController";
  import { theme, missing, setStatus } from "../lib/store";

  const xyzPlaceholder = "https://.../{z}/{x}/{y}.png";
  let xyz = "https://tile.openstreetmap.org/{z}/{x}/{y}.png";

  async function openGeotiff() {
    const path = await open({ filters: [{ name: "GeoTIFF", extensions: ["tif", "tiff"] }] });
    if (typeof path !== "string") return;
    try {
      const res = await api.registerGeotiff(path);
      applySnapshot(res.snapshot);
      fitRaster(res.key);
      setStatus("Loaded raster.");
    } catch (e) {
      setStatus(`Failed: ${e}`);
    }
  }

  async function openProject() {
    const path = await open({ filters: [{ name: "Vectorify project", extensions: ["vproj", "json"] }] });
    if (typeof path !== "string") return;
    try {
      const res = await api.openProject(path);
      missing.set(res.missing);
      applySnapshot(res.snapshot);
      clearSelection();
      setStatus(res.missing.length ? `Opened (${res.missing.length} raster(s) missing).` : "Opened project.");
    } catch (e) {
      setStatus(`Failed: ${e}`);
    }
  }

  async function saveProject() {
    const path = await save({ filters: [{ name: "Vectorify project", extensions: ["vproj"] }] });
    if (!path) return;
    try {
      await api.saveProject(path);
      setStatus("Saved project.");
    } catch (e) {
      setStatus(`Failed: ${e}`);
    }
  }

  async function newProject() {
    applySnapshot(await api.newProject());
    clearSelection();
    setStatus("New project.");
  }

  async function addBasemap() {
    try {
      applySnapshot(await api.addBasemap(xyz));
      setStatus("Added basemap.");
    } catch (e) {
      setStatus(`${e}`);
    }
  }

  async function doExport() {
    const dir = await open({ directory: true });
    if (typeof dir !== "string") return;
    try {
      await api.exportSamples(dir);
      setStatus(`Exported to ${dir}`);
    } catch (e) {
      setStatus(`Failed: ${e}`);
    }
  }

  function toggleTheme() {
    theme.update((t) => (t === "dark" ? "light" : "dark"));
  }
</script>

<header class="topbar">
  <span class="brand">Vectorify</span>
  <button on:click={newProject}>New</button>
  <button on:click={openProject}>Open</button>
  <button on:click={saveProject}>Save</button>
  <span class="sep"></span>
  <button on:click={openGeotiff}>Open GeoTIFF</button>
  <input class="xyz" bind:value={xyz} placeholder={xyzPlaceholder} />
  <button on:click={addBasemap}>Add basemap</button>
  <span class="spacer"></span>
  <button on:click={doExport}>Export</button>
  <button on:click={toggleTheme}>{$theme === "dark" ? "Dark" : "Light"}</button>
</header>
