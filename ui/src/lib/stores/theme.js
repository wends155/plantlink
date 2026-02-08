import { writable } from 'svelte/store';

// Available: 'light', 'dark', 'system', or custom like 'theme-nord'
export const theme = writable('system');

// List of available themes for UI selector
export const availableThemes = [
    { id: 'light', name: 'Light' },
    { id: 'dark', name: 'Dark' },
    { id: 'system', name: 'System' },
    // Future themes:
    // { id: 'theme-nord', name: 'Nord' },
];

// Apply theme class to document
if (typeof window !== 'undefined') {
    theme.subscribe(value => {
        const root = document.documentElement;

        // Remove existing theme classes
        root.classList.remove('dark', 'theme-nord', 'theme-solarized');

        if (value === 'system') {
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            if (prefersDark) root.classList.add('dark');
        } else if (value === 'dark') {
            root.classList.add('dark');
        } else if (value.startsWith('theme-')) {
            root.classList.add(value);
        }
        // 'light' = no class needed (default)
    });
}
