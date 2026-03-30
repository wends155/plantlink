import { test, expect } from '@playwright/test';

// These tests run against the live Rust backend (http://localhost:3000)

test.describe('UI ↔ Backend Integration', () => {

    test('should return 200 from /health endpoint', async ({ request }) => {
        const response = await request.get('/health');
        expect(response.status()).toBe(200);
        const text = await response.text();
        expect(text).toBe('OK');
    });

    test('should serve UI assets', async ({ page }) => {
        const response = await page.goto('/');
        expect(response.status()).toBe(200);
        await expect(page).toHaveTitle(/Plantlink/i);
    });

    test('should open WebSocket connection', async ({ page }) => {
        const wsPromise = page.waitForEvent('websocket');
        await page.goto('/');
        const ws = await wsPromise;
        expect(ws.url()).toContain('/ws');
        expect(ws.isClosed()).toBe(false);
    });

    test('should deploy and run a flow end-to-end via REST + WS', async ({ request, page }) => {
        // 1. Prepare minimal flow: inject -> console
        const flowPayload = {
            nodes: [
                { id: "n1", type: "inject", data: { name: "inject", payload: "Hello Integration", interval: 1 } },
                { id: "n2", type: "console", data: { name: "console" } }
            ],
            edges: [
                { id: "e1", source: "n1", sourceHandle: "output_0", target: "n2", targetHandle: "input_0" }
            ]
        };

        // 2. Open page and intercept the WebSocket
        const wsPromise = page.waitForEvent('websocket');
        await page.goto('/');
        const ws = await wsPromise;

        // 3. Set up a listener for the expected runtime message from the WebSocket directly
        const framePromise = ws.waitForEvent('framereceived', frame => {
            const payload = frame.payload;
            // The payload might be a Buffer if it's binary, so convert to string
            const text = typeof payload === 'string' ? payload : payload.toString();
            return text.includes('Hello Integration');
        });

        // 4. Trigger REST API deploy
        console.log('Using token:', process.env.PLANTLINK_AUTH_TOKEN);
        const response = await request.post('/api/flow', {
            data: flowPayload,
            headers: {
                'Authorization': `Bearer ${process.env.PLANTLINK_AUTH_TOKEN}`
            }
        });
        expect(response.status()).toBe(200);

        // 5. Wait for the message to arrive over WS
        const frame = await framePromise;
        const payloadText = typeof frame.payload === 'string' ? frame.payload : frame.payload.toString();
        expect(payloadText).toContain('Hello Integration');

        // 6. Cleanup: Stop the flow
        const stopResponse = await request.post('/api/flow/stop', {
            headers: {
                'Authorization': `Bearer ${process.env.PLANTLINK_AUTH_TOKEN}`
            }
        });
        expect(stopResponse.status()).toBe(200);
        const stopData = await stopResponse.json();
        expect(stopData.tasks_aborted).toBeGreaterThanOrEqual(0);
    });

});
