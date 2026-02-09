import { writable } from 'svelte/store';

function createThemeStore() {
    // SSR safety
    const isBrowser = typeof window !== 'undefined';

    // Get initial value
    const stored = isBrowser ? localStorage.getItem('theme') : null;
    const systemDark = isBrowser && window.matchMedia('(prefers-color-scheme: dark)').matches;
    const initial = stored || (systemDark ? 'dark' : 'light');

    // Apply initial theme
    if (isBrowser) {
        document.documentElement.classList.toggle('dark', initial === 'dark');
    }

    const { subscribe, set, update } = writable(initial);

    return {
        subscribe,

        set: (value) => {
            set(value);
            if (isBrowser) {
                localStorage.setItem('theme', value);
                document.documentElement.classList.toggle('dark', value === 'dark');
            }
        },

        toggle: () => {
            update(current => {
                const next = current === 'dark' ? 'light' : 'dark';
                if (isBrowser) {
                    localStorage.setItem('theme', next);
                    document.documentElement.classList.toggle('dark', next === 'dark');
                }
                return next;
            });
        }
    };
}

export const theme = createThemeStore();
