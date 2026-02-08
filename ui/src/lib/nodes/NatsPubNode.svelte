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
    nodeType="nats-pub"
    label={data.label || data.subject || "nats-pub"}
    {selected}
    {status}
>
    <ArrowUpFromLine slot="icon" size={16} color="#fff" />
</BaseNode>
