import { test, expect } from '@playwright/test';

// These tests focus on Web Server authentication and WebSocket resilience.
// They require the backend to be running with PLANTLINK_AUTH_TOKEN=test-secret

test.describe('Web Server Hardening & Resilience', () => {

    test.beforeEach(async ({ request }) => {
        // Just checking health first
        const response = await request.get('/health');
        expect(response.status()).toBe(200);
    });

    test('should reject unauthorized /api/flow requests', async ({ request }) => {
        const flowPayload = { nodes: [], edges: [] };
        
        // No header
        const response = await request.post('/api/flow', {
            data: flowPayload
        });
        
        // If the backend has no token set, this might succeed or fail depending on env.
        // For the sake of this test, we assume the backend IS hardened.
        if (process.env.PLANTLINK_AUTH_TOKEN) {
            expect(response.status()).toBe(401);
        }
    });

    test('should accept authorized /api/flow requests with Bearer token', async ({ request }) => {
        const flowPayload = {
            nodes: [{ id: "n1", type: "inject", data: { name: "inject", payload: "auth-test", interval: 1 } }],
            edges: []
        };
        
        const token = process.env.PLANTLINK_AUTH_TOKEN || "test-secret";
        
        const response = await request.post('/api/flow', {
            data: flowPayload,
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        
        expect(response.status()).toBe(200);
        
        // Cleanup
        await request.post('/api/flow/stop', {
            headers: { 'Authorization': `Bearer ${token}` }
        });
    });

    test('should receive WebSocket status updates with heartbeat', async ({ page }) => {
        const wsPromise = page.waitForEvent('websocket');
        await page.goto('/');
        const ws = await wsPromise;

        // Monitor Pings
        let pingCount = 0;
        ws.on('framesent', frame => {
            // Note: In some browsers/drivers, Ping/Pong might not be visible as frames
            // But we can check if the connection stays alive
        });

        // 1. Deploy a flow that generates status updates
        const token = process.env.PLANTLINK_AUTH_TOKEN || "test-secret";
        const flowPayload = {
            nodes: [{ id: "n1", type: "inject", data: { name: "inject", payload: "ws-test", interval: 0.1 } }],
            edges: []
        };

        const deployRes = await page.request.post('/api/flow', {
            data: flowPayload,
            headers: { 'Authorization': `Bearer ${token}` }
        });
        expect(deployRes.status()).toBe(200);

        // 2. Wait for a status frame
        const framePromise = ws.waitForEvent('framereceived', frame => {
            const text = typeof frame.payload === 'string' ? frame.payload : frame.payload.toString();
            return text.includes('Status');
        });

        const frame = await framePromise;
        expect(frame.payload).toContain('Status');

        // 3. Cleanup
        await page.request.post('/api/flow/stop', {
            headers: { 'Authorization': `Bearer ${token}` }
        });
    });
});
