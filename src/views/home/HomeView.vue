<template>
  <div class="app-grid">
    <AppCard v-for="app in appList" :key="app.id" :app="app" />
  </div>
</template>

<script setup lang="ts">
import { computed, inject, type Ref } from 'vue'
import AppCard from '@/components/card/AppCard.vue'
import rawApps from '@/config/apps.json'
import { AppConfig } from '@/types/app'

const apps = rawApps as AppConfig[]

const activeGroup = inject<Ref<string>>('activeGroup')

const appList = computed(() => {
  const currentGroup = activeGroup?.value ?? '1'

  return apps
    .filter(app => app.enabled && app.groupId === currentGroup)
    .sort((a, b) => {
      if (a.pinned && !b.pinned) return -1
      if (!a.pinned && b.pinned) return 1
      return a.order - b.order
    })
})
</script>

<style scoped lang="scss">
.app-grid {
  display: grid;
  gap: var(--lumina-gap-lg);
  grid-template-columns: repeat(auto-fill, 130px);
  justify-content: flex-start;
}
</style>