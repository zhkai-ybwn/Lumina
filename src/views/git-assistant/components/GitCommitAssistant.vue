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

    <footer class="commit-actions">
      <span v-if="submitDisabled" class="hint-text">{{ t('gitAssistant.ai.submitDisabled') }}</span>
      <button class="action-btn primary" :disabled="submitDisabled || committing || pulling || pushing" type="button" @click="$emit('submit')">
        {{ committing ? t('gitAssistant.ai.submitting') : t('gitAssistant.ai.submit') }}
      </button>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { useLocale } from '@/hooks/useLocale'

defineProps<{
  committing: boolean
  pushing: boolean
  pulling: boolean
  submitDisabled: boolean
  title: string
  body: string
}>()

defineEmits<{
  (e: 'submit'): void
  (e: 'update:title', value: string): void
  (e: 'update:body', value: string): void
}>()

const { t } = useLocale()
</script>

<style scoped lang="scss">
.panel-shell {
  background: color-mix(in srgb, var(--lumina-surface-1) 88%, transparent);
  backdrop-filter: blur(18px);
  overflow: hidden;
  position: relative;

  &::before {
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.26), transparent 68%);
    content: '';
    inset: 0;
    pointer-events: none;
    position: absolute;
  }
}

.commit-panel {
  display: grid;
  gap: 12px;
  grid-template-rows: auto auto auto;
  padding: 14px 14px 12px;

  > * {
    position: relative;
  }
}

.commit-header,
.commit-actions {
  align-items: center;
  display: flex;
  gap: 10px;
}

.commit-header {
  justify-content: flex-start;
}

.commit-actions {
  justify-content: flex-end;
}

.panel-eyebrow,
.field-label,
.hint-text {
  color: var(--lumina-text-secondary);
  font-size: 11px;
}

.hint-text {
  margin-right: auto;
}

h3 {
  font-size: 17px;
  letter-spacing: 0;
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
  background: color-mix(in srgb, var(--lumina-input-bg) 92%, transparent);
  border: 1px solid var(--lumina-input-border);
  border-radius: 7px;
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
  min-height: 104px;
  resize: vertical;
}

.action-btn {
  background: color-mix(in srgb, var(--lumina-button-secondary-bg) 92%, transparent);
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 88%, var(--lumina-text-secondary));
  border-radius: 7px;
  color: var(--lumina-text);
  cursor: pointer;
  font-weight: 600;
  height: 32px;
  min-width: 86px;
  padding: 0 14px;
  transition:
    background 0.18s ease,
    border-color 0.18s ease,
    color 0.18s ease;

  &:hover:not(:disabled) {
    background: color-mix(in srgb, var(--lumina-button-secondary-hover) 82%, transparent);
  }

  &:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
}

.primary {
  background: linear-gradient(180deg, color-mix(in srgb, var(--lumina-primary) 88%, #fff), var(--lumina-primary));
  border-color: var(--lumina-primary);
  box-shadow: 0 6px 14px color-mix(in srgb, var(--lumina-primary) 18%, transparent);
  color: #fff;
  min-width: 108px;
}

</style>
