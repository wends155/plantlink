/**
 * Unified Node Definitions Registry
 * Single source of truth for all node metadata.
 * 
 * To add a new node:
 * 1. Add definition here
 * 2. Create the Svelte component in nodes/ folder
 * 3. Register in nodes/registry.js
 */

// Icon mapping for dynamic rendering
export const iconMap = {
    "Database": "Database",
    "ArrowDownToLine": "ArrowDownToLine",
    "ArrowUpFromLine": "ArrowUpFromLine",
    "Play": "Play",
    "Terminal": "Terminal",
    "Wifi": "Wifi",
    "Cpu": "Cpu",
    "FileJson": "FileJson",
};

export const nodeDefinitions = {
    "nats-broker": {
        displayName: "nats broker",
        category: "Network",
        icon: "Database",
        color: "#e5e7eb",
        iconColor: "#444",
        inputs: [],
        outputs: [
            { id: "output_0", label: "Connection", type: "connection" }
        ],
        defaultData: { url: "nats://localhost:4222" },
        properties: [
            { key: "url", label: "URL", type: "text", placeholder: "nats://localhost:4222" }
        ]
    },
    "nats-sub": {
        displayName: "nats sub",
        category: "Network",
        icon: "ArrowDownToLine",
        color: "#2563eb",
        iconColor: "#fff",
        inputs: [
            { id: "input_0", label: "Connection", acceptTypes: ["connection"], maxConnections: 1 }
        ],
        outputs: [
            { id: "output_0", label: "Message", type: "message" }
        ],
        defaultData: { subject: "" },
        properties: [
            { key: "subject", label: "Subject", type: "text", placeholder: "events.>" }
        ]
    },
    "nats-pub": {
        displayName: "nats pub",
        category: "Network",
        icon: "ArrowUpFromLine",
        color: "#2563eb",
        iconColor: "#fff",
        inputs: [
            { id: "input_0", label: "Connection", acceptTypes: ["connection"], maxConnections: 1 },
            { id: "input_1", label: "Data", acceptTypes: ["message"], maxConnections: Infinity }
        ],
        outputs: [],
        defaultData: { subject: "" },
        properties: [
            { key: "subject", label: "Subject", type: "text", placeholder: "events.>" }
        ]
    },
    "inject": {
        displayName: "inject",
        category: "Common",
        icon: "Play",
        color: "#a6bbcf",
        iconColor: "#fff",
        inputs: [],
        outputs: [
            { id: "output_0", label: "Message", type: "message" }
        ],
        defaultData: { name: "inject", payload: "", interval: 0 },
        properties: [
            { key: "payload", label: "Payload (String)", type: "text", placeholder: "Hello World" },
            { key: "interval", label: "Interval (Seconds)", type: "number", placeholder: "0 (disabled)" }
        ]
    },
    "console": {
        displayName: "console",
        category: "Common",
        icon: "Terminal",
        color: "#87a980",
        iconColor: "#fff",
        inputs: [
            { id: "input_0", label: "Message", acceptTypes: ["message"], maxConnections: Infinity }
        ],
        outputs: [],
        defaultData: { name: "console" },
        properties: []
    },
    "mqtt-in": {
        displayName: "mqtt in",
        category: "Network",
        icon: "Wifi",
        color: "#a6bbcf",
        iconColor: "#fff",
        inputs: [],
        outputs: [
            { id: "output_0", label: "Message", type: "message" }
        ],
        defaultData: { topic: "topic/#" },
        properties: [
            { key: "topic", label: "Topic", type: "text", placeholder: "sensor/#" }
        ]
    },
    "modbus-read": {
        displayName: "modbus read",
        category: "Network",
        icon: "Cpu",
        color: "#e04e5d",
        iconColor: "#fff",
        inputs: [],
        outputs: [
            { id: "output_0", label: "Data", type: "message" }
        ],
        defaultData: { name: "Read Holding" },
        properties: []
    },
    "rhai-function": {
        displayName: "function",
        category: "Function",
        icon: "FileJson",
        color: "#fdd0a2",
        iconColor: "#444",
        inputs: [
            { id: "input_0", label: "Message", acceptTypes: ["message"], maxConnections: Infinity }
        ],
        outputs: [
            { id: "output_0", label: "Message", type: "message" }
        ],
        defaultData: { name: "function", code: "" },
        properties: [
            { key: "code", label: "Rhai Script", type: "code" }
        ]
    }
};

/**
 * Get node definition by type
 */
export function getNodeDefinition(nodeType) {
    return nodeDefinitions[nodeType] || null;
}

/**
 * Get all node types
 */
export function getAllNodeTypes() {
    return Object.keys(nodeDefinitions);
}

/**
 * Get nodes grouped by category
 */
export function getNodesByCategory() {
    const categories = {};
    for (const [type, def] of Object.entries(nodeDefinitions)) {
        const cat = def.category || "Other";
        if (!categories[cat]) categories[cat] = {};
        categories[cat][type] = def;
    }
    return categories;
}

/**
 * Get input labels for a node type
 */
export function getInputLabels(nodeType) {
    const def = nodeDefinitions[nodeType];
    if (!def) return [];
    return def.inputs.map(p => p.label);
}

/**
 * Get output labels for a node type
 */
export function getOutputLabels(nodeType) {
    const def = nodeDefinitions[nodeType];
    if (!def) return [];
    return def.outputs.map(p => p.label);
}

/**
 * Get port schema for validation (backwards compatible)
 */
export function getPortSchema(nodeType) {
    const def = nodeDefinitions[nodeType];
    if (!def) return { inputs: [], outputs: [] };
    return { inputs: def.inputs, outputs: def.outputs };
}
