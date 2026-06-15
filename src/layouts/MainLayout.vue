<template>
  <div class="layout-root">
    <header class="window-titlebar" data-tauri-drag-region @dblclick="toggleMaximize">
      <div class="window-title" data-tauri-drag-region>Lumina - Git Assistant</div>

      <div class="window-actions" @mousedown.stop @dblclick.stop>
        <button
          class="window-button"
          :class="{ active: route.name === 'settings' }"
          type="button"
          :title="route.name === 'settings' ? t('topbar.backToGit') : t('topbar.settings')"
          @click="toggleSettings"
        >
          <Icon icon="solar:settings-linear" />
        </button>
        <button class="window-button" type="button" :title="t('topbar.minimize')" @click="minimizeWindow">
          <span class="caption-icon caption-icon--minimize" aria-hidden="true"></span>
        </button>
        <button class="window-button" type="button" :title="maximizeTitle" @click="toggleMaximize">
          <span
            class="caption-icon"
            :class="isMaximized ? 'caption-icon--restore' : 'caption-icon--maximize'"
            aria-hidden="true"
          ></span>
        </button>
        <button class="window-button window-button--close" type="button" :title="t('topbar.close')" @click="closeWindow">
          <span class="caption-icon caption-icon--close" aria-hidden="true"></span>
        </button>
      </div>
    </header>

    <div class="app-area">
      <div class="view-host">
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'

const router = useRouter()
const route = useRoute()
const { t } = useI18n({ useScope: 'global' })
const appWindow = getCurrentWindow()
const isMaximized = ref(false)
let unlistenResize: UnlistenFn | null = null

const maximizeTitle = computed(() => (isMaximized.value ? t('topbar.restore') : t('topbar.maximize')))

onMounted(async () => {
  await refreshMaximizedState()
  unlistenResize = await appWindow.onResized(refreshMaximizedState)
})

onUnmounted(() => {
  unlistenResize?.()
})

function openGitAssistant() {
  router.push({ name: 'git-assistant' })
}

function toggleSettings() {
  if (route.name === 'settings') {
    openGitAssistant()
    return
  }

  router.push({ name: 'settings' })
}

async function minimizeWindow() {
  await appWindow.minimize()
}

async function toggleMaximize() {
  await appWindow.toggleMaximize()
  await refreshMaximizedState()
}

async function closeWindow() {
  await appWindow.close()
}

async function refreshMaximizedState() {
  isMaximized.value = await appWindow.isMaximized()
}
</script>

<style scoped lang="scss">
.layout-root {
  background: var(--lumina-bg);
  background-position: center;
  background-size: cover;
  color: var(--lumina-text);
  display: flex;
  flex-direction: column;
  height: 100vh;
  min-height: 720px;
  min-width: 1120px;
  overflow: hidden;
  width: 100%;
}

.window-titlebar {
  align-items: center;
  background: var(--lumina-surface-2);
  border-bottom: 1px solid var(--lumina-card-border);
  display: flex;
  flex: 0 0 auto;
  height: 32px;
  justify-content: space-between;
  user-select: none;
}

.window-title {
  color: var(--lumina-text-secondary);
  flex: 1;
  font-size: 12px;
  line-height: 32px;
  min-width: 0;
  overflow: hidden;
  padding: 0 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.window-actions {
  align-items: center;
  display: flex;
}

.window-button {
  align-items: center;
  background: transparent;
  border: 0;
  color: var(--lumina-text);
  cursor: pointer;
  display: inline-flex;
  height: 32px;
  justify-content: center;
  transition:
    background 0.18s ease,
    color 0.18s ease;
  width: 42px;

  &:hover,
  &.active {
    background: var(--lumina-primary-soft);
    color: var(--lumina-primary);
  }

  svg {
    height: 16px;
    width: 16px;
  }
}

.caption-icon {
  display: inline-block;
  height: 12px;
  position: relative;
  width: 12px;
}

.caption-icon--minimize {
  &::before {
    background: currentcolor;
    bottom: 2px;
    content: '';
    height: 1px;
    left: 1px;
    position: absolute;
    right: 1px;
  }
}

.caption-icon--maximize {
  border: 1.4px solid currentcolor;
}

.caption-icon--restore {
  height: 13px;
  width: 13px;

  &::before,
  &::after {
    border: 1.4px solid currentcolor;
    content: '';
    height: 8px;
    position: absolute;
    width: 8px;
  }

  &::before {
    right: 0;
    top: 0;
  }

  &::after {
    background: var(--lumina-surface-2);
    bottom: 0;
    left: 0;
  }
}

.caption-icon--close {
  &::before,
  &::after {
    background: currentcolor;
    content: '';
    height: 1.4px;
    left: 0;
    position: absolute;
    top: 5px;
    width: 12px;
  }

  &::before {
    transform: rotate(45deg);
  }

  &::after {
    transform: rotate(-45deg);
  }
}

.window-button--close {
  &:hover {
    background: var(--lumina-danger);
    color: #fff;
  }
}

.app-area {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
  padding: 10px;
}

.view-host {
  flex: 1;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}
</style>
