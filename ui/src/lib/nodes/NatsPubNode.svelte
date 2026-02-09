<script>
    import BaseNode from "./BaseNode.svelte";
    import { ArrowUpFromLine } from "lucide-svelte";

    export let data;
    export let selected;
    export let id;
</script>

<BaseNode
    {id}
    nodeType="nats-pub"
    label={data.label || data.subject || "nats-pub"}
    {selected}
>
    <ArrowUpFromLine slot="icon" size={16} color="#fff" />
</BaseNode>
