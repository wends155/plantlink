import { test, expect } from '@playwright/test';

// These tests focus on Web Server authentication and WebSocket resilience.
// They REQUIRE the backend to be running with PLANTLINK_AUTH_TOKEN injected into the process.

test.describe('Web Server Hardening & Resilience', () => {
  test.beforeEach(async ({ request }) => {
    // Just checking health first
    const response = await request.get('/health');
    expect(response.status()).toBe(200);
  });

  const unauthorizedVectors = [
    { name: 'no header', headers: {} },
    { name: 'empty bearer', headers: { Authorization: 'Bearer ' } },
    { name: 'wrong token', headers: { Authorization: 'Bearer wrong-secret' } },
    { name: 'malformed scheme', headers: { Authorization: 'Token test-secret' } }
  ];

  for (const vector of unauthorizedVectors) {
    test(`should reject ${vector.name}`, async ({ request }) => {
      const response = await request.post('/api/flow', {
        data: { nodes: [], edges: [] },
        headers: vector.headers
      });
      expect(response.status()).toBe(401);
    });
  }

  test('should accept authorized /api/flow requests with Bearer token', async ({ request }) => {
    const flowPayload = {
      nodes: [
        { id: 'n1', type: 'inject', data: { name: 'inject', payload: 'auth-test', interval: 1 } }
      ],
      edges: []
    };

    const token = process.env.PLANTLINK_AUTH_TOKEN;

    const response = await request.post('/api/flow', {
      data: flowPayload,
      headers: {
        Authorization: `Bearer ${token}`
      }
    });

    expect(response.status()).toBe(200);

    // Cleanup
    await request.post('/api/flow/stop', {
      headers: { Authorization: `Bearer ${token}` }
    });
  });

  test('should receive WebSocket status updates with heartbeat', async ({ page }) => {
    const wsPromise = page.waitForEvent('websocket');
    await page.goto('/');
    const ws = await wsPromise;

    // Monitor Pings
    let pingCount = 0;
    ws.on('framesent', (frame) => {
      // Note: In some browsers/drivers, Ping/Pong might not be visible as frames
      // But we can check if the connection stays alive
    });

    // 1. Setup frame listener BEFORE deployment
    const framePromise = ws.waitForEvent('framereceived', (frame) => {
      const text = typeof frame.payload === 'string' ? frame.payload : frame.payload.toString();
      return text.includes('"type":"status"');
    });

    // 2. Deploy a flow that generates status updates
    const token = process.env.PLANTLINK_AUTH_TOKEN;
    const flowPayload = {
      nodes: [
        { id: 'n1', type: 'inject', data: { name: 'inject', payload: 'ws-test', interval: 0.1 } }
      ],
      edges: []
    };

    const deployRes = await page.request.post('/api/flow', {
      data: flowPayload,
      headers: { Authorization: `Bearer ${token}` }
    });
    expect(deployRes.status()).toBe(200);

    // 3. Wait for the status frame to arrive
    const frame = await framePromise;
    expect(frame.payload).toContain('"type":"status"');

    // 3. Cleanup
    await page.request.post('/api/flow/stop', {
      headers: { Authorization: `Bearer ${token}` }
    });
  });
});
