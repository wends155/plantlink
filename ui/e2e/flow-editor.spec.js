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
        await expect(page.getByText('Function', { exact: true })).toBeVisible();
    });

    test('should render palette items as draggable', async ({ page }) => {
        const paletteItems = page.locator('.palette-item');
        await expect(paletteItems).toHaveCount(8);

        const firstItem = paletteItems.first();
        await expect(firstItem).toHaveAttribute('draggable', 'true');
    });

    test('should allow drag and drop of nodes', async ({ page }) => {
        // Find Inject node in palette
        const injectNode = page.locator('.palette-item', { hasText: /inject/i });
        const canvas = page.locator('.flow-canvas');

        // Ensure elements are visible
        await expect(injectNode).toBeVisible();
        await expect(canvas).toBeVisible();

        // Get canvas center position
        const canvasBox = await canvas.boundingBox();

        // Drag from palette to canvas center
        await injectNode.dragTo(canvas, {
            targetPosition: { x: canvasBox.width / 2, y: canvasBox.height / 2 }
        });

        // Wait a bit for the node to be created
        await page.waitForTimeout(500);

        // Verify node was created in the flow
        const svgNodes = page.locator('.svelte-flow__node');
        await expect(svgNodes).toHaveCount(1);
    });

    test('should toggle theme', async ({ page }) => {
        // Get initial theme class
        const htmlElement = page.locator('html');
        const initialClass = await htmlElement.getAttribute('class');

        // Find and click theme toggle button
        // Adjust selector based on actual ThemeToggle component
        const themeToggle = page.locator('button[aria-label="Toggle Dark Mode"]');
        await themeToggle.click();

        // Wait for theme to change
        await page.waitForTimeout(200);

        // Verify theme class changed
        const newClass = await htmlElement.getAttribute('class');
        expect(initialClass).not.toBe(newClass);
    });

    test('should show Start Flow button when not running', async ({ page }) => {
        await expect(page.locator('.btn-success', { hasText: 'Start Flow' })).toBeVisible();
    });

    test('should have proper CSS dimensions on flow canvas', async ({ page }) => {
        const canvas = page.locator('.flow-canvas');

        // Check computed styles
        const width = await canvas.evaluate(el => getComputedStyle(el).width);
        const height = await canvas.evaluate(el => getComputedStyle(el).height);

        // Width and height should not be 'auto' or '0px'
        expect(width).not.toBe('auto');
        expect(width).not.toBe('0px');
        expect(height).not.toBe('auto');
        expect(height).not.toBe('0px');
    });

    test('should NOT auto-zoom when dropping first node on empty canvas', async ({ page }) => {
        // Get the viewport transform before any nodes are placed
        const getZoomLevel = () =>
            page.locator('.svelte-flow__viewport').evaluate((el) => {
                const transform = getComputedStyle(el).transform;
                // CSS transform matrix: matrix(a, b, c, d, tx, ty) where a = scaleX
                if (!transform || transform === 'none') return 1;
                const match = transform.match(/matrix\(([^,]+)/);
                return match ? parseFloat(match[1]) : 1;
            });

        const zoomBefore = await getZoomLevel();

        // Drag an Inject node onto the canvas
        const injectNode = page.locator('.palette-item', { hasText: 'Inject' });
        const canvas = page.locator('.flow-canvas');
        await expect(injectNode).toBeVisible();
        await expect(canvas).toBeVisible();

        const canvasBox = await canvas.boundingBox();
        await injectNode.dragTo(canvas, {
            targetPosition: { x: canvasBox.width / 2, y: canvasBox.height / 2 },
        });

        // Wait for node to be rendered
        await page.waitForTimeout(500);
        await expect(page.locator('.svelte-flow__node')).toHaveCount(1);

        // Verify zoom level did NOT change (no auto-zoom)
        const zoomAfter = await getZoomLevel();
        expect(zoomAfter).toBeCloseTo(zoomBefore, 1);
    });

    test('should not have unexpected console errors on page load', async ({ page }) => {
        const errors = [];
        page.on('console', (msg) => {
            if (msg.type() === 'error') {
                const text = msg.text();
                // Exclude known expected errors (e.g., WebSocket — no backend running in E2E)
                if (!text.includes('WebSocket')) {
                    errors.push(text);
                }
            }
        });

        await page.goto('/');
        await page.waitForTimeout(1000); // Let page stabilize

        expect(errors).toEqual([]);
    });

    test('should log to console when node is dropped on canvas', async ({ page }) => {
        const logs = [];
        page.on('console', (msg) => {
            if (msg.type() === 'log') {
                logs.push(msg.text());
            }
        });

        await page.goto('/');

        const injectNode = page.locator('.palette-item', { hasText: /inject/i });
        const canvas = page.locator('.flow-canvas');
        const canvasBox = await canvas.boundingBox();

        await injectNode.dragTo(canvas, {
            targetPosition: { x: canvasBox.width / 2, y: canvasBox.height / 2 },
        });

        await page.waitForTimeout(500);

        const dropLog = logs.find((l) => l.includes('Dropped item to canvas'));
        expect(dropLog).toBeDefined();
    });
});
