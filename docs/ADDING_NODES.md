# Adding New Nodes

Step-by-step guide for adding new nodes to PlantLink.

---

## Overview

Adding a new node requires **3 steps**:

1. Add definition to `nodeDefinitions.js`
2. Create Svelte component
3. Register in `registry.js`

---

## Step 1: Node Definition

Edit `ui/src/lib/nodeDefinitions.js`:

```javascript
"my-custom-node": {
    // Palette appearance
    displayName: "my custom",
    category: "Common",  // Common, Network, or Function
    icon: "Box",         // Lucide icon name
    color: "#3b82f6",
    iconColor: "#fff",
    
    // Port configuration
    inputs: [
        { 
            id: "input_0", 
            label: "Data", 
            acceptTypes: ["message"], 
            maxConnections: Infinity 
        }
    ],
    outputs: [
        { id: "output_0", label: "Result", type: "message" }
    ],
    
    // Default data for new instances
    defaultData: { setting: "default" },
    
    // Properties panel fields
    properties: [
        { key: "setting", label: "Setting", type: "text", placeholder: "Enter value" }
    ]
}
```

### Port Types

| Type | Description |
|------|-------------|
| `connection` | Broker/driver connections (NATS, MQTT) |
| `message` | Data payloads (string, JSON, objects) |

---

## Step 2: Svelte Component

Create `ui/src/lib/nodes/MyCustomNode.svelte`:

```svelte
<script>
    import BaseNode from "./BaseNode.svelte";
    import { Box } from "lucide-svelte";

    export let data;
    export let selected;
    export let id;
</script>

<BaseNode
    {id}
    nodeType="my-custom-node"
    label={data.label || data.name || "my custom"}
    {selected}
>
    <Box slot="icon" size={16} color="#fff" />
</BaseNode>
```

**Key points:**
- Pass `{id}` to enable automatic error status display
- Pass `nodeType` to auto-fetch ports/colors from definition
- Use `data.label` or fallback for display

---

## Step 3: Register Component

Edit `ui/src/lib/nodes/registry.js`:

```javascript
import MyCustomNode from "./MyCustomNode.svelte";

const registry = {
    // ... existing nodes
    "my-custom-node": MyCustomNode,
};
```

---

## Backend Implementation (Optional)

For runtime behavior, add to `plantlink-runtime/src/nodes/`:

```rust
pub struct MyCustomNode {
    setting: String,
}

impl MyCustomNode {
    pub fn new(config: &NodeConfig) -> Self {
        let setting = config.data.get("setting")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        Self { setting }
    }
}

#[async_trait]
impl NodeBehavior for MyCustomNode {
    async fn on_input(&mut self, _port: usize, msg: MessagePayload, ctx: NodeContext) -> Result<()> {
        // Process message
        let output = MessagePayload {
            payload: msg.payload,
            ..Default::default()
        };
        ctx.send_output(output).await;
        Ok(())
    }
}
```

Register in `nodes/mod.rs`:

```rust
pub fn register_defaults(registry: &mut NodeRegistry) {
    // ... existing registrations
    registry.register("my-custom-node", |cfg| Box::new(MyCustomNode::new(cfg)));
}
```

---

## Error Handling

Use `NodeContext` helpers to emit error status to the UI:

```rust
// In on_input or start:
if some_condition_fails {
    ctx.emit_error("Description of what went wrong");
    return Err(...);
}

// On success:
ctx.emit_running("Connected successfully");
```

The UI automatically:
- Shows red background on nodes in error state
- Displays error message in PropertyPanel

---

## Checklist

- [ ] Definition in `nodeDefinitions.js`
- [ ] Svelte component created
- [ ] Registered in `registry.js`
- [ ] (Optional) Backend behavior in `nodes/`
- [ ] (Optional) Properties in `PropertyPanel.svelte`
