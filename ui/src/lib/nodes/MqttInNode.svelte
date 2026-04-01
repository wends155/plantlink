<script>
  import BaseNode from './BaseNode.svelte';
  import { Wifi } from 'lucide-svelte';

  export let data;
  export let selected;
</script>

<BaseNode
  label={data.label || data.name || data.topic || 'mqtt in'}
  color="#a6bbcf"
  inputs={0}
  outputs={1}
  {selected}
>
  <Wifi slot="icon" size={16} color="white" />
</BaseNode>
