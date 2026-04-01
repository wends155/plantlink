<script>
  import { Input } from '../ui';

  export let data = {};
  export let onUpdate;

  function handleChange() {
    onUpdate(data);
  }
</script>

<Input
  label="Topic"
  id="node-topic"
  bind:value={data.topic}
  on:input={handleChange}
  placeholder="sensor/#"
  mono
/>
