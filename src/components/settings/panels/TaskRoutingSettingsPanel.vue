<template>
  <section class="panel-section">
    <h2>{{ t('settings.aiRouting.title') }}</h2>
    <p>{{ t('settings.aiRouting.description') }}</p>

    <div v-if="!aiSettings.enabledModels.length" class="empty-state">
      {{ t('settings.aiRouting.empty') }}
    </div>

    <div v-else class="routing-list">
      <label v-for="task in AI_TASKS" :key="task.key" class="routing-row">
        <span>{{ t(task.i18nKey) }}</span>
        <NSelect
          class="routing-select"
          :value="aiSettings.taskModelMap[task.key]"
          :options="enabledModelOptions"
          @update:value="value => setTaskModel(task.key, value)"
        />
      </label>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NSelect } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { AI_TASKS, useAiSettingsStore } from '@/stores/ai-settings'
import type { AiTaskType } from '@/types/ai-settings'

const aiSettings = useAiSettingsStore()
const { t } = useI18n({ useScope: 'global' })
const enabledModelOptions = computed(() =>
  aiSettings.enabledModels.map(model => ({
    label: model.name,
    value: model.id,
  })),
)

function setTaskModel(task: AiTaskType, value: string | number | null) {
  aiSettings.setTaskModel(task, String(value ?? ''))
}
</script>

<style scoped lang="scss">
.panel-section {
  background: var(--lumina-card-bg);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-lg);
  box-shadow: var(--lumina-shadow-sm);
  padding: var(--lumina-gap-lg);
}

h2 {
  font-size: 18px;
  margin: 0 0 var(--lumina-gap-sm);
}

p {
  color: var(--lumina-text-secondary);
  margin: 0 0 var(--lumina-gap-lg);
}

.routing-list {
  display: flex;
  flex-direction: column;
  gap: var(--lumina-gap-md);
}

.empty-state {
  border: 1px dashed var(--lumina-card-border);
  border-radius: var(--lumina-radius-lg);
  color: var(--lumina-text-secondary);
  padding: var(--lumina-gap-lg);
  text-align: center;
}

.routing-row {
  align-items: center;
  color: var(--lumina-text);
  display: grid;
  gap: var(--lumina-gap-md);
  grid-template-columns: minmax(180px, 240px) minmax(260px, 420px);
}

.routing-select {
  width: 100%;
}

:deep(.routing-select .n-base-selection) {
  background: var(--lumina-surface-2);
  border-radius: var(--lumina-radius-md);
  min-height: 34px;
}

:deep(.routing-select .n-base-selection-label) {
  color: var(--lumina-text);
}
</style>
