import { test, expect } from '@playwright/test';

// Runtime E2E tests covering deep-stack features.
// Requires live backend with PLANTLINK_AUTH_TOKEN set.

test.describe('Runtime Engine E2E', () => {
  const token = process.env.PLANTLINK_AUTH_TOKEN;

  test.afterEach(async ({ request }) => {
    // Ensure flow is stopped after each test
    await request.post('/api/flow/stop', {
      headers: { Authorization: `Bearer ${token}` }
    });
  });

  test('should execute Rhai script and observe mutated output via WebSocket', async ({
    request,
    page
  }) => {
    // Flow: inject -> rhai (mutates payload) -> console (broadcasts to WS)
    const flowPayload = {
      nodes: [
        { id: 'n1', type: 'inject', data: { name: 'inject', payload: 'test-input', interval: 1 } },
        {
          id: 'n2',
          type: 'rhai',
          data: { name: 'rhai', code: 'msg.payload = msg.payload + " RHAI_OK"; return msg;' }
        },
        { id: 'n3', type: 'console', data: { name: 'console' } }
      ],
      edges: [
        { id: 'e1', source: 'n1', sourceHandle: 'output_0', target: 'n2', targetHandle: 'input_0' },
        { id: 'e2', source: 'n2', sourceHandle: 'output_0', target: 'n3', targetHandle: 'input_0' }
      ]
    };

    // 1. Open page and intercept the WebSocket
    const wsPromise = page.waitForEvent('websocket');
    await page.goto('/');
    const ws = await wsPromise;

    // 2. Listen for the Console node's Log broadcast containing the mutated payload
    const logPromise = ws.waitForEvent('framereceived', (frame) => {
      const text = typeof frame.payload === 'string' ? frame.payload : frame.payload.toString();
      return text.includes('RHAI_OK');
    });

    // 3. Deploy the flow
    const res = await request.post('/api/flow', {
      data: flowPayload,
      headers: { Authorization: `Bearer ${token}` }
    });
    expect(res.status()).toBe(200);

    // 4. Assert mutated payload arrived via WebSocket
    const frame = await logPromise;
    const text = typeof frame.payload === 'string' ? frame.payload : frame.payload.toString();
    expect(text).toContain('RHAI_OK');
  });

  test('should reject malformed flow with unregistered node type', async ({ request }) => {
    const badFlow = {
      nodes: [{ id: 'bad1', type: 'nonexistent_node_type', data: {} }],
      edges: []
    };

    const res = await request.post('/api/flow', {
      data: badFlow,
      headers: { Authorization: `Bearer ${token}` }
    });

    // Backend should return 500 with error message, NOT crash
    expect(res.status()).toBe(500);
    const body = await res.text();
    expect(body).toContain('Deployment error');

    // Verify backend is still alive after the bad request
    const health = await request.get('/health');
    expect(health.status()).toBe(200);
  });

  test('should report tasks_aborted > 0 when stopping an active flow', async ({ request }) => {
    // Deploy a flow that actively spawns timer tasks
    const flowPayload = {
      nodes: [
        {
          id: 'n1',
          type: 'inject',
          data: { name: 'inject', payload: 'timer-test', interval: 0.5 }
        },
        { id: 'n2', type: 'console', data: { name: 'console' } }
      ],
      edges: [
        { id: 'e1', source: 'n1', sourceHandle: 'output_0', target: 'n2', targetHandle: 'input_0' }
      ]
    };

    const deployRes = await request.post('/api/flow', {
      data: flowPayload,
      headers: { Authorization: `Bearer ${token}` }
    });
    expect(deployRes.status()).toBe(200);

    // Give the flow a moment to fully initialize its background tasks
    await new Promise((r) => setTimeout(r, 500));

    // Stop and assert cooperative shutdown metrics
    const stopRes = await request.post('/api/flow/stop', {
      headers: { Authorization: `Bearer ${token}` }
    });
    expect(stopRes.status()).toBe(200);

    const stopData = await stopRes.json();
    expect(stopData.tasks_aborted).toBeGreaterThan(0);
  });

  test('should maintain WebSocket connection across heartbeat interval', async ({ page }) => {
    test.setTimeout(30_000); // Allow enough time for the 15s heartbeat cycle

    const wsPromise = page.waitForEvent('websocket');
    await page.goto('/');
    const ws = await wsPromise;

    // Track if the connection closes
    let closed = false;
    ws.on('close', () => {
      closed = true;
    });

    // Hold the connection idle for 18 seconds (past the 15s Ping interval)
    await page.waitForTimeout(18_000);

    // If Ping/Pong is working, the connection should still be open
    expect(closed).toBe(false);
    expect(ws.isClosed()).toBe(false);
  });
});
