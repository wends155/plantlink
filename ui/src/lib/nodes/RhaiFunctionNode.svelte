<script>
  import BaseNode from './BaseNode.svelte';
  import { FileJson } from 'lucide-svelte';

  export let data;
  export let selected;
  export let id;
</script>

<BaseNode
  {id}
  nodeType="rhai-function"
  label={data.label || data.name || 'function'}
  {selected}
  {status}
>
  <FileJson slot="icon" size={16} color="#444" />
</BaseNode>
