<script>
  import { Handle, Position } from "@xyflow/svelte";

  export let label = "Node";
  export let color = "#a6bbcf"; // Default Node-RED color
  export let inputs = 1;
  export let outputs = 1;
  export let selected = false;
  export let status = null;
</script>

<div
  class="shadow-md rounded-[5px] border border-gray-500 {status?.state ===
  'error'
    ? 'bg-red-100 dark:bg-red-900'
    : 'bg-[#f3f3f3] dark:bg-gray-800 dark:border-gray-600'} min-w-[120px] flex items-stretch h-[30px] overflow-hidden transition-shadow parent-node {selected
    ? 'ring-2 ring-red-500 border-red-500'
    : ''}"
>
  {#if inputs > 0}
    <Handle
      type="target"
      position={Position.Left}
      class="!w-2.5 !h-2.5 !bg-[#999] !border-none !-left-[5px]"
    />
  {/if}

  <!-- Icon Area -->
  <div
    class="w-[30px] flex items-center justify-center border-r border-[#0000001a] dark:border-gray-700"
    style="background-color: {color};"
  >
    <slot name="icon">
      <div class="w-2 h-2 rounded-full bg-white opacity-50"></div>
    </slot>
  </div>

  <!-- Content -->
  <div
    class="flex-1 px-2 flex items-center text-[13px] font-sans text-gray-900 dark:text-gray-200 select-none whitespace-nowrap"
  >
    {label}
  </div>

  <!-- Status / Decoration (Optional) -->
  <div class="absolute -bottom-4 left-1 text-[9px] text-gray-500 hidden">
    running
  </div>

  {#if outputs > 0}
    <Handle
      type="source"
      position={Position.Right}
      class="!w-2.5 !h-2.5 !bg-[#999] !border-none !-right-[5px]"
    />
  {/if}
</div>

<style>
  /* Custom styles to override SvelteFlow handle defaults if needed */
</style>
