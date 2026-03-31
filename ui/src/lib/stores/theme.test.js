import { describe, it, expect, beforeEach, vi } from 'vitest';

// We reset modules because theme.js initializes based on environment on mount
import { theme, THEMES, getColorMode } from './theme.js';
import { get } from 'svelte/store';

describe('theme store', () => {
    beforeEach(() => {
        // Reset localStorage and document classes before each test
        localStorage.clear();
        document.documentElement.className = '';
        vi.clearAllMocks();
    });

    it('toggles between light and dark', () => {
        const initial = get(theme);

        theme.toggle();
        const toggled = get(theme);
        expect(toggled).not.toBe(initial);

        theme.toggle();
        expect(get(theme)).toBe(initial);
    });

    it('sets specific theme and persists to localStorage', () => {
        theme.set('dark');
        expect(get(theme)).toBe('dark');
        expect(localStorage.getItem('theme')).toBe('dark');
        expect(document.documentElement.classList.contains('dark')).toBe(true);

        theme.set('light');
        expect(get(theme)).toBe('light');
        expect(localStorage.getItem('theme')).toBe('light');
        expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('supports setting a custom theme and applies correct CSS class', () => {
        theme.set('nord');
        expect(get(theme)).toBe('nord');
        expect(localStorage.getItem('theme')).toBe('nord');
        expect(document.documentElement.classList.contains('theme-nord')).toBe(true);
        expect(document.documentElement.classList.contains('dark')).toBe(true); // nord is dark-variant
    });

    it('removes previous theme class when switching themes', () => {
        theme.set('nord');
        expect(document.documentElement.classList.contains('theme-nord')).toBe(true);
        
        theme.set('light');
        expect(document.documentElement.classList.contains('theme-nord')).toBe(false);
        expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('falls back to light for unknown theme names', () => {
        const spy = vi.spyOn(console, 'warn').mockImplementation(() => {});
        theme.set('invalid-theme-name');
        expect(get(theme)).toBe('light');
        expect(document.documentElement.className).toBe('');
        spy.mockRestore();
    });

    it('exposes THEMES registry as frozen array', () => {
        expect(Array.isArray(THEMES)).toBe(true);
        expect(Object.isFrozen(THEMES)).toBe(true);
        expect(THEMES.find(t => t.name === 'nord')).toBeDefined();
    });

    it('provides a colorMode helper', () => {
        expect(getColorMode('light')).toBe('light');
        expect(getColorMode('dark')).toBe('dark');
        expect(getColorMode('nord')).toBe('dark');
        expect(getColorMode('unknown')).toBe('light');
    });
});
