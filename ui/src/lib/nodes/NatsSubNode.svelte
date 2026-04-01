<script>
  import BaseNode from './BaseNode.svelte';
  import { ArrowDownToLine } from 'lucide-svelte';

  export let data;
  export let selected;
  export let id;
</script>

<BaseNode {id} nodeType="nats-sub" label={data.label || data.subject || 'nats-sub'} {selected}>
  <ArrowDownToLine slot="icon" size={16} color="#fff" />
</BaseNode>
