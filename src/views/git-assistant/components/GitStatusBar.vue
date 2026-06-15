<template>
  <section class="status-bar">
    <div class="status-bar__main">
      <div class="context-pill mono" :title="repoPath || t('gitAssistant.repo.emptyPath')">
        <span class="context-label">{{ t('gitAssistant.repo.currentRepoShort') }}</span>
        <span class="context-value">{{ repoName || t('gitAssistant.repo.emptyPath') }}</span>
      </div>

      <div class="context-pill">
        <span class="context-label">{{ t('gitAssistant.repo.branchShort') }}</span>
        <span class="context-value">{{ branch || '--' }}</span>
      </div>

      <div class="metric-pill">
        <span>{{ t('gitAssistant.repo.summaryTotal') }}</span>
        <strong>{{ summary.total }}</strong>
      </div>
      <div class="metric-pill">
        <span>{{ t('gitAssistant.repo.summaryStaged') }}</span>
        <strong>{{ summary.staged }}</strong>
      </div>
      <div class="metric-pill">
        <span>{{ t('gitAssistant.repo.summaryUnstaged') }}</span>
        <strong>{{ summary.unstaged }}</strong>
      </div>
      <div class="metric-pill">
        <span>{{ t('gitAssistant.repo.summaryUntracked') }}</span>
        <strong>{{ summary.untracked }}</strong>
      </div>
      <div class="metric-pill metric-pill--accent">
        <span>{{ t('gitAssistant.repo.summaryRecommended') }}</span>
        <strong>{{ recommendedCount }}</strong>
      </div>
    </div>

    <div class="status-bar__actions">
      <button class="tool-btn" :title="repoPath || t('gitAssistant.repo.emptyPath')">
        {{ statusText }}
      </button>
      <button class="tool-btn" @click="$emit('pick-directory')">{{ t('gitAssistant.repo.chooseDirectory') }}</button>
      <button class="tool-btn primary" :disabled="loading || !repoPath" @click="$emit('refresh')">
        {{ loading ? t('gitAssistant.repo.refreshing') : t('gitAssistant.repo.refreshRepo') }}
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useLocale } from '@/hooks/useLocale'
import type { GitAssistantSummary } from '../git-assistant.types'

const props = defineProps<{
  repoPath: string
  branch: string
  loading: boolean
  summary: GitAssistantSummary
  recommendedCount: number
  statusText: string
}>()

defineEmits<{
  (e: 'pick-directory'): void
  (e: 'refresh'): void
}>()

const { t } = useLocale()

const repoName = computed(() => {
  const normalized = props.repoPath.replace(/\\/g, '/')
  const parts = normalized.split('/').filter(Boolean)
  return parts[parts.length - 1] ?? ''
})
</script>

<style scoped lang="scss">
.status-bar {
  align-items: center;
  background: var(--lumina-surface-2);
  border: 1px solid var(--lumina-card-border);
  border-radius: 14px;
  box-shadow: var(--lumina-shadow-sm);
  display: flex;
  gap: 10px;
  justify-content: space-between;
  min-height: 42px;
  overflow: hidden;
  padding: 6px 10px;
}

.status-bar__main,
.status-bar__actions {
  align-items: center;
  display: flex;
  gap: 8px;
  min-width: 0;
}

.status-bar__main {
  flex: 1;
  overflow: hidden;
}

.status-bar__actions {
  flex-shrink: 0;
}

.context-pill,
.metric-pill,
.tool-btn {
  border: 1px solid var(--lumina-card-border);
  border-radius: 999px;
  font-size: 11px;
  height: 28px;
}

.context-pill,
.metric-pill {
  align-items: center;
  background: var(--lumina-surface-3);
  display: inline-flex;
  gap: 8px;
  padding: 0 10px;
  white-space: nowrap;
}

.context-pill {
  max-width: 240px;
  min-width: 0;
}

.context-label {
  color: var(--lumina-text-secondary);
}

.context-value {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.metric-pill strong {
  color: var(--lumina-text);
}

.metric-pill--accent {
  background: var(--lumina-primary-soft);
}

.tool-btn {
  align-items: center;
  background: var(--lumina-button-secondary-bg);
  color: var(--lumina-text);
  cursor: pointer;
  display: inline-flex;
  padding: 0 10px;
  transition:
    background 0.18s ease,
    border-color 0.18s ease,
    color 0.18s ease;

  &:hover {
    background: var(--lumina-button-secondary-hover);
  }
}

.primary {
  background: var(--lumina-primary);
  border-color: var(--lumina-primary);
  color: #fff;
}

.mono {
  font-family: SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
}
</style>
