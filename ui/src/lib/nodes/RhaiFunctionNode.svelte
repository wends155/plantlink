<script>
  import BaseNode from "./BaseNode.svelte";
  import { FileJson } from "lucide-svelte";
  import { nodeStatuses } from "../stores/nodeStatus.js";

  export let data;
  export let selected;
  export let id;

  $: status = $nodeStatuses[id];
</script>

<BaseNode
  label={data.label || data.name || "function"}
  color="#fdd0a2"
  inputs={1}
  outputs={1}
  {selected}
  {status}
>
  <FileJson slot="icon" size={16} color="#444" />
</BaseNode>
