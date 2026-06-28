<script lang="ts">
  import { onMount } from "svelte";
  import { init, setTool, setVisibility, refresh } from "./lib/mapController";
  import { tool, theme, visibility } from "./lib/store";
  import TopBar from "./components/TopBar.svelte";
  import LeftPanel from "./components/LeftPanel.svelte";
  import RightPanel from "./components/RightPanel.svelte";
  import StatusBar from "./components/StatusBar.svelte";

  let mapEl: HTMLElement;

  onMount(() => {
    init(mapEl);
    refresh();
    const offTool = tool.subscribe((v) => setTool(v));
    const offVis = visibility.subscribe((vis) => setVisibility(vis));
    return () => {
      offTool();
      offVis();
    };
  });

  $: if (typeof document !== "undefined") document.documentElement.dataset.theme = $theme;
</script>

<div class="app">
  <TopBar />
  <div class="body">
    <LeftPanel />
    <div class="map" bind:this={mapEl}></div>
    <RightPanel />
  </div>
  <StatusBar />
</div>
