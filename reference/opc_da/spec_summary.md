# OPC DA Custom Interface Standard (Legacy)

This document contains key technical details and COM interface mappings for OPC Data Access (DA).

## Core Interfaces

### IOPCServer
The main interface used to connect to an OPC Server and manage groups.

**Methods:**
- `AddGroup`: Creates a new logical group.
- `GetErrorString`: Translates error codes to human-readable strings.
- `RemoveGroup`: Deletes a group and its items.

### IOPCItemMgmt
Found on the Group object, used to manage individual items.

**Methods:**
- `AddItems`: Adds items to the group.
- `RemoveItems`: Removes items.
- `SetClientHandles`: Updates client-side identifiers.

### IOPCSyncIO
Synchronous data access interface.

**Methods:**
- `Read`: Reads values for specific items.
- `Write`: Writes values to specific items.

## Version Mappings (from OPC UA Part 8)

| OPC DA Error ID     | OPC UA Status Code | Description |
|---------------------|--------------------|-------------|
| OPC_E_BADRIGHTS     | Bad_NotReadable    | Access denied specifically for the read operation |
| OPC_E_UNKNOWNITEMID | Bad_NodeIdUnknown  | The item ID is not recognized by the server |
| E_INVALIDITEMID     | Bad_NodeIdInvalid  | The item ID format is invalid |

## Protocol Sequences
1. `CoCreateInstance` with server CLSID to get `IOPCServer`.
2. Call `IOPCServer::AddGroup` to create a group and get `IOPCItemMgmt`.
3. Call `IOPCItemMgmt::AddItems` with item definitions.
4. Query for `IOPCSyncIO` to perform data operations.
