# PlantLink UI Testing Guide

This document describes the testing strategy for the PlantLink UI to prevent regressions and ensure reliable functionality.

---

## Testing Stack

| Tool | Purpose | When to Use |
|------|---------|-------------|
| **Playwright** | End-to-end browser testing | Test user workflows (drag/drop, theme switching) |
| **Vitest** | Unit testing | Test individual functions and utilities |
| **@testing-library/svelte** | Component testing | Test Svelte component behavior |

---

## Setup

### Install Dependencies

```bash
cd ui
npm install -D @playwright/test
npx playwright install
```

### Configuration

Create `playwright.config.js`:

```javascript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
  },
});
```

---

## E2E Tests

### Critical User Flows

Create `e2e/flow-editor.spec.js`:

```javascript
import { test, expect } from '@playwright/test';

test.describe('Flow Editor', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should render flow editor layout', async ({ page }) => {
    await expect(page.locator('.flow-editor')).toBeVisible();
    await expect(page.locator('.node-palette')).toBeVisible();
    await expect(page.locator('.flow-canvas')).toBeVisible();
  });

  test('should display node categories', async ({ page }) => {
    await expect(page.locator('.palette-category')).toHaveCount(3);
    await expect(page.getByText('Common')).toBeVisible();
    await expect(page.getByText('Network')).toBeVisible();
    await expect(page.getByText('Function')).toBeVisible();
  });

  test('should allow drag and drop of nodes', async ({ page }) => {
    // Find Inject node in palette
    const injectNode = page.locator('.palette-item', { hasText: 'Inject' });
    const canvas = page.locator('.flow-canvas');

    // Get canvas center position
    const canvasBox = await canvas.boundingBox();
    const dropX = canvasBox.x + canvasBox.width / 2;
    const dropY = canvasBox.y + canvasBox.height / 2;

    // Drag from palette to canvas
    await injectNode.dragTo(canvas, {
      targetPosition: { x: canvasBox.width / 2, y: canvasBox.height / 2 }
    });

    // Verify node was created
    const svgNodes = page.locator('.svelte-flow__node');
    await expect(svgNodes).toHaveCount(1);
  });

  test('should toggle theme', async ({ page }) => {
    // Get initial theme class
    const htmlBefore = await page.locator('html').getAttribute('class');
    
    // Click theme toggle (adjust selector as needed)
    await page.locator('button[title*="theme"], .theme-toggle').first().click();
    
    // Wait for theme to change
    await page.waitForTimeout(100);
    
    // Verify theme class changed
    const htmlAfter = await page.locator('html').getAttribute('class');
    expect(htmlBefore).not.toBe(htmlAfter);
  });

  test('should show Start Flow button when not running', async ({ page }) => {
    await expect(page.locator('.btn-success', { hasText: 'Start Flow' })).toBeVisible();
  });
});
```

### Run Tests

```bash
# Run all tests
npm run test:e2e

# Run in UI mode (interactive)
npm run test:e2e:ui

# Run specific test
npx playwright test e2e/flow-editor.spec.js
```

---

## Component Tests (Optional)

### Setup Vitest

Install dependencies:

```bash
npm install -D vitest @testing-library/svelte @testing-library/jest-dom jsdom
```

Create `vitest.config.js`:

```javascript
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.js'],
    setupFiles: ['./vitest-setup.js'],
  },
});
```

Create `vitest-setup.js`:

```javascript
import '@testing-library/jest-dom/vitest';
```

### Example Component Test

`src/lib/NodePalette.test.js`:

```javascript
import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import NodePalette from './NodePalette.svelte';

describe('NodePalette', () => {
  it('renders category headers', () => {
    render(NodePalette);
    expect(screen.getByText('Common')).toBeInTheDocument();
    expect(screen.getByText('Network')).toBeInTheDocument();
    expect(screen.getByText('Function')).toBeInTheDocument();
  });

  it('renders palette items with draggable attribute', () => {
    render(NodePalette);
    const items = screen.getAllByRole('button');
    items.forEach(item => {
      expect(item).toHaveAttribute('draggable', 'true');
    });
  });
});
```

---

## NPM Scripts

Add to `package.json`:

```json
{
  "scripts": {
    "test": "vitest",
    "test:run": "vitest run",
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:report": "playwright show-report"
  }
}
```

---

## CI/CD Integration

### GitHub Actions Example

Create `.github/workflows/test.yml`:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 20
      
      - name: Install dependencies
        working-directory: ./ui
        run: npm ci
      
      - name: Install Playwright browsers
        working-directory: ./ui
        run: npx playwright install --with-deps
      
      - name: Run E2E tests
        working-directory: ./ui
        run: npm run test:e2e
      
      - uses: actions/upload-artifact@v3
        if: failure()
        with:
          name: playwright-report
          path: ui/playwright-report/
```

---

## Visual Regression Testing

### Snapshot Testing

Add to test file:

```javascript
test('flow editor visual snapshot', async ({ page }) => {
  await page.goto('/');
  
  // Wait for content to load
  await page.waitForSelector('.flow-editor');
  
  // Take screenshot
  await expect(page).toHaveScreenshot('flow-editor.png');
});
```

First run creates baseline. Subsequent runs compare against it.

Update snapshots:

```bash
npx playwright test --update-snapshots
```

---

## Best Practices

1. **Test Critical Paths First**
   - Drag & drop functionality
   - Theme switching
   - Flow deployment
   - Node property editing

2. **Use Data Test IDs**
   ```svelte
   <div data-testid="flow-canvas">
   ```
   
   ```javascript
   await page.getByTestId('flow-canvas');
   ```

3. **Avoid Brittle Selectors**
   - ❌ `.absolute.top-4.right-4`
   - ✅ `.flow-canvas`
   - ✅ `[data-testid="canvas"]`

4. **Test User Behavior, Not Implementation**
   - Focus on what users see and do
   - Don't test internal component state

5. **Keep Tests Fast**
   - Use `page.waitForSelector()` instead of `waitForTimeout()`
   - Parallelize independent tests

---

## Troubleshooting

### Tests Timing Out

Increase timeout in config:

```javascript
use: {
  actionTimeout: 10000,
}
```

### Drag & Drop Not Working

Use `dragTo` instead of manual mouse events:

```javascript
await element.dragTo(target);
```

### Flaky Tests

Add explicit waits:

```javascript
await page.waitForSelector('.svelte-flow__node');
await page.waitForLoadState('networkidle');
```

---

## See Also

- [Playwright Documentation](https://playwright.dev)
- [Testing Library](https://testing-library.com/docs/svelte-testing-library/intro)
- [Vitest Documentation](https://vitest.dev)
