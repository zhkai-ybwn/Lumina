<template>
  <section class="diff-viewer panel-shell">
    <header class="diff-header">
      <div class="diff-header__main">
        <div class="panel-eyebrow">{{ t('gitAssistant.detail.eyebrow') }}</div>
        <h2>{{ activeFile ? activeFile.fileName : t('gitAssistant.detail.emptyTitle') }}</h2>
        <div v-if="activeFile" class="meta-row">
          <span class="meta-pill" :class="`tone-${statusMeta[activeFile.type].tone}`">
            {{ t(statusMeta[activeFile.type].labelKey) }}
          </span>
          <span class="meta-pill">{{ t('gitAssistant.detail.score', { score: activeFile.score }) }}</span>
          <span v-if="activeFile.recommended" class="meta-pill meta-pill--accent">
            {{ t('gitAssistant.detail.reviewRecommended') }}
          </span>
        </div>
      </div>

      <div v-if="activeFile && hasToggle" class="diff-switches">
        <button
          v-if="activeFile.staged"
          class="switch-btn"
          :class="{ active: currentMode === 'staged' }"
          type="button"
          @click="$emit('update:mode', 'staged')"
        >
          {{ t('gitAssistant.detail.stagedView') }}
        </button>
        <button
          v-if="activeFile.unstaged"
          class="switch-btn"
          :class="{ active: currentMode === 'unstaged' }"
          type="button"
          @click="$emit('update:mode', 'unstaged')"
        >
          {{ t('gitAssistant.detail.unstagedView') }}
        </button>
      </div>
    </header>

    <div v-if="!hasSnapshot" class="panel-empty">{{ t('gitAssistant.detail.emptyNoRepo') }}</div>
    <div v-else-if="!activeFile" class="panel-empty">{{ t('gitAssistant.detail.emptySelect') }}</div>
    <div v-else-if="loading" class="panel-empty">{{ t('gitAssistant.detail.diffLoading') }}</div>
    <div v-else-if="!diffText" class="panel-empty">{{ t('gitAssistant.detail.diffEmpty') }}</div>

    <div v-else class="diff-body mono">
      <div
        v-for="(line, index) in lines"
        :key="`${index}-${line}`"
        class="diff-line"
        :class="getLineClass(line)"
      >
        {{ line || ' ' }}
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useLocale } from '@/hooks/useLocale'
import { STATUS_META } from '../git-assistant.config'
import type { GitAssistantFileView } from '../git-assistant.types'

const props = defineProps<{
  hasSnapshot: boolean
  activeFile: GitAssistantFileView | null
  diffText: string
  loading: boolean
  currentMode: 'staged' | 'unstaged'
}>()

defineEmits<{
  (e: 'update:mode', value: 'staged' | 'unstaged'): void
}>()

const statusMeta = STATUS_META
const { t } = useLocale()

const lines = computed(() => props.diffText.split('\n'))
const hasToggle = computed(() => Boolean(props.activeFile?.staged && props.activeFile?.unstaged))

function getLineClass(line: string) {
  if (line.startsWith('+++') || line.startsWith('---')) return 'diff-line--file'
  if (line.startsWith('@@')) return 'diff-line--hunk'
  if (line.startsWith('+')) return 'diff-line--add'
  if (line.startsWith('-')) return 'diff-line--remove'
  return ''
}
</script>

<style scoped lang="scss">
.panel-shell {
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: 16px;
  box-shadow: var(--lumina-shadow-sm);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
  overflow: hidden;
}

.diff-header {
  align-items: flex-start;
  border-bottom: 1px solid var(--lumina-card-border);
  display: flex;
  gap: 12px;
  justify-content: space-between;
  padding: 12px 14px;
}

.panel-eyebrow {
  color: var(--lumina-text-secondary);
  font-size: 11px;
  margin-bottom: 4px;
}

h2 {
  font-size: 16px;
  margin: 0;
}

.meta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 6px;
}

.meta-pill,
.switch-btn {
  align-items: center;
  background: var(--lumina-surface-3);
  border: 1px solid var(--lumina-card-border);
  border-radius: 999px;
  display: inline-flex;
  font-size: 11px;
  height: 24px;
  padding: 0 8px;
}

.meta-pill--accent,
.switch-btn.active {
  background: var(--lumina-primary-soft);
  border-color: var(--lumina-primary);
  color: var(--lumina-primary);
}

.diff-switches {
  display: flex;
  gap: 8px;
}

.switch-btn {
  cursor: pointer;
  transition:
    background 0.18s ease,
    border-color 0.18s ease,
    color 0.18s ease;

  &:hover {
    background: var(--lumina-button-secondary-hover);
  }
}

.panel-empty {
  align-items: center;
  background: var(--lumina-empty-bg);
  border: 1px dashed var(--lumina-empty-border);
  border-radius: 12px;
  color: var(--lumina-text-secondary);
  display: flex;
  font-size: 12px;
  justify-content: center;
  margin: 14px;
  text-align: center;
}

.diff-body {
  background: var(--lumina-diff-bg);
  min-height: 0;
  overflow: auto;
  padding: 12px 0;
}

.diff-line {
  font-size: 12px;
  line-height: 1.7;
  padding: 0 14px;
  white-space: pre;
}

.diff-line--file {
  background: var(--lumina-diff-file-bg);
  color: var(--lumina-diff-file-text);
}

.diff-line--hunk {
  background: var(--lumina-diff-hunk-bg);
  color: var(--lumina-diff-hunk-text);
}

.diff-line--add {
  background: var(--lumina-diff-add-bg);
  color: var(--lumina-diff-add-text);
}

.diff-line--remove {
  background: var(--lumina-diff-remove-bg);
  color: var(--lumina-diff-remove-text);
}

.tone-warning {
  color: var(--lumina-warning);
}

.tone-success {
  color: var(--lumina-success);
}

.tone-danger {
  color: var(--lumina-danger);
}

.tone-info {
  color: var(--lumina-primary);
}

.mono {
  font-family: SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
}
</style>
