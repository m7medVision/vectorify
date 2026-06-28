<script lang="ts">
  import { cursor, status } from "../lib/store";
</script>

<footer class="status">
  {#if $cursor}<span class="coord">lon {$cursor.lon.toFixed(5)} &nbsp; lat {$cursor.lat.toFixed(5)}</span>{/if}
  <span class="msg">{$status}</span>
</footer>
