<script>
  /** Label text (optional) */
  export let label = '';

  /** Input ID (required if label provided) */
  export let id = '';

  /** Bound value */
  export let value = '';

  /** Input type */
  export let type = 'text';

  /** Placeholder text */
  export let placeholder = '';

  /** Use monospace font */
  export let mono = false;
</script>

<div class="form-group">
  {#if label}
    <label for={id} class="label">{label}</label>
  {/if}
  <input {id} {type} {placeholder} class="input" class:font-mono={mono} bind:value on:input />
</div>
