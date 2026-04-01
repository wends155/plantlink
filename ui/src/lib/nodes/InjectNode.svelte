<script>
  import BaseNode from './BaseNode.svelte';
  import { Play } from 'lucide-svelte';

  export let data;
  export let selected;
</script>

<BaseNode nodeType="inject" label={data.label || data.name || 'inject'} {selected}>
  <Play slot="icon" size={16} color="white" />
</BaseNode>
