<script>
  import { Handle, Position } from "@xyflow/svelte";
  import { getNodeDefinition } from "../nodeDefinitions.js";
  import { nodeStatuses } from "../stores/nodeStatus.js";

  export let label = "Node";
  export let color = "";
  export let nodeType = "";  // If provided, auto-fetch config
  export let inputs = 0;
  export let outputs = 0;
  export let inputLabels = [];
  export let outputLabels = [];
  export let selected = false;
  export let id = null;  // Node ID for status lookup

  // Auto-fetch status by ID
  $: status = id ? $nodeStatuses[id] : null;

  // Auto-fetch from nodeDefinitions if nodeType provided
  $: def = nodeType ? getNodeDefinition(nodeType) : null;
  $: actualInputs = def ? def.inputs.length : inputs;
  $: actualOutputs = def ? def.outputs.length : outputs;
  $: actualInputLabels = def ? def.inputs.map(p => p.label) : inputLabels;
  $: actualOutputLabels = def ? def.outputs.map(p => p.label) : outputLabels;
  $: actualColor = def ? def.color : (color || "#a6bbcf");
  
  // Compute state class
  $: stateClass = status?.state === 'error' ? 'node--error' 
                : status?.state === 'running' ? 'node--running'
                : status?.state === 'stopped' ? 'node--stopped'
                : '';
</script>

<div
  class="shadow-md rounded-[5px] border border-gray-500 bg-[#f3f3f3] dark:bg-gray-800 dark:border-gray-600 min-w-[120px] flex items-stretch h-[30px] overflow-hidden transition-shadow parent-node {stateClass} {selected
    ? 'ring-2 ring-red-500 border-red-500'
    : ''}"
>
  {#each Array(actualInputs) as _, i}
    <Handle
      type="target"
      position={Position.Left}
      id={`input_${i}`}
      title={actualInputLabels[i] || `Input ${i}`}
      class="!w-2.5 !h-2.5 !bg-[#999] !border-none !-left-[5px]"
      style="top: {actualInputs === 1 ? '50%' : `${((i + 1) / (actualInputs + 1)) * 100}%`}; transform: translateY(-50%);"
    />
  {/each}

  <!-- Icon Area -->
  <div
    class="w-[30px] flex items-center justify-center border-r border-[#0000001a] dark:border-gray-700"
    style="background-color: {actualColor};"
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

  {#each Array(actualOutputs) as _, i}
    <Handle
      type="source"
      position={Position.Right}
      id={`output_${i}`}
      title={actualOutputLabels[i] || `Output ${i}`}
      class="!w-2.5 !h-2.5 !bg-[#999] !border-none !-right-[5px]"
      style="top: {actualOutputs === 1 ? '50%' : `${((i + 1) / (actualOutputs + 1)) * 100}%`}; transform: translateY(-50%);"
    />
  {/each}
</div>

<style>
  /* Custom styles to override SvelteFlow handle defaults if needed */
</style>
