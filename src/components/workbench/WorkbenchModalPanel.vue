<template>
  <section class="workbench-modal-panel" :class="`size-${size}`">
    <button class="workbench-modal-close" type="button" :aria-label="closeLabel" @click="$emit('close')">
      <Icon icon="solar:close-circle-linear" />
    </button>
    <slot />
  </section>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  closeLabel: string
  size?: 'normal' | 'wide' | 'diff' | 'log'
}>(), {
  size: 'normal',
})

defineEmits<{
  (e: 'close'): void
}>()
</script>

<style scoped lang="scss">
.workbench-modal-panel {
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-lg);
  box-shadow: var(--lumina-shadow-md);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  max-height: min(720px, calc(100vh - 96px));
  overflow: hidden;
  position: relative;
  width: min(760px, calc(100vw - 72px));

  &.size-wide {
    height: min(820px, calc(100vh - 72px));
    width: min(1560px, calc(100vw - 44px));
  }

  &.size-diff {
    display: block;
    height: min(820px, calc(100vh - 76px));
    width: min(1480px, calc(100vw - 72px));
  }

  &.size-log {
    grid-template-rows: minmax(0, 1fr);
    height: min(760px, calc(100vh - 76px));
    width: min(1180px, calc(100vw - 72px));
  }
}

.workbench-modal-close {
  align-items: center;
  background: var(--lumina-surface-2);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-sm);
  color: var(--lumina-text-secondary);
  cursor: pointer;
  display: flex;
  height: 30px;
  justify-content: center;
  padding: 0;
  position: absolute;
  right: 12px;
  top: 12px;
  width: 30px;
  z-index: 3;

  &:hover {
    background: var(--lumina-button-secondary-hover);
    color: var(--lumina-text);
  }

  &:focus-visible {
    box-shadow: 0 0 0 3px var(--lumina-accent-ring);
    outline: none;
  }

  svg {
    height: 16px;
    width: 16px;
  }
}
</style>
