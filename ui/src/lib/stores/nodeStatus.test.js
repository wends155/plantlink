import { describe, it, expect, beforeEach } from 'vitest';
import { nodeStatuses, initNodeAsStopped } from './nodeStatus.js';
import { get } from 'svelte/store';

describe('nodeStatus store', () => {
    beforeEach(() => {
        nodeStatuses.set({});
    });

    it('initializes node status as stopped', () => {
        initNodeAsStopped('node-1');
        const state = get(nodeStatuses);

        expect(state['node-1']).toBeDefined();
        expect(state['node-1'].state).toBe('stopped');
        expect(state['node-1'].message).toBe('');
    });

    it('adds multiple nodes independently', () => {
        initNodeAsStopped('node-1');
        initNodeAsStopped('node-2');

        const state = get(nodeStatuses);
        expect(state['node-1']).toBeDefined();
        expect(state['node-2']).toBeDefined();
    });

    it('preserves existing states when initing new node', () => {
        nodeStatuses.set({
            'node-existing': { state: 'running', message: 'ok' }
        });

        initNodeAsStopped('node-new');

        const state = get(nodeStatuses);
        expect(state['node-existing'].state).toBe('running');
        expect(state['node-new'].state).toBe('stopped');
    });
});
