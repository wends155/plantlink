# PlantLink API Reference

REST and WebSocket API documentation.

---

## REST Endpoints

### Health Check

```
GET /health
```

**Response**: `OK`

---

### Deploy Flow

```
POST /api/flow
Content-Type: application/json
Authorization: Bearer <token>
```

**Request Body**:

```json
{
  "nodes": [
    {
      "id": "node_1",
      "type": "inject",
      "data": {
        "name": "inject",
        "payload": "Hello",
        "interval": 0
      }
    },
    {
      "id": "node_2",
      "type": "console",
      "data": {
        "name": "console"
      }
    }
  ],
  "edges": [
    {
      "source": "node_1",
      "sourceHandle": "output_0",
      "target": "node_2",
      "targetHandle": "input_0"
    }
  ]
}
```

**Response**: `200 OK` - "Flow Deployed"

---

### Stop Flow

```
POST /api/flow/stop
Authorization: Bearer <token>
```

**Response**: `200 OK` - "Flow Stopped"

---

## WebSocket

### Connection

```
ws://localhost:3001/ws
```

### Message Types (Server → Client)

#### Log Message

```json
{
  "type": "log",
  "message": "ConsoleNode [node_2]: Hello World"
}
```

#### Node Status

```json
{
  "type": "status",
  "data": {
    "node_id": "node_3",
    "state": "error",
    "message": "Compilation Error: ..."
  }
}
```

**State values**:
- `running` - Node is active
- `error` - Node has an error

---

## Flow Configuration Schema

### Node

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique node identifier |
| `type` | string | Node type (e.g., "inject", "nats-broker") |
| `data` | object | Node-specific configuration |

### Edge

| Field | Type | Description |
|-------|------|-------------|
| `source` | string | Source node ID |
| `sourceHandle` | string | Output port (e.g., "output_0") |
| `target` | string | Target node ID |
| `targetHandle` | string | Input port (e.g., "input_0") |

---

## Node Data Schemas

### inject

```json
{
  "name": "inject",
  "payload": "Hello World",
  "interval": 5
}
```

### console

```json
{
  "name": "console"
}
```

### nats-broker

```json
{
  "url": "nats://localhost:4222"
}
```

### nats-sub

```json
{
  "subject": "events.>"
}
```

**Ports:**
- Input 0 (Connection) - Receives broker connection ID
- Output 0 (Message) - Emits received messages

---

### nats-pub

```json
{
  "subject": "events.>"
}
```

**Ports:**
- Input 0 (Connection) - Receives broker connection ID
- Input 1 (Data) - Receives message payload to publish

**Data Flow (Port 1):**

| Incoming Payload Type | Published to NATS |
|-----------------------|-------------------|
| String (`"Hello"`) | `"Hello"` (UTF-8 bytes) |
| JSON (`{ "temp": 25 }`) | `{"temp":25}` (stringified) |
| Other types | Empty bytes |

**Subject Selection (Priority):**
1. Node config `subject` (if not empty)
2. `msg.topic` from incoming message
3. If both empty, no publish

**Example with Function Node:**
```
[Function] ─── msg.payload="sensor:42" ───▶ [NATS Pub Port 1]
                                                    │
                                                    ▼
                                           Publishes "sensor:42"
                                           to configured subject
```

### rhai-function

```json
{
  "name": "function",
  "code": "msg.payload = msg.payload + \" World\";\nreturn msg;"
}
```

### mqtt-in

```json
{
  "topic": "sensor/#"
}
```
