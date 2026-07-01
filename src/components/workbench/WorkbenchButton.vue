<template>
  <button class="workbench-button" :class="variantClass" :type="type" :disabled="disabled">
    <slot />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  disabled?: boolean
  type?: 'button' | 'submit' | 'reset'
  variant?: 'secondary' | 'primary' | 'danger' | 'ghost'
}>(), {
  disabled: false,
  type: 'button',
  variant: 'secondary',
})

const variantClass = computed(() => `variant-${props.variant}`)
</script>

<style scoped lang="scss">
.workbench-button {
  align-items: center;
  background: var(--lumina-button-secondary-bg);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-sm);
  color: var(--lumina-text);
  cursor: pointer;
  display: inline-flex;
  flex: 0 0 auto;
  font-size: 11px;
  font-weight: 600;
  height: 28px;
  justify-content: center;
  padding: 0 10px;

  &:hover:not(:disabled) {
    background: var(--lumina-button-secondary-hover);
  }

  &:focus-visible {
    box-shadow: 0 0 0 3px var(--lumina-accent-ring);
    outline: none;
  }

  &:disabled {
    cursor: not-allowed;
    opacity: 0.56;
  }
}

.variant-primary {
  background: var(--lumina-primary);
  border-color: var(--lumina-primary);
  color: #fff;
}

.variant-danger {
  color: var(--lumina-danger);

  &:hover:not(:disabled) {
    border-color: color-mix(in srgb, var(--lumina-danger) 45%, var(--lumina-card-border));
  }
}

.variant-ghost {
  background: transparent;
  border-color: transparent;
  color: var(--lumina-text-secondary);
}
</style>
