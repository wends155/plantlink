# COM to OPC UA Mapping Guidelines

Source Reference: OPC 10000-8 (Part 8: Data Access)

## Overview
The COM UA Wrapper maps legacy COM Data Access objects into OPC UA VariableTypes and NodeClasses.

## Data Retrieval Behavior
### Version 2.05a
- Uses `IOPCServer::AddGroup` and `IOPCItemMgmt::AddItems`.
- Data retrieval via `IOPCSyncIO::Read`.
- **Note**: Only "Read from Device" is supported; "maxAge" is ignored.

### Version 3.0
- Uses `IOPCItemIO::Read` for more efficient direct retrieval.
- Supports both "Read from Device" and "Cache".
- Utilizes the "maxAge" parameter correctly.

## Attribute Mappings
- **VQT (Value, Quality, Timestamp)** maps to the OPC UA `DataValue` structure:
  - `Value` -> `Value`
  - `Quality` -> `StatusCode`
  - `Timestamp` -> `SourceTimestamp` / `ServerTimestamp`
