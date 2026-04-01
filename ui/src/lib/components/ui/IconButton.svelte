<script>
  /** Accessible label */
  export let label = '';

  /** Title tooltip */
  export let title = '';
</script>

<button class="btn-icon" aria-label={label} {title} on:click>
  <slot />
</button>
