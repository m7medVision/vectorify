<script lang="ts">
  import { tool, selected, drawCount, snapshot, setStatus, type Tool } from "../lib/store";
  import { api } from "../lib/api";
  import { refresh, clearSelection } from "../lib/mapController";

  let cat = "";
  $: cat = $selected ? $selected.category : "";

  function pick(t: Tool) {
    tool.set(t);
    clearSelection();
  }
  async function applyCat() {
    if (!$selected || !cat.trim()) return;
    try {
      await api.setCategory($selected.id, cat);
      await refresh();
    } catch (e) {
      setStatus(`${e}`);
    }
  }
  async function del() {
    if (!$selected) return;
    await api.deleteFeature($selected.id);
    clearSelection();
    await refresh();
    setStatus("Feature deleted.");
  }
</script>

<aside class="right">
  <h3>Tools</h3>
  <div class="tools">
    <button class:active={$tool === "pan"} on:click={() => pick("pan")}>Pan / Select</button>
    <button class:active={$tool === "polygon"} on:click={() => pick("polygon")}>Polygon</button>
    <button class:active={$tool === "point"} on:click={() => pick("point")}>Point</button>
  </div>

  {#if $tool === "polygon"}
    <div class="hint">
      {#if $drawCount === 0}Click to place vertices.{:else}{$drawCount} pt(s){#if $drawCount < 3} — need ≥3{/if} · double-click or click first vertex to finish · Backspace/right-click undo · Esc cancel{/if}
    </div>
  {:else if $tool === "point"}
    <div class="hint">Click to place a point.</div>
  {/if}

  <h3>Selected feature</h3>
  {#if $selected}
    <label class="field">category
      <input bind:value={cat} on:change={applyCat} />
    </label>
    <button on:click={del}>Delete</button>
  {:else}
    <p class="muted">none</p>
  {/if}
  <p class="muted">{$snapshot.features.length} feature(s)</p>
</aside>
