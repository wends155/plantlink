<script>
  import { Input } from '../ui';
  
  export let data = {};
  export let onUpdate;
  
  function handleChange() {
    onUpdate(data);
  }
</script>

<Input
  label="Payload (String)"
  id="inject-payload"
  bind:value={data.payload}
  on:input={handleChange}
  placeholder="Hello World"
/>

<Input
  label="Interval (Seconds)"
  id="inject-interval"
  type="number"
  bind:value={data.interval}
  on:input={handleChange}
  placeholder="0 (disabled)"
/>
