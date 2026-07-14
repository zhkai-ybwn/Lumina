<template>
  <div class="settings-root">
    <SettingsNav v-model="active" :sections="navSections" />

    <div class="settings-content">
      <header class="settings-header">
        <div>
          <h1>{{ t('settings.title') }}</h1>
          <p>{{ t('settings.description') }}</p>
        </div>
      </header>

      <component :is="activeComponent" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import SettingsNav from './SettingsNav.vue'
import LocalizationSettingsPanel from './panels/LocalizationSettingsPanel.vue'
import CloseBehaviorSettingsPanel from './panels/CloseBehaviorSettingsPanel.vue'
import ModelSettingsPanel from './panels/ModelSettingsPanel.vue'
import ProjectProfileSettingsPanel from './panels/ProjectProfileSettingsPanel.vue'
import TaskRoutingSettingsPanel from './panels/TaskRoutingSettingsPanel.vue'
import ThemeSettingsPanel from './panels/ThemeSettingsPanel.vue'

const { t } = useI18n({ useScope: 'global' })
const active = ref('profile')

const navSections = computed(() => [
  {
    key: 'project',
    label: t('settings.navSections.project'),
    items: [{ key: 'profile', label: t('settings.nav.profile'), icon: 'solar:code-square-linear' }],
  },
  {
    key: 'app',
    label: t('settings.navSections.app'),
    items: [
      { key: 'language', label: t('settings.nav.language'), icon: 'solar:global-linear' },
      { key: 'theme', label: t('settings.nav.theme'), icon: 'solar:moon-linear' },
      { key: 'closeBehavior', label: t('settings.nav.closeBehavior'), icon: 'solar:power-linear' },
      { key: 'models', label: t('settings.nav.models'), icon: 'solar:cpu-bolt-linear' },
      { key: 'routing', label: t('settings.nav.routing'), icon: 'solar:routing-3-linear' },
    ],
  },
])

const activeComponent = computed(() => {
  switch (active.value) {
    case 'profile':
      return ProjectProfileSettingsPanel
    case 'theme':
      return ThemeSettingsPanel
    case 'closeBehavior':
      return CloseBehaviorSettingsPanel
    case 'models':
      return ModelSettingsPanel
    case 'routing':
      return TaskRoutingSettingsPanel
    default:
      return LocalizationSettingsPanel
  }
})
</script>

<style scoped lang="scss">
.settings-root {
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: 10px;
  display: flex;
  height: 100%;
  overflow: hidden;
  width: 100%;
}

.settings-content {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 22px;
  min-height: 0;
  overflow-y: auto;
  padding: 28px 32px;
}

.settings-header {
  max-width: 1040px;

  h1 {
    font-size: 26px;
    line-height: 1.2;
    margin: 0 0 8px;
  }

  p {
    color: var(--lumina-text-secondary);
    margin: 0;
  }
}
</style>
