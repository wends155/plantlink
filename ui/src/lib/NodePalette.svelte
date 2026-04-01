<script>
  import { getNodesByCategory, nodeDefinitions } from './nodeDefinitions.js';
  import * as Icons from 'lucide-svelte';

  const onDragStart = (event, nodeType, data) => {
    event.dataTransfer.setData('application/svelteflow', JSON.stringify({ type: nodeType, data }));
    event.dataTransfer.effectAllowed = 'move';
  };

  // Get nodes grouped by category
  const categories = getNodesByCategory();

  // Category display order
  const categoryOrder = ['Common', 'Network', 'Function'];
</script>

<aside class="node-palette">
  {#each categoryOrder as category}
    {#if categories[category]}
      <div class="palette-category">
        {category}
      </div>
      <div class="p-2 space-y-2">
        {#each Object.entries(categories[category]) as [nodeType, def]}
          <div
            class="palette-item"
            draggable={true}
            on:dragstart={(e) => onDragStart(e, nodeType, def.defaultData)}
            role="button"
            tabindex="0"
          >
            <div
              class="w-4 h-4 flex items-center justify-center rounded-sm"
              style="background-color: {def.color};"
            >
              <svelte:component this={Icons[def.icon]} size={10} color={def.iconColor || '#fff'} />
            </div>
            <span>{def.displayName}</span>
          </div>
        {/each}
      </div>
    {/if}
  {/each}
</aside>
