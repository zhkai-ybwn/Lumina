<template>
  <div class="app-card" @click="handleClick">
    <div class="app-icon">
      <Icon :icon="app.icon" />
    </div>
    <div class="app-name">{{ app.name }}</div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'

interface AppItem {
  id: string
  name: string
  icon: string
  route?: string
}
const props = defineProps<{
  app: AppItem
}>()
const router = useRouter()
const handleClick = () => {
  if (props.app.route) {
    router.push(props.app.route)
  }
}
</script>

<style lang="scss" scoped>
.app-card {
  width: 130px;
  height: 130px;
  background: var(--lumina-card-bg);
  border-radius: var(--lumina-radius-lg);
  border: 1px solid var(--lumina-card-border);

  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;

  padding: var(--lumina-gap-md);
  box-shadow: var(--lumina-shadow-sm);

  transition:
    transform 0.25s ease,
    box-shadow 0.25s ease,
    border-color 0.25s ease,
    background 0.25s ease;

  cursor: pointer;
  user-select: none;

  &:hover {
    transform: translateY(-4px) scale(1.03);
    box-shadow: var(--lumina-shadow-md);
    border-color: var(--lumina-primary);
  }

  /* 磁吸效果（轻微 3D） */
  &:hover {
    transition: transform 0.25s ease;
  }

  .app-icon {
    width: 56px;
    height: 56px;
    border-radius: 16px;

    background: var(--lumina-primary-soft);
    backdrop-filter: blur(6px);

    display: flex;
    justify-content: center;
    align-items: center;
    margin-bottom: var(--lumina-gap-sm);

    svg {
      width: 32px;
      height: 32px;
      color: var(--lumina-primary);
    }
  }

  .app-name {
    font-size: var(--lumina-font-title);
    font-weight: 600;
    text-align: center;
    color: var(--lumina-text-secondary);
  }
}
</style>
