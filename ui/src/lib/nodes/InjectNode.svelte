<script>
    import BaseNode from "./BaseNode.svelte";
    import { Play } from "lucide-svelte";

    export let data;
    export let selected;
</script>

<BaseNode
    label={data.label || data.name || "inject"}
    color="#a6bbcf"
    inputs={0}
    outputs={1}
    {selected}
>
    <Play slot="icon" size={16} color="white" />
</BaseNode>
