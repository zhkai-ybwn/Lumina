<template>
  <div class="layout-root">
    <header class="window-titlebar" data-tauri-drag-region @dblclick="toggleMaximize">
      <div class="window-title" data-tauri-drag-region>{{ windowTitle }}</div>

      <div class="window-actions" @mousedown.stop @dblclick.stop>
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
      <nav class="sidebar">
        <div class="sidebar-top">
          <img src="@/assets/logo.png" class="sidebar-logo" alt="Lumina" />
        </div>

        <div class="sidebar-nav">
          <button
            v-for="item in navItems"
            :key="item.route"
            class="sidebar-item"
            :class="{ active: route.name === item.route }"
            type="button"
            :title="item.label"
            @click="router.push({ name: item.route })"
          >
            <Icon :icon="item.icon" />
          </button>
        </div>

        <div class="sidebar-bottom">
          <button
            class="sidebar-item"
            :class="{ active: route.name === 'settings' }"
            type="button"
            :title="t('topbar.settings')"
            @click="toggleSettings"
          >
            <Icon icon="solar:settings-linear" />
          </button>
        </div>
      </nav>

      <div class="view-host">
        <router-view v-slot="{ Component }">
          <keep-alive>
            <component :is="Component" />
          </keep-alive>
        </router-view>
      </div>
    </div>

    <n-modal v-model:show="exitDialogOpen" :auto-focus="false" :mask-closable="false">
      <section class="exit-dialog" role="dialog" aria-modal="true" :aria-label="t('topbar.exitTitle')">
        <header>
          <h3>{{ t('topbar.exitTitle') }}</h3>
          <p>{{ runningProcesses.length ? t('topbar.exitRunningHint') : t('topbar.exitIdleHint') }}</p>
        </header>
        <div v-if="runningProcesses.length" class="exit-process-list">
          <div v-for="process in runningProcesses" :key="process.id" class="exit-process-row">
            <strong>{{ process.projectName }}</strong>
            <span>{{ process.scriptName }} · PID {{ process.pid }}</span>
          </div>
        </div>
        <footer>
          <button type="button" @click="exitDialogOpen = false">{{ t('topbar.cancel') }}</button>
          <button type="button" @click="hideToTray">{{ t('topbar.hideToTray') }}</button>
          <button class="danger" type="button" :disabled="exiting" @click="exitApplication">
            {{ exiting ? t('topbar.exiting') : t('topbar.exitAndStop') }}
          </button>
        </footer>
      </section>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useMessage } from 'naive-ui'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listProjectProcesses, stopAllProjectProcesses, type ProjectProcessSnapshot } from '@/services/project/project-service'

const router = useRouter()
const route = useRoute()
const { t } = useI18n({ useScope: 'global' })
const message = useMessage()
const appWindow = getCurrentWindow()
const isMaximized = ref(false)
let unlistenResize: UnlistenFn | null = null
let unlistenCloseRequested: UnlistenFn | null = null
let unlistenTrayExit: UnlistenFn | null = null
const exitDialogOpen = ref(false)
const exiting = ref(false)
const runningProcesses = ref<ProjectProcessSnapshot[]>([])

const navItems = computed(() => [
  { route: 'git-assistant', label: t('workbench.git'), icon: 'solar:code-square-linear' },
  { route: 'devdock', label: t('workbench.devdock'), icon: 'solar:folder-linear' },
])

const windowTitle = computed(() => {
  if (route.name === 'devdock') return t('topbar.titleDevDock')
  if (route.name === 'settings') return t('topbar.titleSettings')
  return t('topbar.titleGit')
})
const maximizeTitle = computed(() => (isMaximized.value ? t('topbar.restore') : t('topbar.maximize')))

onMounted(async () => {
  await refreshMaximizedState()
  unlistenResize = await appWindow.onResized(refreshMaximizedState)
  unlistenCloseRequested = await appWindow.onCloseRequested(event => {
    event.preventDefault()
    void requestExit()
  })
  unlistenTrayExit = await listen('lumina://request-exit', () => {
    void requestExit()
  })
})

onUnmounted(() => {
  unlistenResize?.()
  unlistenCloseRequested?.()
  unlistenTrayExit?.()
})

function toggleSettings() {
  if (route.name === 'settings') {
    router.push({ name: 'git-assistant' })
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
  await requestExit()
}

async function requestExit() {
  try {
    runningProcesses.value = await listProjectProcesses()
    exitDialogOpen.value = true
  } catch (error) {
    message.error(error instanceof Error ? error.message : String(error))
  }
}

async function hideToTray() {
  exitDialogOpen.value = false
  await appWindow.hide()
}

async function exitApplication() {
  exiting.value = true
  try {
    await stopAllProjectProcesses()
    await appWindow.destroy()
  } catch (error) {
    message.error(error instanceof Error ? error.message : String(error), { duration: 8000 })
  } finally {
    exiting.value = false
  }
}

async function refreshMaximizedState() {
  isMaximized.value = await appWindow.isMaximized()
}
</script>

<style scoped lang="scss">
.layout-root {
  background: var(--lumina-bg);
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
  background: color-mix(in srgb, var(--lumina-surface-2) 92%, var(--lumina-bg));
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
  transition: background 0.18s ease, color 0.18s ease;
  width: 42px;

  &:hover,
  &.active {
    background: var(--lumina-button-secondary-hover);
    color: var(--lumina-primary);
  }

  svg {
    height: 16px;
    width: 16px;
  }
}

.exit-dialog {
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: 8px;
  box-shadow: var(--lumina-shadow-lg);
  color: var(--lumina-text);
  padding: 20px;
  width: min(480px, calc(100vw - 32px));

  h3,
  p {
    margin: 0;
  }

  p {
    color: var(--lumina-text-secondary);
    margin-top: 6px;
  }

  footer {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 18px;
  }

  button {
    background: var(--lumina-surface-2);
    border: 1px solid var(--lumina-card-border);
    border-radius: 6px;
    color: var(--lumina-text);
    min-height: 32px;
    padding: 0 12px;
  }

  button.danger {
    background: var(--lumina-danger);
    color: #fff;
  }
}

.exit-process-list {
  border-block: 1px solid var(--lumina-card-border);
  margin-top: 16px;
  max-height: 220px;
  overflow-y: auto;
}

.exit-process-row {
  align-items: center;
  display: flex;
  justify-content: space-between;
  min-height: 42px;

  & + & {
    border-top: 1px solid var(--lumina-card-border);
  }

  span {
    color: var(--lumina-text-secondary);
    font-size: 12px;
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
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}

.sidebar {
  align-items: center;
  background: color-mix(in srgb, var(--lumina-surface-2) 80%, var(--lumina-bg));
  border-right: 1px solid var(--lumina-card-border);
  display: flex;
  flex: 0 0 48px;
  flex-direction: column;
  padding: 8px 0;
  user-select: none;
}

.sidebar-top {
  display: flex;
  justify-content: center;
  margin-bottom: 12px;
}

.sidebar-logo {
  border-radius: 6px;
  height: 28px;
  width: 28px;
}

.sidebar-nav {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 4px;
}

.sidebar-bottom {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sidebar-item {
  align-items: center;
  background: transparent;
  border: 0;
  border-radius: 8px;
  color: var(--lumina-text-secondary);
  cursor: pointer;
  display: flex;
  height: 36px;
  justify-content: center;
  transition: background 0.15s ease, color 0.15s ease;
  width: 36px;

  &:hover {
    background: var(--lumina-button-secondary-hover);
    color: var(--lumina-text);
  }

  &.active {
    background: color-mix(in srgb, var(--lumina-primary) 12%, transparent);
    color: var(--lumina-primary);
  }

  svg {
    height: 18px;
    width: 18px;
  }
}

.view-host {
  flex: 1;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}
</style>
