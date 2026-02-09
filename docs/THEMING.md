# UI Theming System & Style Guide

PlantLink's UI uses a modern theming system built on **CSS custom properties** and **semantic classes** for consistent styling, easy theme switching, and extensibility.

---

## Architecture

The theming system consists of three layers:

1. **Theme Variables** (`ui/src/styles/themes.css`) - CSS custom properties defining colors
2. **Semantic Classes** (`ui/src/styles/components.css`) - Reusable component styles
3. **Theme Store** (`ui/src/lib/stores/theme.js`) - Runtime theme management

```
themes.css (CSS vars) → components.css (semantic classes) → Svelte components
                     ↑
             theme.js (store) applies .dark class
```

---

## 1. CSS Custom Properties

All colors are defined as CSS custom properties in `ui/src/styles/themes.css`.

### Core Variables

| Variable | Light Default | Dark Default | Usage |
|----------|---------------|--------------|-------|
| `--color-bg-primary` | `#fafafa` | `#111827` | Main background (panels, cards) |
| `--color-bg-secondary` | `#f3f3f3` | `#1f2937` | Secondary background (headers, inputs) |
| `--color-text-primary` | `#333333` | `#f3f4f6` | Main text color |
| `--color-border` | `#cccccc` | `#4b5563` | Standard border |
| `--color-btn-primary` | `#2563eb` | `#2563eb` | Primary action color |

### Secondary Buttons (High Contrast)

To ensure visibility on light gray panels, secondary buttons use specific high-contrast variables:

| Variable | Light | Dark | Usage |
|----------|-------|------|-------|
| `--color-btn-secondary-bg` | `#cbd5e1` (Slate 300) | `#374151` (Gray 700) | Button background |
| `--color-btn-secondary-text` | `#0f172a` (Slate 900) | `#f3f4f6` (Gray 100) | Button text |

### Node Variables

| Variable | Description |
|----------|-------------|
| `--color-node-bg` | Node body background |
| `--color-node-border` | Node border color |
| `--color-port` | Connection handle color |

### State Variables

| Variable | Description |
|----------|-------------|
| `--color-state-running` | Green (Active/Good) |
| `--color-state-error` | Red (Failed/Bad) |
| `--color-state-stopped` | Gray (Inactive/Paused) |

---

## 2. Component Usage

Components should use **semantic class names** defined in `ui/src/styles/components.css` instead of raw Tailwind utilities for coloring.

### Buttons

Use the `.btn` class with a variant modifier.

```html
<button class="btn btn-primary">Save</button>
<button class="btn btn-secondary">Discard (non-destructive)</button>
<button class="btn btn-danger">Delete (destructive)</button>
```

- **Secondary Buttons:** Use the dedicated high-contrast style.
- **Focus States:** All buttons include accessible `:focus-visible` styles automatically.

### Nodes

Nodes extend `.node-base` and apply state modifiers.

```html
<div class="node-base node--running">...</div>
```

- **Selection:** `.node-base--selected` (Overrides border color)
- **Cascade Order:** The selection class must be applied **after** state modifiers in CSS to take precedence.

---

## 3. Best Practices

> [!IMPORTANT]
> Follow these rules to ensure consistency and accessibility.

1. **Use Semantic Classes:** Avoid inline Tailwind for theme-dependent colors.
   - ❌ `class="bg-gray-100"`
   - ✅ `class="bg-[var(--color-bg-secondary)]"` OR use `.panel-header`

2. **Check Contrast:** Ensure text/background contrast is > 4.5:1 (WCAG AA). 
   - This is why secondary buttons use Slate 300 instead of lighter grays.

3. **Avoid Duplicates:** Before adding a new class to `components.css`, search to see if it already exists to avoid specificity wars.

4. **Responsive Layouts:**
   - Always provide explicit `width: 100%; height: 100%;` for SvelteFlow containers.

---

## 4. Extensibility

### Adding a New Theme

1. Add a new class block in `themes.css`.
2. Override semantic variables.

```css
.theme-ocean {
    --color-bg-primary: #e0f2fe;
    --color-btn-primary: #0284c7;
}
```

3. Register in `ui/src/lib/stores/theme.js`.

### Adding a New Button Variant

1. Define variables in `themes.css`.
2. Create class in `components.css`.

```css
/* themes.css */
--color-btn-warning: #eab308;

/* components.css */
.btn-warning {
    background-color: var(--color-btn-warning);
}
```

---

## 5. Migration Guide

### From Hardcoded Colors

**Before:**
```svelte
<div style="background-color: #fafafa; color: #333;">
```

**After:**
```svelte
<div style="background-color: var(--color-bg-primary); color: var(--color-text-primary);">
```

---

## See Also

- [Architecture](./ARCHITECTURE.md) - Overall system architecture
- [Adding Nodes](./ADDING_NODES.md) - Creating new node types
- [UI README](../ui/README.md) - Frontend development guide
