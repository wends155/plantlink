<script>
    import { getNodesByCategory, nodeDefinitions } from "./nodeDefinitions.js";
    import * as Icons from "lucide-svelte";

    const onDragStart = (event, nodeType, data) => {
        event.dataTransfer.setData(
            "application/svelteflow",
            JSON.stringify({ type: nodeType, data }),
        );
        event.dataTransfer.effectAllowed = "move";
    };

    // Get nodes grouped by category
    const categories = getNodesByCategory();
    
    // Category display order
    const categoryOrder = ["Common", "Network", "Function"];
</script>

<aside
    class="w-[200px] bg-[#fafafa] dark:bg-gray-900 border-r border-[#ccc] dark:border-gray-700 flex flex-col h-full text-[13px]"
>
    {#each categoryOrder as category}
        {#if categories[category]}
            <div
                class="bg-[#f3f3f3] dark:bg-gray-800 p-2 font-bold border-b border-[#ccc] dark:border-gray-700 text-[#555] dark:text-gray-400"
            >
                {category}
            </div>
            <div class="p-2 space-y-2">
                {#each Object.entries(categories[category]) as [nodeType, def]}
                    <div
                        class="flex items-center gap-2 p-1 bg-[#e9e9e9] dark:bg-gray-700 border border-[#999] dark:border-gray-600 rounded cursor-grab hover:border-[#333] dark:hover:border-gray-400 text-gray-800 dark:text-gray-200"
                        draggable={true}
                        on:dragstart={(e) => onDragStart(e, nodeType, def.defaultData)}
                        role="button"
                        tabindex="0"
                    >
                        <div
                            class="w-4 h-4 flex items-center justify-center rounded-sm"
                            style="background-color: {def.color};"
                        >
                            <svelte:component 
                                this={Icons[def.icon]} 
                                size={10} 
                                color={def.iconColor || '#fff'} 
                            />
                        </div>
                        <span>{def.displayName}</span>
                    </div>
                {/each}
            </div>
        {/if}
    {/each}
</aside>
