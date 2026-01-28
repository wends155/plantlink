<script>
    import BaseNode from "./BaseNode.svelte";
    import { Terminal } from "lucide-svelte";

    export let data;
    export let selected;
</script>

<BaseNode
    label={data.label || data.name || "console"}
    color="#87a980"
    inputs={1}
    outputs={0}
    {selected}
>
    <Terminal slot="icon" size={16} color="white" />
</BaseNode>
