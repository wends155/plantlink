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

## Node State & Error Handling
 
Nodes have 3 possible states:
- **stopped**: Default state (gray). Code is not running.
- **running**: Successfully started (green). Code is active.
- **error**: Failed to start or runtime error (red). 
 
Use `NodeContext` helpers to emit status updates:
 
```rust
// 1. On successful start:
ctx.emit_running("Connected to broker");

// 2. On error (start or runtime):
if let Err(e) = do_something() {
    ctx.emit_error(&format!("Operation failed: {}", e));
    return Err(e);
}

// 3. Stopped state is handled automatically by the runtime when flow stops.
```
 
The UI automatically:
- Shows **Green** border for `running`
- Shows **Red** background for `error`
- Shows **Gray** (dimmed) for `stopped`
- Displays status message in PropertyPanel banner

---

## Checklist

- [ ] Definition in `nodeDefinitions.js`
- [ ] Svelte component created
- [ ] Registered in `registry.js`
- [ ] (Optional) Backend behavior in `nodes/`
- [ ] (Optional) Properties in `PropertyPanel.svelte`
