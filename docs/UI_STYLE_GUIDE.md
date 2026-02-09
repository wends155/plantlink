# PlantLink UI Style Guide

This guide documents the design system, theming architecture, and component usage for PlantLink.

---

## 1. Theme System

The UI uses CSS variables defined in [themes.css](ui/src/styles/themes.css). Colors are semantic, not absolute.

### Core Variables

| Variable | Description | Light Default | Dark Default |
|----------|-------------|---------------|--------------|
| `--color-bg-primary` | Main background (panels, cards) | `#fafafa` | `#111827` |
| `--color-bg-secondary` | Secondary background (headers, inputs) | `#f3f3f3` | `#1f2937` |
| `--color-text-primary` | Main text color | `#333333` | `#f3f4f6` |
| `--color-border` | Standard border | `#cccccc` | `#4b5563` |
| `--color-btn-primary` | Primary action color | `#2563eb` | `#2563eb` |

### Node Variables

| Variable | Description |
|----------|-------------|
| `--color-node-bg` | Node body background |
| `--color-node-border` | Node border color |
| `--color-port` | Connection handle color |

### State Variables

| Variable | Usage |
|----------|-------|
| `--color-state-running` | Green (Active/Good) |
| `--color-state-error` | Red (Failed/Bad) |
| `--color-state-stopped` | Gray (Inactive/Paused) |
| `--color-state-*-bg` | Backgroundtint for banners/nodes |

---

## 2. Components

Defined in [components.css](ui/src/styles/components.css).

### Buttons

Use the `.btn` class with a variant modifier.

```html
<button class="btn btn-primary">Save</button>
<button class="btn btn-secondary">Discard</button>
<button class="btn btn-danger">Delete</button>
```

- **Secondary Buttons:** Use an outline style for high contrast.
- **Focus States:** All buttons have accessible `:focus-visible` styles.

### Nodes

Nodes extend `.node-base` and apply state modifiers.

```html
<div class="node-base node--running">...</div>
```

- **Selection:** `.node-base--selected` (Overrides border color)
- **Cascade Order:** Selection style must come **after** state modifiers in CSS.

---

## 3. Extensibility

### Adding a New Theme

1. Add a new class block in `themes.css`.
2. Override semantic variables.

```css
.theme-ocean {
    --color-bg-primary: #e0f2fe;
    --color-btn-primary: #0284c7;
}
```

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

## 4. Best Practices

1. **Use Semantic Classes:** Avoid inline Tailwind for theme-dependent colors (e.g., don't use `bg-gray-100`, use `bg-[var(--color-bg-secondary)]` or configured utility).
2. **Check Contrast:** Ensure text/background contrast is > 4.5:1.
3. **Avoid Duplicates:** Search `components.css` before adding new classes.
4. **Cascade Matters:** Modifiers (selected, error) must come after base classes.

---

## 5. Tailwind Configuration

Tailwind is used for layout and spacing utilities (`flex`, `p-4`, `gap-2`).
For colors, reliance is on CSS variables to support runtime theme switching.
