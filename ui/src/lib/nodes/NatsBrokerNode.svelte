<script>
    import BaseNode from "./BaseNode.svelte";
    import { Database } from "lucide-svelte";

    export let data;
    export let selected;
    export let id;
</script>

<BaseNode
    {id}
    nodeType="nats-broker"
    label={data.label || "NATS Broker"}
    {selected}
>
    <Database slot="icon" size={16} color="#444" />
</BaseNode>
