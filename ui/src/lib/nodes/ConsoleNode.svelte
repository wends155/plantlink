<script>
    import BaseNode from "./BaseNode.svelte";
    import { Terminal } from "lucide-svelte";

    export let data;
    export let selected;
</script>

<BaseNode
    nodeType="console"
    label={data.label || data.name || "console"}
    {selected}
>
    <Terminal slot="icon" size={16} color="white" />
</BaseNode>
