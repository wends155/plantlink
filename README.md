# PlantLink

**PlantLink** is a flow-based programming environment designed for IoT and Industrial Automation, built with performance and reliability in mind using **Rust**.

![PlantLink Screenshot](https://via.placeholder.com/800x400?text=PlantLink+Flow+Editor)

## Features

-   **High-Performance Runtime**: Powered by Rust and Tokio for efficient, asynchronous message processing.
-   **Modern Flow Editor**: A beautiful, dark-mode enabled visual editor built with Svelte Flow.
-   **Scripting Support**: Integrated **Rhai** scripting engine for custom logic with a familiar syntax.
-   **Industrial Protocols**:
    -   **MQTT**: Publish and Subscribe support.
    -   **Modbus TCP**: Read Holding Registers.
    -   **NATS**: High-performance messaging integration.
-   **Developer Experience**:
    -   Hot-reload development workflow.
    -   Single-binary release capability.

## Getting Started

### Prerequisites
-   **Rust**: Stable toolchain installed (latest).
-   **Node.js**: v20+ (for UI build).

### Development
To run the full stack (UI + Backend) in development mode:
```bash
make run
```
This will build the UI assets and launch the `plantlink-cli`.

### Building for Release
To create a single, optimized binary with embedded assets:
```bash
make build-release
```
The binary will be located at `target/release/plantlink-cli`.

### Cleanup
To clean up build artifacts and logs:
```bash
make clean
```

## Architecture
-   **plantlink-core**: Shared data types and protocol definitions.
-   **plantlink-runtime**: The engine that executes the flow logic.
-   **plantlink-web**: Axum-based web server and WebSocket handler.
-   **plantlink-cli**: The command-line entry point.
-   **ui**: SvelteKit/Vite frontend application.

## License
MIT
