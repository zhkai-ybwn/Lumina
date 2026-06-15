<template>
  <nav class="settings-nav">
    <section v-for="section in sections" :key="section.key" class="settings-nav__section">
      <div class="settings-nav__section-title">{{ section.label }}</div>
      <button
        v-for="item in section.items"
        :key="item.key"
        class="settings-nav__item"
        :class="{ active: item.key === modelValue }"
        type="button"
        @click="$emit('update:modelValue', item.key)"
      >
        <Icon :icon="item.icon" />
        <span>{{ item.label }}</span>
      </button>
    </section>
  </nav>
</template>

<script setup lang="ts">
defineProps<{
  modelValue: string
  sections: Array<{
    key: string
    label: string
    items: Array<{ key: string; label: string; icon: string }>
  }>
}>()

defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()
</script>

<style scoped lang="scss">
.settings-nav {
  background: color-mix(in srgb, var(--lumina-surface-2) 78%, var(--lumina-bg));
  border-right: 1px solid var(--lumina-card-border);
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 18px 10px;
  width: 188px;
}

.settings-nav__section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.settings-nav__section-title {
  color: var(--lumina-text-secondary);
  font-size: 11px;
  font-weight: 600;
  padding: 0 12px 4px;
}

.settings-nav__item {
  align-items: center;
  background: transparent;
  border: 0;
  border-radius: 8px;
  color: var(--lumina-text-secondary);
  cursor: pointer;
  display: flex;
  gap: 10px;
  min-height: 38px;
  padding: 0 12px;
  text-align: left;
  transition: all 0.2s ease;

  &:hover {
    background: var(--lumina-button-secondary-hover);
    color: var(--lumina-primary);
  }

  &.active {
    background: var(--lumina-primary-soft);
    color: var(--lumina-primary);
  }

  svg {
    flex: 0 0 auto;
    height: 18px;
    width: 18px;
  }

  span {
    font-size: 14px;
  }
}
</style>
