<script>
  import { Moon, Sun } from 'lucide-svelte';
  import { theme } from '../stores/theme';
</script>

<button
  class="btn-icon"
  on:click={() => theme.toggle()}
  aria-label="Toggle Dark Mode"
  title="Toggle Dark Mode"
>
  {#if $theme === 'dark'}
    <Moon size={20} class="text-yellow-400" />
  {:else}
    <Sun size={20} class="text-orange-500" />
  {/if}
</button>
