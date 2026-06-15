<template>
  <div class="sidebar-item" :class="{ active }" @click="$emit('click')">
    <Icon :icon="icon" />
    <span v-if="label">{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
const emit = defineEmits(['click'])

defineProps({
  icon: String,
  label: String,
  active: Boolean,
})
</script>

<style scoped lang="scss">
.sidebar-item {
  width: 100%;
  height: 54px;

  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;

  gap: 4px;
  cursor: pointer;

  color: var(--lumina-text-secondary);
  transition: all 0.25s ease;

  svg {
    width: 22px;
    height: 22px;
  }

  span {
    font-size: 12px;
  }

  &:hover {
    color: var(--lumina-primary);
  }

  &.active {
    background: var(--lumina-card-hover-bg);
    border-left: 3px solid var(--lumina-primary);
    color: var(--lumina-primary);
  }
}
</style>
