<script>
    import {
        SvelteFlow,
        Background,
        Controls,
        MiniMap,
        useSvelteFlow,
    } from "@xyflow/svelte";
    import "@xyflow/svelte/dist/style.css";

    import { getNodeTypes } from "./nodes/registry.js";

    import NodePalette from "./NodePalette.svelte";
    import PropertyPanel from "./PropertyPanel.svelte";
    import ThemeToggle from "./components/ThemeToggle.svelte";
    import { theme } from "./stores/theme";

    const nodeTypes = getNodeTypes();

    let nodes = [];
    let edges = [];

    let isRunning = false;

    let selectedNode = null;
    const { screenToFlowPosition, deleteElements } = useSvelteFlow();

    const onDragOver = (event) => {
        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
    };

    const onDrop = (event) => {
        event.preventDefault();
        const dataStr = event.dataTransfer.getData("application/svelteflow");
        if (!dataStr) return;

        const data = JSON.parse(dataStr);
        const position = screenToFlowPosition({
            x: event.clientX,
            y: event.clientY,
        });

        const newNode = {
            id: `${Math.random().toString(36).substr(2, 9)}`,
            type: data.type,
            position,
            data: data.data,
        };

        nodes = [...nodes, newNode];
    };

    // Handle manual deletion if SvelteFlow's deleteKeyCode isn't catching it
    const onKeyDown = (event) => {
        if (event.key === "Delete" || event.key === "Backspace") {
            const selectedNodes = nodes.filter((n) => n.selected);
            const selectedEdges = edges.filter((e) => e.selected);
            if (selectedNodes.length > 0 || selectedEdges.length > 0) {
                deleteElements({ nodes: selectedNodes, edges: selectedEdges });
            }
        }
    };

    $: {
        const selectedNodes = nodes.filter((n) => n.selected);
        selectedNode = selectedNodes.length === 1 ? selectedNodes[0] : null;
        if (selectedNode) {
            console.log("Node Selected:", selectedNode);
        }
    }

    const updateNodeData = (id, newData) => {
        console.log("Update Node Data:", id, newData);
        nodes = nodes.map((n) => {
            if (n.id === id) {
                // Svelte 5 needs a new object reference or explicit mutation to trigger reactivity
                // If using SvelteFlow store via props, creating a new object for 'data' is key.
                const updatedData = { ...n.data, ...newData };

                // Update 'label' if 'name' or 'topic' is present to show it on the node
                if (newData.name) updatedData.label = newData.name;
                else if (newData.topic && n.type === "mqtt-in")
                    updatedData.label = newData.topic;

                return { ...n, data: updatedData };
            }
            return n;
        });
        selectedNode = nodes.find((n) => n.id === id);
    };

    const deployFlow = async () => {
        const flow = { nodes, edges };
        console.log("Deploying flow:", flow);
        try {
            const response = await fetch("/api/flow", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(flow),
            });
            if (response.ok) {
                console.log("Flow deployed successfully");
                isRunning = true;
            } else {
                console.error("Failed to deploy flow", response.statusText);
                alert("Failed to start flow");
            }
        } catch (err) {
            console.error("Error deploying flow", err);
            alert("Error deploying flow");
        }
    };

    const stopFlow = async () => {
        try {
            const response = await fetch("/api/flow/stop", {
                method: "POST",
            });
            if (response.ok) {
                console.log("Flow stopped successfully");
                isRunning = false;
            } else {
                console.error("Failed to stop flow", response.statusText);
                alert("Failed to stop flow");
            }
        } catch (err) {
            console.error("Error stopping flow", err);
            alert("Error stopping flow");
        }
    };
</script>

<div class="h-screen w-screen flex flex-row overflow-hidden">
    <NodePalette />

    <div
        class="flex-1 h-full relative"
        on:dragover={onDragOver}
        on:drop={(e) => {
            console.log("Dropped item to canvas", e);
            onDrop(e);
        }}
        on:keydown={onKeyDown}
        role="application"
        tabindex="0"
    >
        <div class="absolute top-4 right-4 z-10 flex gap-2 items-center">
            <ThemeToggle />
            {#if !isRunning}
                <button
                    class="bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded shadow flex items-center gap-2"
                    on:click={deployFlow}
                >
                    Start Flow
                </button>
            {:else}
                <button
                    class="bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-4 rounded shadow flex items-center gap-2"
                    on:click={stopFlow}
                >
                    Stop Flow
                </button>
            {/if}
        </div>
        <SvelteFlow
            bind:nodes
            bind:edges
            {nodeTypes}
            fitView
            class="bg-gray-50 dark:bg-gray-900"
            colorMode={$theme}
            deleteKeyCode={null}
            on:nodeclick={(e) => console.log("Node Clicked", e.detail)}
        >
            <Background gap={20} size={1} />
            <Controls />
            <MiniMap />
        </SvelteFlow>
    </div>

    {#if selectedNode}
        <PropertyPanel
            {selectedNode}
            onUpdate={updateNodeData}
            onDelete={(id) => {
                console.log("Deleting node", id);
                nodes = nodes.filter((n) => n.id !== id);
                selectedNode = null;
            }}
            onClose={() => {
                nodes = nodes.map((n) => ({ ...n, selected: false }));
                selectedNode = null;
            }}
        />
    {/if}
</div>
