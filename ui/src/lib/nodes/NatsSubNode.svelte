<script>
    import BaseNode from "./BaseNode.svelte";
    import { ArrowDownToLine } from "lucide-svelte";
    import { nodeStatuses } from "../stores/nodeStatus.js";

    export let data;
    export let selected;
    export let id;

    $: status = $nodeStatuses[id];
</script>

<BaseNode
    nodeType="nats-sub"
    label={data.label || data.subject || "nats-sub"}
    {selected}
    {status}
>
    <ArrowDownToLine slot="icon" size={16} color="#fff" />
</BaseNode>
