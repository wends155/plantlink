import { writable } from 'svelte/store';

// Map: NodeID -> { state: "running" | "error" | "stopped", message: string }
export const nodeStatuses = writable({});
