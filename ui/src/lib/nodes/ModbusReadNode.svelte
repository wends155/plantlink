<script>
  import BaseNode from "./BaseNode.svelte";
  import { Cpu } from "lucide-svelte";

  export let data;
  export let selected;
</script>

<BaseNode
  label={data.label || data.name || "modbus read"}
  color="#e04e5d"
  inputs={1}
  outputs={1}
  {selected}
>
  <Cpu slot="icon" size={16} color="white" />
</BaseNode>
