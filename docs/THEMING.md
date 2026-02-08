# UI Theming System

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

## Available Themes

| Theme | Class | Description |
|-------|-------|-------------|
| **Light** | `:root` | Default light theme |
| **Dark** | `.dark` | Dark mode |
| **System** | Auto | Matches OS preference |

Future themes can be added by creating new CSS classes (see [Adding Themes](#adding-new-themes)).

---

## CSS Custom Properties

### Color Variables

All colors are defined as CSS custom properties:

```css
:root {
  /* Backgrounds */
  --color-bg-primary: #fafafa;
  --color-bg-secondary: #f3f3f3;
  --color-text-primary: #333333;
  --color-btn-success: #16a34a;
  /* ... */
}

.dark {
  --color-bg-primary: #111827;
  --color-bg-secondary: #1f2937;
  --color-text-primary: #f3f4f6;
  /* ... */
}
```

### Available Variables

| Variable | Light | Dark | Usage |
|----------|-------|------|-------|
| `--color-bg-primary` | `#fafafa` | `#111827` | Main backgrounds |
| `--color-bg-secondary` | `#f3f3f3` | `#1f2937` | Secondary panels |
| `--color-text-primary` | `#333333` | `#f3f4f6` | Primary text |
| `--color-border` | `#cccccc` | `#4b5563` | Borders |
| `--color-btn-success` | `#16a34a` | (same) | Success buttons |
| `--color-btn-danger` | `#dc2626` | (same) | Danger buttons |

Full list: [`ui/src/styles/themes.css`](file:///c:/Users/WSALIGAN/code/plantlink/ui/src/styles/themes.css)

---

## Semantic Classes

Components use semantic class names instead of Tailwind utilities:

### Layout Classes

```css
.flow-editor        /* Main editor container */
.flow-canvas        /* Flow canvas area */
.node-palette       /* Node palette sidebar */
.property-panel     /* Property panel sidebar */
```

### Component Classes

```css
.palette-category   /* Palette category header */
.palette-item       /* Draggable palette item */
.btn                /* Base button */
.btn-success        /* Success button (green) */
.btn-danger         /* Danger button (red) */
.btn-primary        /* Primary button (blue) */
.input              /* Form input */
```

Full list: [`ui/src/styles/components.css`](file:///c:/Users/WSALIGAN/code/plantlink/ui/src/styles/components.css)

---

## Using Themes in Components

### Method 1: Semantic Classes (Recommended)

```svelte
<aside class="node-palette">
  <div class="palette-category">Common Nodes</div>
  <button class="btn btn-success">Start</button>
</aside>
```

### Method 2: CSS Variables (for custom styles)

```svelte
<div style="background-color: var(--color-bg-primary);">
  Custom component
</div>
```

### Method 3: Hybrid (Tailwind + CSS vars)

```svelte
<div class="p-4 rounded" style="border-color: var(--color-border);">
  Mixed approach
</div>
```

---

## Theme Switching

### Programmatic

```javascript
import { theme } from '$lib/stores/theme';

// Set theme
theme.set('dark');      // Dark mode
theme.set('light');     // Light mode
theme.set('system');    // Follow OS preference
```

### In Components

```svelte
<script>
  import { theme } from '$lib/stores/theme';
  
  function toggleTheme() {
    theme.update(t => t === 'dark' ? 'light' : 'dark');
  }
</script>

<button on:click={toggleTheme}>Toggle Theme</button>
```

### Theme Store API

```javascript
import { theme, availableThemes } from '$lib/stores/theme';

// Get current theme
$theme // 'light' | 'dark' | 'system' | 'theme-*'

// Available themes
availableThemes = [
  { id: 'light', name: 'Light' },
  { id: 'dark', name: 'Dark' },
  { id: 'system', name: 'System' }
];
```

---

## Adding New Themes

### Step 1: Define CSS Variables

Add to `ui/src/styles/themes.css`:

```css
.theme-nord {
  --color-bg-primary: #2e3440;
  --color-bg-secondary: #3b4252;
  --color-text-primary: #eceff4;
  --color-btn-success: #a3be8c;
  --color-btn-danger: #bf616a;
  /* ... define all variables */
}
```

### Step 2: Register Theme

Add to `ui/src/lib/stores/theme.js`:

```javascript
export const availableThemes = [
  { id: 'light', name: 'Light' },
  { id: 'dark', name: 'Dark' },
  { id: 'system', name: 'System' },
  { id: 'theme-nord', name: 'Nord' }, // Add this
];
```

### Step 3: Use It

```javascript
theme.set('theme-nord');
```

**That's it!** No component changes needed.

---

## Build Pipeline

The theming system integrates seamlessly with the build pipeline:

- **Minification**: Vite minifies CSS (themes.css + components.css → single file)
- **Gzip**: `vite-plugin-compression` creates `.gz` files
- **Purging**: Tailwind removes unused utilities (semantic classes are preserved)

**Build Output:**
- CSS: ~33 KB raw → ~6 KB gzipped
- No runtime JavaScript overhead (pure CSS)

---

## Design Principles

1. **Single Source of Truth**: All colors defined once in `themes.css`
2. **Semantic Over Utility**: Use `.node-palette` instead of `w-[200px] bg-[#fafafa] ...`
3. **Runtime Switching**: Themes switch via class changes (no rebuild)
4. **Zero-Config Extensibility**: Add themes without touching components

---

## Migration Guide

### From Tailwind Utilities

**Before:**
```svelte
<div class="w-64 bg-white dark:bg-gray-900 border-l border-gray-300 dark:border-gray-700">
```

**After:**
```svelte
<div class="property-panel">
```

### From Hardcoded Colors

**Before:**
```svelte
<div style="background-color: #fafafa; color: #333;">
```

**After:**
```svelte
<div style="background-color: var(--color-bg-primary); color: var(--color-text-primary);">
```

### Migration Safety Rules

> [!IMPORTANT]
> Follow these rules when migrating Tailwind utilities to semantic classes to avoid regressions:

1. **Always include explicit dimensions** for layout containers:
   ```css
   /* ❌ BAD */
   .flow-canvas {
       flex: 1;
       height: 100%;
   }
   
   /* ✅ GOOD */
   .flow-canvas {
       flex: 1;
       width: 100%;   /* Explicit width required */
       height: 100%;  /* Explicit height required */
   }
   ```

2. **Test drag/drop functionality** after any canvas-related changes:
   - Drag nodes from palette to canvas
   - Verify drop zones are capturing events
   - Check browser console for errors

3. **Verify flexbox children** have proper width/height constraints:
   - Use `width: 100%` for components that need to fill their container
   - Test at different viewport sizes

4. **Check browser DevTools** for CSS inheritance issues:
   - Inspect element to verify semantic classes are applied
   - Check for conflicting Tailwind utilities
   - Verify CSS custom properties are resolving correctly

5. **Document critical CSS requirements** with comments:
   ```css
   /**
    * CRITICAL: SvelteFlow requires explicit width/height.
    * Do not remove width: 100% or height: 100%.
    */
   .flow-canvas {
       /* ... */
   }
   ```

---

## Examples

### Example 1: Custom Button

```svelte
<button class="btn" style="background-color: var(--color-btn-success);">
  Custom Green Button
</button>
```

### Example 2: Theme-Aware Component

```svelte
<script>
  import { theme } from '$lib/stores/theme';
</script>

<div class="p-4" style="
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
">
  Current theme: {$theme}
</div>
```

### Example 3: Conditional Styling

```svelte
<script>
  import { theme } from '$lib/stores/theme';
  $: isDark = $theme === 'dark';
</script>

<div class="palette-item">
  <span class:opacity-50={isDark}>
    Conditionally styled
  </span>
</div>
```

---

## Troubleshooting

### Theme Not Applying

1. Check that `themes.css` is imported in `app.css`
2. Verify the theme class is on `<html>` (not `<body>`)
3. Ensure CSS custom property names match

### Build Issues

```bash
# Clear build cache
cd ui
rm -rf node_modules/.vite
npm run build
```

### Dark Mode Not Working

1. Check browser DevTools → Elements → `<html class="dark">`
2. Verify theme store subscription in `theme.js`
3. Check system preference: `window.matchMedia('(prefers-color-scheme: dark)')`

---

## File Structure

```
ui/src/
├── styles/
│   ├── themes.css          # CSS custom properties
│   └── components.css      # Semantic classes
├── lib/
│   └── stores/
│       └── theme.js        # Theme management
└── app.css                 # Entry point (imports themes)
```

---

## See Also

- [Architecture](./ARCHITECTURE.md) - Overall system architecture
- [Adding Nodes](./ADDING_NODES.md) - Creating new node types
- [UI README](../ui/README.md) - Frontend development guide
