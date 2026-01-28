<script>
    import BaseNode from "./BaseNode.svelte";
    import { Database } from "lucide-svelte";
    import { nodeStatuses } from "../stores/nodeStatus.js";

    export let data;
    export let selected;
    export let id;

    $: status = $nodeStatuses[id];
</script>

<BaseNode
    label={data.label || "NATS Broker"}
    color="#e5e7eb"
    inputs={0}
    outputs={1}
    {selected}
    {status}
>
    <Database slot="icon" size={16} color="#444" />
</BaseNode>
