<script>
    import { Wifi, Cpu, FileJson, Play, Terminal } from "lucide-svelte";

    const onDragStart = (event, nodeType, data) => {
        event.dataTransfer.setData(
            "application/svelteflow",
            JSON.stringify({ type: nodeType, data }),
        );
        event.dataTransfer.effectAllowed = "move";
    };
</script>

<aside
    class="w-[200px] bg-[#fafafa] dark:bg-gray-900 border-r border-[#ccc] dark:border-gray-700 flex flex-col h-full text-[13px]"
>
    <div
        class="bg-[#f3f3f3] dark:bg-gray-800 p-2 font-bold border-b border-[#ccc] dark:border-gray-700 text-[#555] dark:text-gray-400"
    >
        Common
    </div>
    <div class="p-2 space-y-2">
        <div
            class="flex items-center gap-2 p-1 bg-[#e9e9e9] dark:bg-gray-700 border border-[#999] dark:border-gray-600 rounded cursor-grab hover:border-[#333] dark:hover:border-gray-400 text-gray-800 dark:text-gray-200"
            draggable={true}
            on:dragstart={(e) =>
                onDragStart(e, "inject", {
                    name: "inject",
                    payload: "",
                    interval: 0,
                })}
            role="button"
            tabindex="0"
        >
            <div
                class="w-4 h-4 bg-[#a6bbcf] flex items-center justify-center rounded-sm"
            >
                <Play size={10} color="white" />
            </div>
            <span>inject</span>
        </div>

        <div
            class="flex items-center gap-2 p-1 bg-[#e9e9e9] dark:bg-gray-700 border border-[#999] dark:border-gray-600 rounded cursor-grab hover:border-[#333] dark:hover:border-gray-400 text-gray-800 dark:text-gray-200"
            draggable={true}
            on:dragstart={(e) => onDragStart(e, "console", { name: "console" })}
            role="button"
            tabindex="0"
        >
            <div
                class="w-4 h-4 bg-[#87a980] flex items-center justify-center rounded-sm"
            >
                <Terminal size={10} color="white" />
            </div>
            <span>console</span>
        </div>
    </div>

    <div
        class="bg-[#f3f3f3] dark:bg-gray-800 p-2 font-bold border-b border-[#ccc] dark:border-gray-700 text-[#555] dark:text-gray-400"
    >
        Network
    </div>
    <div class="p-2 space-y-2">
        <div
            class="flex items-center gap-2 p-1 bg-[#e9e9e9] dark:bg-gray-700 border border-[#999] dark:border-gray-600 rounded cursor-grab hover:border-[#333] dark:hover:border-gray-400 text-gray-800 dark:text-gray-200"
            draggable={true}
            on:dragstart={(e) =>
                onDragStart(e, "mqtt-in", { topic: "topic/#" })}
            role="button"
            tabindex="0"
        >
            <div
                class="w-4 h-4 bg-[#a6bbcf] flex items-center justify-center rounded-sm"
            >
                <Wifi size={10} color="white" />
            </div>
            <span>mqtt in</span>
        </div>

        <div
            class="flex items-center gap-2 p-1 bg-[#e9e9e9] dark:bg-gray-700 border border-[#999] dark:border-gray-600 rounded cursor-grab hover:border-[#333] dark:hover:border-gray-400 text-gray-800 dark:text-gray-200"
            draggable={true}
            on:dragstart={(e) =>
                onDragStart(e, "modbus-read", { name: "Read Holding" })}
            role="button"
            tabindex="0"
        >
            <div
                class="w-4 h-4 bg-[#e04e5d] flex items-center justify-center rounded-sm"
            >
                <Cpu size={10} color="white" />
            </div>
            <span>modbus read</span>
        </div>
    </div>

    <div
        class="bg-[#f3f3f3] dark:bg-gray-800 p-2 font-bold border-b border-[#ccc] dark:border-gray-700 text-[#555] dark:text-gray-400 mt-2"
    >
        Function
    </div>
    <div class="p-2 space-y-2">
        <div
            class="flex items-center gap-2 p-1 bg-[#e9e9e9] dark:bg-gray-700 border border-[#999] dark:border-gray-600 rounded cursor-grab hover:border-[#333] dark:hover:border-gray-400 text-gray-800 dark:text-gray-200"
            draggable={true}
            on:dragstart={(e) =>
                onDragStart(e, "rhai-function", { name: "function", code: "" })}
            role="button"
            tabindex="0"
        >
            <div
                class="w-4 h-4 bg-[#fdd0a2] flex items-center justify-center rounded-sm"
            >
                <FileJson size={10} color="#444" />
            </div>
            <span>function</span>
        </div>
    </div>
</aside>
