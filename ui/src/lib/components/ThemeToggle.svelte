<script>
    import { onMount } from "svelte";
    import { Moon, Sun } from "lucide-svelte";
    import { theme } from "../stores/theme";

    let isDark = false;

    // Initialize
    onMount(() => {
        const stored = localStorage.theme;
        const systemDark = window.matchMedia(
            "(prefers-color-scheme: dark)",
        ).matches;

        if (stored === "dark" || (!stored && systemDark)) {
            isDark = true;
            document.documentElement.classList.add("dark");
            $theme = "dark";
        } else {
            isDark = false;
            document.documentElement.classList.remove("dark");
            $theme = "light";
        }
    });

    const toggleTheme = () => {
        isDark = !isDark;
        if (isDark) {
            document.documentElement.classList.add("dark");
            localStorage.theme = "dark";
            $theme = "dark";
        } else {
            document.documentElement.classList.remove("dark");
            localStorage.theme = "light";
            $theme = "light";
        }
    };
</script>

<button
    on:click={toggleTheme}
    class="p-2 rounded-full bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
    aria-label="Toggle Dark Mode"
    title="Toggle Dark Mode"
>
    {#if isDark}
        <Moon size={20} class="text-yellow-400" />
    {:else}
        <Sun size={20} class="text-orange-500" />
    {/if}
</button>
