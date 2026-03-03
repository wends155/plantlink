import { describe, it, expect, beforeEach } from 'vitest';

// We reset modules because theme.js initializes based on environment on mount
import { theme } from './theme.js';
import { get } from 'svelte/store';

describe('theme store', () => {
    beforeEach(() => {
        // Reset localStorage and document classes before each test
        localStorage.clear();
        document.documentElement.className = '';
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
});
