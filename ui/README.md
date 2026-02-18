# PlantLink UI

Flow-based programming editor built with Svelte 5 and Vite.

## Quick Start

```bash
npm install
npm run dev
```

## Project Structure

```
src/
├── App.svelte              # Root component (WebSocket setup)
├── app.css                 # Global styles
├── styles/
│   ├── themes.css          # CSS variables (light/dark)
│   └── components.css      # Semantic CSS classes
└── lib/
    ├── components/
    │   ├── ui/             # Reusable primitives
    │   │   ├── Button.svelte
    │   │   ├── Input.svelte
    │   │   ├── FormGroup.svelte
    │   │   └── IconButton.svelte
    │   ├── properties/     # Node property editors
    │   │   ├── InjectProperties.svelte
    │   │   ├── MqttInProperties.svelte
    │   │   ├── NatsProperties.svelte
    │   │   └── RhaiProperties.svelte
    │   └── ThemeToggle.svelte
    ├── nodes/              # Flow node components
    ├── stores/             # Svelte stores
    └── nodeDefinitions.js  # Node type registry
```

## Available Scripts

| Script | Description |
|--------|-------------|
| `npm run dev` | Start dev server with HMR |
| `npm run build` | Production build |
| `npm run preview` | Preview production build |
| `npm run test:e2e` | Run Playwright tests |

## Styling Guide

Always use semantic classes from `components.css`:

```svelte
<!-- ✅ Good -->
<button class="btn btn-success">Start</button>
<input class="input" />

<!-- ❌ Avoid -->
<button class="px-4 py-2 bg-green-600 ...">Start</button>
```

### Available Classes

- **Buttons**: `.btn`, `.btn-success`, `.btn-danger`, `.btn-primary`, `.btn-secondary`
- **Button Sizes**: `.btn-sm`, `.btn-lg`
- **Forms**: `.input`, `.label`, `.form-group`
- **Panels**: `.panel-header`, `.panel-footer`, `.panel-content`
- **Layout**: `.flow-editor`, `.flow-canvas`, `.node-palette`, `.property-panel`

## Adding a New Node Type

1. Create node component in `lib/nodes/`
2. Add to `nodeDefinitions.js`
3. Create property editor in `lib/components/properties/`
4. Register in `properties/index.js`

See [Adding Nodes](../docs/ADDING_NODES.md) for details.

## Code Splitting

Build output is split into chunks for better caching:
- `codemirror-[hash].js` - CodeMirror editor (100KB+)
- `flow-[hash].js` - SvelteFlow library
- `icons-[hash].js` - Lucide icons

## See Also

- [Architecture](../architecture.md)
- [Theming](../docs/THEMING.md)
- [Testing](../docs/UI_TESTING.md)
