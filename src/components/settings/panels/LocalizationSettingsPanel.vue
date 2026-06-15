<template>
  <section class="panel-section">
    <h2>{{ t('settings.language.title') }}</h2>
    <p>{{ t('settings.language.description') }}</p>
    <n-radio-group :value="preferencesStore.locale" @update:value="preferencesStore.setLocale">
      <n-radio value="zh-CN">{{ t('settings.language.optionZh') }}</n-radio>
      <n-radio value="en-US">{{ t('settings.language.optionEn') }}</n-radio>
    </n-radio-group>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { usePreferencesStore } from '@/stores/preferences'

const preferencesStore = usePreferencesStore()
const { t } = useI18n({ useScope: 'global' })
</script>

<style scoped lang="scss">
.panel-section {
  background: var(--lumina-card-bg);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-lg);
  box-shadow: var(--lumina-shadow-sm);
  padding: var(--lumina-gap-lg);

  h2 {
    font-size: 18px;
    margin: 0 0 var(--lumina-gap-sm);
  }

  p {
    color: var(--lumina-text-secondary);
    margin: 0 0 var(--lumina-gap-lg);
  }
}
</style>
