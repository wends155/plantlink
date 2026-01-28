<script>
    import BaseNode from "./BaseNode.svelte";
    import { ArrowUpFromLine } from "lucide-svelte";
    import { nodeStatuses } from "../stores/nodeStatus.js";

    export let data;
    export let selected;
    export let id;

    $: status = $nodeStatuses[id];
</script>

<BaseNode
    label={data.label || data.subject || "nats-pub"}
    color="#2563eb"
    inputs={2}
    outputs={0}
    {selected}
    {status}
>
    <ArrowUpFromLine slot="icon" size={16} color="#fff" />
</BaseNode>
