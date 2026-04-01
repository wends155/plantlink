<script>
  /**
   * Button component with semantic styling variants.
   * @type {'primary' | 'success' | 'danger' | 'secondary'}
   */
  export let variant = 'primary';

  /**
   * Button size.
   * @type {'sm' | 'md' | 'lg'}
   */
  export let size = 'md';

  /** Disabled state */
  export let disabled = false;

  /** Button type for forms */
  export let type = 'button';
</script>

<button
  {type}
  {disabled}
  class="btn btn-{variant}"
  class:btn-sm={size === 'sm'}
  class:btn-lg={size === 'lg'}
  on:click
>
  <slot />
</button>
