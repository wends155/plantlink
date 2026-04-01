<script>
  /** Label text */
  export let label = '';

  /** For attribute linking */
  export let forId = '';
</script>

<div class="form-group">
  {#if label}
    <label for={forId} class="label">{label}</label>
  {/if}
  <slot />
</div>
