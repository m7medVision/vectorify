<script lang="ts">
  import { snapshot, visibility, missing } from "../lib/store";
  import { api, type LayerDto } from "../lib/api";
  import { applySnapshot, fitRaster } from "../lib/mapController";

  function label(l: LayerDto): string {
    return l.kind === "basemap" ? "basemap" : l.path.split(/[\\/]/).pop() || l.path;
  }
  function toggle(key: string, v: boolean) {
    visibility.update((m) => ({ ...m, [key]: v }));
  }
  async function reorder(i: number, d: number) {
    applySnapshot(await api.reorderLayer(i, d));
  }
  async function remove(i: number) {
    applySnapshot(await api.removeLayer(i));
  }
</script>

<aside class="left">
  <h3>Layers</h3>
  {#each $snapshot.layers as l, i (l.key)}
    <div class="layer">
      <label class="vis">
        <input
          type="checkbox"
          checked={$visibility[l.key] !== false}
          on:change={(e) => toggle(l.key, e.currentTarget.checked)}
        />
        <span class:missing={l.kind === "raster" && $missing.includes(l.path)}>{label(l)}</span>
      </label>
      <div class="row">
        <button title="up" on:click={() => reorder(i, -1)}>↑</button>
        <button title="down" on:click={() => reorder(i, 1)}>↓</button>
        {#if l.kind === "raster"}<button title="fit" on:click={() => fitRaster(l.key)}>Fit</button>{/if}
        <button title="remove" on:click={() => remove(i)}>✕</button>
      </div>
    </div>
  {/each}
  {#if $snapshot.layers.length === 0}<p class="muted">no layers</p>{/if}
</aside>
