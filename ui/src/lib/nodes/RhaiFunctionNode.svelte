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
  nodeType="rhai-function"
  label={data.label || data.name || "function"}
  {selected}
  {status}
>
  <FileJson slot="icon" size={16} color="#444" />
</BaseNode>
