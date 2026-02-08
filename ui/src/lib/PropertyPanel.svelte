<script>
    import CodeMirror from "svelte-codemirror-editor";
    import { rust } from "@codemirror/lang-rust";
    import { oneDark } from "@codemirror/theme-one-dark";
    import { theme } from "./stores/theme";

    export let selectedNode = null;
    export let onUpdate;
    export let onDelete;
    export let onClose;

    let localData = {};
    let editorTheme = [];

    $: editorTheme = $theme === "dark" ? oneDark : [];

    // Re-sync local data when selected node changes
    // Re-sync local data when selected node changes
    $: if (selectedNode) {
        // Only update localData if it's a DIFFERENT node than what we are currently editing.
        // This prevents the "typing -> update -> prop update -> overwrite" cycle for the same node.
        if (localData.id !== selectedNode.id) {
            localData = { ...selectedNode.data };
            // Use ID from selectedNode to track which node we are editing
            localData.id = selectedNode.id;

            if (selectedNode.type === "rhai-function") {
                localData.code = localData.code || "";
            }
        }
    }

    function applyChanges() {
        if (selectedNode && onUpdate) {
            onUpdate(selectedNode.id, localData);
        }
    }
</script>

<aside
    class="w-[300px] bg-white dark:bg-gray-900 border-l border-[#ccc] dark:border-gray-700 flex flex-col h-full text-[13px]"
>
    <div
        class="bg-[#f3f3f3] dark:bg-gray-800 p-2 font-bold border-b border-[#ccc] dark:border-gray-700 text-[#555] dark:text-gray-400 flex justify-between items-center"
    >
        <span>Properties</span>
        {#if selectedNode}
            <button
                on:click={onClose}
                class="text-gray-500 hover:text-black dark:hover:text-white"
                >✕</button
            >
        {/if}
    </div>

    {#if selectedNode}
        <div class="p-4 space-y-4 flex-1 overflow-y-auto">
            <div
                class="text-xs text-gray-500 dark:text-gray-400 font-mono mb-2"
            >
                ID: {selectedNode.id} <br />
                Type: {selectedNode.type}
            </div>

            <div class="space-y-1">
                <label
                    for="node-name"
                    class="block text-gray-700 dark:text-gray-300 font-medium"
                    >Name</label
                >
                <input
                    id="node-name"
                    type="text"
                    bind:value={localData.name}
                    on:input={applyChanges}
                    class="w-full border border-gray-300 dark:border-gray-600 rounded px-2 py-1 focus:ring-1 focus:ring-blue-500 outline-none text-gray-900 dark:text-gray-200 bg-white dark:bg-gray-800"
                    placeholder="Node Name"
                />
            </div>

            {#if selectedNode.type === "mqtt-in"}
                <div class="space-y-1">
                    <label
                        for="node-topic"
                        class="block text-gray-700 dark:text-gray-300 font-medium"
                        >Topic</label
                    >
                    <input
                        id="node-topic"
                        type="text"
                        bind:value={localData.topic}
                        on:input={applyChanges}
                        class="w-full border border-gray-300 dark:border-gray-600 rounded px-2 py-1 font-mono text-xs text-gray-900 dark:text-gray-200 bg-white dark:bg-gray-800"
                        placeholder="sensor/#"
                    />
                </div>
            {/if}

            {#if selectedNode.type === "rhai-function"}
                <div
                    class="flex-1 flex flex-col h-[300px] border border-gray-300 dark:border-gray-600 rounded overflow-hidden"
                >
                    <label
                        for="node-code"
                        class="block text-gray-700 dark:text-gray-300 font-medium px-2 py-1 bg-gray-50 dark:bg-gray-800 border-b dark:border-gray-600"
                        >Rhai Script</label
                    >
                    <div
                        class="flex-1 overflow-auto bg-white dark:bg-[#1e1e1e]"
                        id="node-code"
                    >
                        <CodeMirror
                            bind:value={localData.code}
                            lang={rust()}
                            theme={editorTheme}
                            on:change={(e) => {
                                // Bind manually to avoid auto-apply
                                localData.code = e.detail.value;
                            }}
                        />
                    </div>
                    <div
                        class="px-2 py-1 bg-gray-50 dark:bg-gray-800 border-t dark:border-gray-600 flex justify-between items-center text-xs"
                    >
                        <span class="text-gray-500 italic"
                            >msg: MessagePayload is input</span
                        >
                        <div class="flex gap-2">
                            <button
                                class="px-2 py-1 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 rounded text-gray-700 dark:text-gray-200"
                                on:click={() => {
                                    // Revert to original data
                                    localData.code =
                                        selectedNode.data.code || "";
                                }}
                            >
                                Discard
                            </button>
                            <button
                                class="px-2 py-1 bg-blue-600 hover:bg-blue-700 text-white rounded"
                                on:click={() => applyChanges()}
                            >
                                Save Script
                            </button>
                        </div>
                    </div>
                </div>
            {/if}

            {#if selectedNode.type === "inject"}
                <div class="space-y-1">
                    <label
                        for="inject-payload"
                        class="block text-gray-700 dark:text-gray-300 font-medium"
                        >Payload (String)</label
                    >
                    <input
                        id="inject-payload"
                        type="text"
                        bind:value={localData.payload}
                        on:input={applyChanges}
                        class="w-full border border-gray-300 dark:border-gray-600 rounded px-2 py-1 text-xs text-gray-900 dark:text-gray-200 bg-white dark:bg-gray-800"
                        placeholder="Hello World"
                    />
                </div>
                <div class="space-y-1">
                    <label
                        for="inject-interval"
                        class="block text-gray-700 dark:text-gray-300 font-medium"
                        >Interval (Seconds)</label
                    >
                    <input
                        id="inject-interval"
                        type="number"
                        bind:value={localData.interval}
                        on:input={applyChanges}
                        class="w-full border border-gray-300 dark:border-gray-600 rounded px-2 py-1 text-xs text-gray-900 dark:text-gray-200 bg-white dark:bg-gray-800"
                        placeholder="0 (disabled)"
                    />
                </div>
            {/if}

            {#if selectedNode.type === "nats-broker"}
                <div class="space-y-1">
                    <label
                        for="nats-url"
                        class="block text-gray-700 dark:text-gray-300 font-medium"
                        >URL</label
                    >
                    <input
                        id="nats-url"
                        type="text"
                        bind:value={localData.url}
                        on:input={applyChanges}
                        class="w-full border border-gray-300 dark:border-gray-600 rounded px-2 py-1 font-mono text-xs text-gray-900 dark:text-gray-200 bg-white dark:bg-gray-800"
                        placeholder="nats://localhost:4222"
                    />
                </div>
            {/if}

            {#if selectedNode.type === "nats-sub" || selectedNode.type === "nats-pub"}
                <div class="space-y-1">
                    <label
                        for="nats-subject"
                        class="block text-gray-700 dark:text-gray-300 font-medium"
                        >Subject</label
                    >
                    <input
                        id="nats-subject"
                        type="text"
                        bind:value={localData.subject}
                        on:input={applyChanges}
                        class="w-full border border-gray-300 dark:border-gray-600 rounded px-2 py-1 font-mono text-xs text-gray-900 dark:text-gray-200 bg-white dark:bg-gray-800"
                        placeholder="events.>"
                    />
                </div>
            {/if}
        </div>

        <div
            class="p-2 border-t border-[#ccc] dark:border-gray-700 bg-[#f9f9f9] dark:bg-gray-800 flex justify-end gap-2"
        >
            <button
                class="px-3 py-1 bg-[#d9534f] text-white rounded hover:bg-[#c9302c] text-xs"
                on:click={() => {
                    if (selectedNode && onDelete) onDelete(selectedNode.id);
                }}>Delete Node</button
            >
        </div>
    {:else}
        <div class="p-4 text-gray-400 italic text-center mt-10">
            Select a node to view properties.
        </div>
    {/if}
</aside>
