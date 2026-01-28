/** @type {import('tailwindcss').Config} */
export default {
    darkMode: 'class',
    content: ['./src/**/*.{html,js,svelte,ts}'],
    theme: {
        extend: {
            colors: {
                'node-red': '#8f0000',
                'node-grey': '#e5e5e5',
                'node-body': '#fbfbfb',
                'port-in': '#D9D9D9',
                'port-out': '#D9D9D9',
            }
        },
    },
    plugins: [],
}
