import { writable, derived } from 'svelte/store';

/**
 * Registry of all available themes.
 * Each entry defines the theme's CSS class, its base color mode
 * (for Tailwind dark: utilities and SvelteFlow colorMode), and
 * a human-readable label for future UI selectors.
 *
 * @type {ReadonlyArray<{name: string, cssClass: string|null, colorMode: 'light'|'dark', label: string}>}
 */
export const THEMES = Object.freeze([
  { name: 'light', cssClass: null, colorMode: 'light', label: 'Light' },
  { name: 'dark', cssClass: 'dark', colorMode: 'dark', label: 'Dark' },
  { name: 'nord', cssClass: 'theme-nord', colorMode: 'dark', label: 'Nord' }
]);

/**
 * Returns the base color mode ('light' or 'dark') for a given theme name.
 * Falls back to 'light' if the theme is not found.
 * @param {string} name
 * @returns {'light' | 'dark'}
 */
export function getColorMode(name) {
  const themeEntry = THEMES.find((t) => t.name === name);
  return themeEntry ? themeEntry.colorMode : 'light';
}

function createThemeStore() {
  // SSR safety
  const isBrowser = typeof window !== 'undefined';

  // Collect all unique CSS classes for cleanup
  const allCssClasses = THEMES.map((t) => t.cssClass).filter(Boolean);
  if (!allCssClasses.includes('dark')) {
    allCssClasses.push('dark');
  }
  const uniqueClasses = [...new Set(allCssClasses)];

  // Get initial value and validate
  let initial = isBrowser ? localStorage.getItem('theme') : 'light';
  if (!THEMES.find((t) => t.name === initial)) {
    const systemDark = isBrowser && window.matchMedia('(prefers-color-scheme: dark)').matches;
    initial = systemDark ? 'dark' : 'light';
  }

  const themeStore = writable(initial);
  const { subscribe, set: svelteSet, update } = themeStore;

  /**
   * Internal helper to apply theme classes to the DOM
   */
  const applyTheme = (name) => {
    if (!isBrowser) return;

    const entry = THEMES.find((t) => t.name === name) || THEMES[0];

    // Remove all known theme classes to prevent contamination
    uniqueClasses.forEach((c) => document.documentElement.classList.remove(c));

    // Apply primary theme class if defined
    if (entry.cssClass) {
      document.documentElement.classList.add(entry.cssClass);
    }

    // Ensure .dark is applied for all dark-variant themes (matches Tailwind config)
    if (entry.colorMode === 'dark' && entry.cssClass !== 'dark') {
      document.documentElement.classList.add('dark');
    }

    localStorage.setItem('theme', name);
  };

  // Apply initial theme on mount
  applyTheme(initial);

  const store = {
    subscribe,

    /**
     * Set a specific theme by name
     * @param {string} value
     */
    set: (value) => {
      const entry = THEMES.find((t) => t.name === value);
      if (!entry) {
        console.warn(`Invalid theme: ${value}. Falling back to 'light'.`);
        svelteSet('light');
        applyTheme('light');
      } else {
        svelteSet(value);
        applyTheme(value);
      }
    },

    /**
     * Simple toggle between light and dark (backward compatibility for icon button)
     */
    toggle: () => {
      update((current) => {
        const next = current === 'dark' ? 'light' : 'dark';
        applyTheme(next);
        return next;
      });
    }
  };

  // Derived store for SvelteFlow colorMode prop
  store.colorMode = derived(themeStore, ($theme) => getColorMode($theme));

  return store;
}

export const theme = createThemeStore();
