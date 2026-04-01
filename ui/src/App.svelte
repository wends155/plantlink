<script>
  import FlowEditor from './lib/FlowEditor.svelte';
  import { onMount } from 'svelte';
  import { nodeStatuses } from './lib/stores/nodeStatus.js';

  onMount(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    // Use current host if serving from backend, or localhost:3000 for dev
    const host = window.location.port === '5173' ? 'localhost:3000' : window.location.host;
    const wsUrl = `${protocol}//${host}/ws`;

    let ws;

    // Defer WS connection so it doesn't block the browser 'load' event in E2E tests
    const connectTimer = setTimeout(() => {
      console.log(`Connecting to WebSocket: ${wsUrl}`);
      ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        console.log('Connected to PlantLink Runtime');
      };

      ws.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data);
          if (msg.type === 'log') {
            console.log('%c[Runtime]', 'color: #00ff00; font-weight: bold;', msg.message);
          } else if (msg.type === 'status') {
            // Update Node Status Store
            nodeStatuses.update((statuses) => ({
              ...statuses,
              [msg.data.node_id]: msg.data
            }));
          } else {
            console.log('%c[Runtime-JSON]', 'color: #00ff00; font-weight: bold;', msg);
          }
        } catch (e) {
          // Legacy plain text
          console.log('%c[Runtime]', 'color: #00ff00; font-weight: bold;', event.data);
        }
      };

      ws.onerror = (error) => {
        console.error('WebSocket Error:', error);
        console.log('WS URL that failed:', wsUrl);
      };

      ws.onclose = (event) => {
        console.log('Disconnected from PlantLink Runtime', event.code, event.reason);
      };
    }, 0);

    return () => {
      clearTimeout(connectTimer);
      if (ws) ws.close();
    };
  });
</script>

<main>
  <FlowEditor />
</main>
