import { writable } from 'svelte/store';

// Map: NodeID -> { state: "running" | "error" | "stopped", message: string }
export const nodeStatuses = writable({});

// Helper to initialize a node as stopped
export function initNodeAsStopped(nodeId) {
    nodeStatuses.update(statuses => ({
        ...statuses,
        [nodeId]: { state: 'stopped', message: '' }
    }));
}
