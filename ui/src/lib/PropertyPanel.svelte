<script>
  import { propertyComponents } from './components/properties';
  import { Input, Button } from './components/ui';
  import { nodeStatuses } from './stores/nodeStatus';
  
  export let selectedNode = null;
  export let onUpdate;
  export let onDelete;
  export let onClose;
  
  let localData = {};
  
  // Get node status
  $: nodeStatus = selectedNode ? $nodeStatuses[selectedNode.id] : null;
  
  // Sync local data when node changes
  $: if (selectedNode && localData.id !== selectedNode.id) {
    localData = { ...selectedNode.data, id: selectedNode.id };
    if (selectedNode.type === 'rhai-function') {
      localData.code = localData.code || "";
    }
  }
  
  function handlePropertyUpdate(newData) {
    localData = { ...localData, ...newData };
    if (onUpdate) onUpdate(selectedNode.id, localData);
  }
  
  $: PropertyComponent = selectedNode ? propertyComponents[selectedNode.type] : null;
</script>

<aside class="property-panel">
  <header class="panel-header">
    <span>Properties</span>
    {#if selectedNode}
      <button class="btn-icon" on:click={onClose} aria-label="Close">✕</button>
    {/if}
  </header>

  {#if selectedNode}
    <div class="panel-content space-y-4">
      <!-- Error Banner -->
      {#if nodeStatus?.state === 'error'}
        <div class="error-banner">
          <span class="error-title">⚠️ Error</span>
          <p class="error-message">{nodeStatus.message}</p>
        </div>
      {/if}

      <div class="text-xs text-[var(--color-text-muted)] font-mono">
        ID: {selectedNode.id}<br />
        Type: {selectedNode.type}
      </div>

      <Input
        label="Name"
        id="node-name"
        bind:value={localData.name}
        on:input={() => handlePropertyUpdate(localData)}
        placeholder="Node Name"
      />

      {#if PropertyComponent}
        <svelte:component
          this={PropertyComponent}
          bind:data={localData}
          nodeType={selectedNode.type}
          originalCode={selectedNode.data?.code}
          onUpdate={handlePropertyUpdate}
        />
      {/if}
    </div>

    <footer class="panel-footer">
      <Button variant="danger" size="sm" on:click={() => onDelete?.(selectedNode.id)}>
        Delete Node
      </Button>
    </footer>
  {:else}
    <div class="panel-content text-[var(--color-text-muted)] italic text-center mt-10">
      Select a node to view properties.
    </div>
  {/if}
</aside>
