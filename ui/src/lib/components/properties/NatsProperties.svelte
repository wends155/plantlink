<script>
  import { Input } from '../ui';
  
  export let data = {};
  export let nodeType = '';
  export let onUpdate;
  
  function handleChange() {
    onUpdate(data);
  }
</script>

{#if nodeType === 'nats-broker'}
  <Input
    label="URL"
    id="nats-url"
    bind:value={data.url}
    on:input={handleChange}
    placeholder="nats://localhost:4222"
    mono
  />
{/if}

{#if nodeType === 'nats-sub' || nodeType === 'nats-pub'}
  <Input
    label="Subject"
    id="nats-subject"
    bind:value={data.subject}
    on:input={handleChange}
    placeholder="events.>"
    mono
  />
{/if}
