import { describe, it, expect } from 'vitest';
import {
    getNodeDefinition,
    getAllNodeTypes,
    getNodesByCategory,
    getInputLabels,
    getOutputLabels,
    getPortSchema
} from './nodeDefinitions.js';

describe('nodeDefinitions', () => {
    it('returns definition for an existing node type', () => {
        const def = getNodeDefinition('inject');
        expect(def).not.toBeNull();
        expect(def.category).toBe('Common');
        expect(def.displayName).toBe('inject');
    });

    it('returns null for nonexistent node type', () => {
        expect(getNodeDefinition('nonexistent')).toBeNull();
    });

    it('returns array of all known node types', () => {
        const types = getAllNodeTypes();
        expect(types.length).toBeGreaterThan(0);
        expect(types).toContain('inject');
        expect(types).toContain('console');
        expect(types).toContain('rhai-function');
    });

    it('groups nodes by category', () => {
        const groups = getNodesByCategory();
        expect(groups).toHaveProperty('Common');
        expect(groups).toHaveProperty('Network');
        expect(groups).toHaveProperty('Function');
        expect(groups['Common']).toHaveProperty('inject');
    });

    it('returns input labels for a node', () => {
        const labels = getInputLabels('console');
        expect(labels).toEqual(['Message']);
    });

    it('returns empty array of input labels for node with no inputs', () => {
        const labels = getInputLabels('inject');
        expect(labels).toEqual([]);
    });

    it('returns output labels for a node', () => {
        const labels = getOutputLabels('inject');
        expect(labels).toEqual(['Message']);
    });

    it('returns correct port schema', () => {
        const schema = getPortSchema('nats-sub');
        expect(schema.inputs).toHaveLength(1);
        expect(schema.outputs).toHaveLength(1);
        expect(schema.inputs[0].acceptTypes).toContain('connection');
    });

    it('returns empty schema for nonexistent node', () => {
        const schema = getPortSchema('nonexistent');
        expect(schema).toEqual({ inputs: [], outputs: [] });
    });
});
