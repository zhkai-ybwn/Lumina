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
        <select :value="aiSettings.taskModelMap[task.key]" @change="setTaskModel(task.key, $event)">
          <option v-for="model in aiSettings.enabledModels" :key="model.id" :value="model.id">
            {{ model.name }}
          </option>
        </select>
      </label>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { AI_TASKS, useAiSettingsStore } from '@/stores/ai-settings'
import type { AiTaskType } from '@/types/ai-settings'

const aiSettings = useAiSettingsStore()
const { t } = useI18n({ useScope: 'global' })

function setTaskModel(task: AiTaskType, event: Event) {
  aiSettings.setTaskModel(task, (event.target as HTMLSelectElement).value)
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

select {
  background: var(--lumina-surface-2);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-md);
  box-sizing: border-box;
  color: var(--lumina-text);
  height: 34px;
  padding: 0 10px;
}
</style>
