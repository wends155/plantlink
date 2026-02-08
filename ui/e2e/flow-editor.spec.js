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

    test('should render palette items as draggable', async ({ page }) => {
        const paletteItems = page.locator('.palette-item');
        await expect(paletteItems).toHaveCount(15); // Adjust based on actual node count

        const firstItem = paletteItems.first();
        await expect(firstItem).toHaveAttribute('draggable', 'true');
    });

    test('should allow drag and drop of nodes', async ({ page }) => {
        // Find Inject node in palette
        const injectNode = page.locator('.palette-item', { hasText: 'Inject' });
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
        const themeToggle = page.locator('button').filter({ hasText: /theme|light|dark/i }).first();
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
});
