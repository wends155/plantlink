# Rhai Scripting Guide

Rhai is a lightweight, embedded scripting language used in PlantLink's **Function Node** for custom data transformation.

---

## What is Rhai?

[Rhai](https://rhai.rs/) is a simple, fast, embedded scripting language for Rust with syntax similar to JavaScript/Rust. It's sandboxed and safe for user-provided scripts.

---

## Function Node Basics

The Function node receives messages, executes your Rhai script, and outputs the result.

```
[Input Message] → [Your Rhai Script] → [Output Message]
```

### Script Structure

Your code receives `msg` (the input message) and must return it:

```rhai
// Access and modify the payload
msg.payload = msg.payload + " modified";

// MUST return msg
return msg;
```

---

## MessagePayload Structure

The `msg` object has these fields:

| Field | Type | Description |
|-------|------|-------------|
| `msg.payload` | Dynamic | The main data (string, number, object) |
| `msg.topic` | String | Optional topic/subject name |
| `msg.timestamp` | Integer | Unix timestamp (optional) |

---

## Examples

### String Manipulation

```rhai
// Concatenate strings
msg.payload = msg.payload + " World";
return msg;
```

### Number Operations

```rhai
// Double the value
msg.payload = msg.payload * 2;
return msg;
```

### Conditionals

```rhai
if msg.payload > 100 {
    msg.payload = "HIGH";
} else {
    msg.payload = "LOW";
}
return msg;
```

### Working with Objects

```rhai
// If payload is a map/object
let data = msg.payload;
data.processed = true;
data.timestamp = timestamp();
msg.payload = data;
return msg;
```

### String Parsing

```rhai
// Parse and transform
let value = parse_int(msg.payload);
msg.payload = value * 10;
return msg;
```

---

## Built-in Functions

| Function | Description |
|----------|-------------|
| `print(x)` | Log to console |
| `parse_int(s)` | Parse string to integer |
| `parse_float(s)` | Parse string to float |
| `to_string(x)` | Convert to string |
| `len(s)` | String/array length |
| `type_of(x)` | Get type name |

---

## Error Handling

If your script has an error, the node will:
1. Show error status (red border)
2. Log error message to console
3. Not output any message

### Common Errors

| Error | Cause |
|-------|-------|
| Compilation Error | Syntax error in script |
| Runtime Error | Exception during execution |
| Return Type Mismatch | Didn't return `msg` object |

---

## Tips

1. **Always return `msg`** - Scripts must return the message object
2. **Keep it simple** - Complex logic should be in backend nodes
3. **Test incrementally** - Add logic step by step
4. **Check Console** - Errors are logged to browser console

---

## Complete Example

**Goal**: Transform sensor data from Celsius to Fahrenheit

**Input**: `{ "temp": 25 }`

```rhai
// Get temperature from payload
let temp_c = msg.payload.temp;

// Convert to Fahrenheit
let temp_f = (temp_c * 9 / 5) + 32;

// Update payload
msg.payload.temp_f = temp_f;
msg.payload.unit = "F";

return msg;
```

**Output**: `{ "temp": 25, "temp_f": 77, "unit": "F" }`
