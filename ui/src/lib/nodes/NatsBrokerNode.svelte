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
    nodeType="nats-broker"
    label={data.label || "NATS Broker"}
    {selected}
    {status}
>
    <Database slot="icon" size={16} color="#444" />
</BaseNode>
