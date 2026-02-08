# PlantLink Architecture

Comprehensive system architecture documentation for PlantLink.

## System Overview

```mermaid
graph TB
    subgraph UI ["Frontend (Svelte)"]
        FE[Flow Editor]
        NP[Node Palette]
        PP[Property Panel]
    end
    
    subgraph Backend ["Backend (Rust)"]
        CLI[plantlink-cli]
        WEB[plantlink-web]
        RT[plantlink-runtime]
        CORE[plantlink-core]
    end
    
    subgraph Protocols
        MQTT[MQTT Broker]
        NATS[NATS Server]
        MB[Modbus Device]
    end
    
    FE <-->|WebSocket| WEB
    CLI --> WEB --> RT --> CORE
    RT <--> MQTT
    RT <--> NATS
    RT <--> MB
```

---

## Crate Breakdown

| Crate | Purpose |
|-------|---------|
| **plantlink-core** | Shared types: `DataValue`, `MessagePayload`, protocol drivers |
| **plantlink-runtime** | Flow execution engine, node behaviors, message routing |
| **plantlink-web** | Axum HTTP server, WebSocket handler, static file serving |
| **plantlink-cli** | CLI entry point, argument parsing, server startup |

---

## Data Flow

```mermaid
sequenceDiagram
    participant UI as Flow Editor
    participant WEB as Web Server
    participant RT as Runtime
    participant Node as Node Actor
    
    UI->>WEB: POST /api/flow
    WEB->>RT: update_flow(config)
    RT->>Node: spawn actor task
    Node-->>RT: MPSC messages
    RT-->>WEB: broadcast status
    WEB-->>UI: WebSocket update
```

---

## Key Components

### Runtime Engine

- Manages node lifecycle (create, start, stop)
- Routes messages between nodes via MPSC channels
- Uses actor pattern: each node runs in its own Tokio task

### Node Behavior Trait

```rust
#[async_trait]
pub trait NodeBehavior: Send + Sync {
    async fn start(&mut self, ctx: NodeContext) -> Result<()> { Ok(()) }
    async fn on_input(&mut self, port: usize, msg: MessagePayload, ctx: NodeContext) -> Result<()> { Ok(()) }
    async fn stop(&mut self) -> Result<()> { Ok(()) }
}
```

### Shared Resources

Connections (NATS, MQTT) stored in shared map:
```rust
Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>
```

---

## UI Architecture

| Component | Purpose |
|-----------|---------|
| `nodeDefinitions.js` | Single source of truth for node metadata |
| `NodePalette.svelte` | Auto-generated from definitions |
| `BaseNode.svelte` | Generic node component with auto-fetch |
| `InnerFlowEditor.svelte` | SvelteFlow canvas with connection validation |
| `PropertyPanel.svelte` | Dynamic property editing |

---

## Port Type System

```javascript
// Port definition in nodeDefinitions.js
{
    label: "Connection",
    type: "connection",        // Output type
    acceptTypes: ["connection"], // Accepted input types
    maxConnections: 1           // Single connection limit
}
```

**Types:**
- `connection` - Broker/driver connection IDs
- `message` - Data payloads (string, JSON, etc.)
