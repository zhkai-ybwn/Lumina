<template>
  <section class="commit-panel panel-shell">
    <header class="commit-header">
      <div>
        <div class="panel-eyebrow">{{ t('gitAssistant.ai.title') }}</div>
        <h3>Commit Message</h3>
      </div>
    </header>

    <div class="commit-form">
      <label class="field">
        <span class="field-label">{{ t('gitAssistant.ai.editTitle') }}</span>
        <input
          :value="title"
          class="field-input"
          type="text"
          @input="$emit('update:title', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <label class="field field--body">
        <span class="field-label">{{ t('gitAssistant.ai.editBody') }}</span>
        <textarea
          :value="body"
          class="field-textarea"
          @input="$emit('update:body', ($event.target as HTMLTextAreaElement).value)"
        />
      </label>
    </div>

    <div class="commit-meta">
      <span class="meta-pill">{{ t('gitAssistant.ai.reviewSelected', { count: selectedCount }) }}</span>
      <span class="meta-pill">
        {{
          recommendedFiles.length
            ? `${t('gitAssistant.ai.reviewRecommended')} ${recommendedFiles.length}`
            : t('gitAssistant.ai.reviewNone')
        }}
      </span>
      <span class="meta-pill">{{ statusText }}</span>
    </div>

    <footer class="commit-actions">
      <div class="sync-actions">
        <button class="action-btn" :disabled="pulling || pushing || committing" type="button" @click="$emit('pull')">
          {{ pulling ? t('gitAssistant.ai.pulling') : t('gitAssistant.ai.pull') }}
        </button>
        <button class="action-btn" :disabled="pushing || pulling || committing" type="button" @click="$emit('push')">
          {{ pushing ? t('gitAssistant.ai.pushing') : t('gitAssistant.ai.push') }}
        </button>
      </div>
      <button class="action-btn primary" :disabled="submitDisabled || committing || pulling || pushing" type="button" @click="$emit('submit')">
        {{ committing ? t('gitAssistant.ai.submitting') : t('gitAssistant.ai.submit') }}
      </button>
      <span v-if="submitDisabled" class="hint-text">{{ t('gitAssistant.ai.submitDisabled') }}</span>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { useLocale } from '@/hooks/useLocale'
import type { GitAssistantFileView } from '../git-assistant.types'

defineProps<{
  generating: boolean
  generateDisabled: boolean
  committing: boolean
  pushing: boolean
  pulling: boolean
  submitDisabled: boolean
  title: string
  body: string
  selectedCount: number
  recommendedFiles: GitAssistantFileView[]
  statusText: string
}>()

defineEmits<{
  (e: 'submit'): void
  (e: 'push'): void
  (e: 'pull'): void
  (e: 'update:title', value: string): void
  (e: 'update:body', value: string): void
}>()

const { t } = useLocale()
</script>

<style scoped lang="scss">
.panel-shell {
  background: var(--lumina-surface-2);
  border-top: 1px solid var(--lumina-card-border);
}

.commit-panel {
  display: grid;
  gap: 10px;
  grid-template-rows: auto auto auto auto;
  padding: 12px;
}

.commit-header,
.commit-actions {
  align-items: flex-start;
  display: flex;
  gap: 10px;
  justify-content: space-between;
}

.panel-eyebrow,
.field-label,
.hint-text {
  color: var(--lumina-text-secondary);
  font-size: 11px;
}

h3 {
  font-size: 15px;
  margin: 0;
}

.commit-form {
  display: grid;
  gap: 8px;
}

.field {
  display: grid;
  gap: 6px;
}

.field-input,
.field-textarea {
  background: var(--lumina-input-bg);
  border: 1px solid var(--lumina-input-border);
  border-radius: 10px;
  box-sizing: border-box;
  color: var(--lumina-text);
  padding: 10px 12px;
  width: 100%;

  &:focus {
    border-color: var(--lumina-input-focus);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--lumina-input-focus) 18%, transparent);
    outline: none;
  }
}

.field-textarea {
  min-height: 84px;
  resize: vertical;
}

.commit-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.meta-pill {
  align-items: center;
  background: var(--lumina-surface-3);
  border: 1px solid var(--lumina-card-border);
  border-radius: 999px;
  color: var(--lumina-text-secondary);
  display: inline-flex;
  font-size: 11px;
  height: 24px;
  padding: 0 8px;
}

.action-btn {
  background: var(--lumina-button-secondary-bg);
  border: 1px solid var(--lumina-card-border);
  border-radius: 10px;
  color: var(--lumina-text);
  cursor: pointer;
  height: 34px;
  padding: 0 12px;
  transition:
    background 0.18s ease,
    border-color 0.18s ease,
    color 0.18s ease;

  &:hover:not(:disabled) {
    background: var(--lumina-button-secondary-hover);
  }

  &:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
}

.sync-actions {
  display: flex;
  gap: 8px;
}

.primary {
  background: var(--lumina-primary);
  border-color: var(--lumina-primary);
  color: #fff;
}
</style>
