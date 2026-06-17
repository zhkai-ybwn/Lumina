<template>
  <section class="commit-panel panel-shell">
    <header class="commit-header">
      <div>
        <div class="panel-eyebrow">{{ t('gitAssistant.ai.title') }}</div>
        <h3>Commit Message</h3>
      </div>
      <div class="commit-readiness" :class="readinessTone">
        <span class="readiness-dot"></span>
        <span>{{ readinessLabel }}</span>
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

    <section class="sync-cockpit">
      <div class="cockpit-rail">
        <span class="rail-node rail-node--active"></span>
        <span class="rail-line"></span>
        <span class="rail-node" :class="{ 'rail-node--active': repositoryState?.hasCommits }"></span>
        <span class="rail-line"></span>
        <span class="rail-node" :class="{ 'rail-node--active': Boolean(repositoryState?.upstream && !repositoryState?.upstreamGone) }"></span>
      </div>
      <div class="cockpit-grid">
        <div class="sync-card">
          <span>{{ t('gitAssistant.sync.localBranch') }}</span>
          <strong>{{ branch || '-' }}</strong>
        </div>
        <div class="sync-card">
          <span>{{ t('gitAssistant.sync.remote') }}</span>
          <strong>{{ repositoryState?.remoteName || t('gitAssistant.sync.none') }}</strong>
        </div>
        <div class="sync-card" :class="{ warning: repositoryState?.upstreamGone }">
          <span>{{ t('gitAssistant.sync.upstream') }}</span>
          <strong>{{ upstreamText }}</strong>
        </div>
        <div class="sync-card sync-card--split">
          <span>{{ t('gitAssistant.sync.aheadBehind') }}</span>
          <strong>{{ repositoryState?.ahead ?? 0 }} / {{ repositoryState?.behind ?? 0 }}</strong>
        </div>
      </div>
      <p class="sync-advice">{{ syncAdvice }}</p>
    </section>

    <footer class="commit-actions">
      <div class="sync-actions">
        <button class="action-btn" :disabled="pullDisabled" type="button" @click="$emit('pull')">
          {{ pulling ? t('gitAssistant.ai.pulling') : t('gitAssistant.ai.pull') }}
        </button>
        <button class="action-btn" :disabled="pushDisabled" type="button" @click="$emit('push')">
          {{ pushing ? t('gitAssistant.ai.pushing') : pushLabel }}
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
import { computed } from 'vue'
import { useLocale } from '@/hooks/useLocale'
import type { GitRepositoryState } from '@/services/git/git-service'
import type { GitAssistantFileView } from '../git-assistant.types'

const props = defineProps<{
  generating: boolean
  generateDisabled: boolean
  committing: boolean
  pushing: boolean
  pulling: boolean
  submitDisabled: boolean
  title: string
  body: string
  branch: string
  repositoryState: GitRepositoryState | null
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

const upstreamText = computed(() => {
  if (!props.repositoryState?.upstream) return t('gitAssistant.sync.notSet')
  if (props.repositoryState.upstreamGone) return t('gitAssistant.sync.gone', { upstream: props.repositoryState.upstream })
  return props.repositoryState.upstream
})

const readinessTone = computed(() => {
  if (!props.repositoryState?.remoteName) return 'warning'
  if (!props.repositoryState.hasCommits) return 'draft'
  if (props.repositoryState.upstreamGone) return 'warning'
  return 'ready'
})

const readinessLabel = computed(() => {
  if (!props.repositoryState?.remoteName) return t('gitAssistant.sync.remoteMissing')
  if (!props.repositoryState.hasCommits) return t('gitAssistant.sync.firstCommit')
  if (props.repositoryState.upstreamGone) return t('gitAssistant.sync.upstreamGone')
  return t('gitAssistant.sync.ready')
})

const syncAdvice = computed(() => {
  if (!props.repositoryState?.remoteName) return t('gitAssistant.sync.adviceRemoteMissing')
  if (!props.repositoryState.hasCommits) return t('gitAssistant.sync.adviceFirstCommit')
  if (props.repositoryState.upstreamGone || !props.repositoryState.upstream) return t('gitAssistant.sync.adviceSetUpstream')
  if (props.repositoryState.behind > 0) return t('gitAssistant.sync.advicePullFirst')
  if (props.repositoryState.ahead > 0) return t('gitAssistant.sync.advicePushReady', { count: props.repositoryState.ahead })
  return t('gitAssistant.sync.adviceCleanSync')
})

const pushLabel = computed(() => {
  if (props.repositoryState?.hasCommits && (!props.repositoryState.upstream || props.repositoryState.upstreamGone)) {
    return t('gitAssistant.sync.publishBranch')
  }
  return t('gitAssistant.ai.push')
})

const pushDisabled = computed(() =>
  props.pushing || props.pulling || props.committing || !props.repositoryState?.hasCommits || !props.repositoryState.remoteName,
)
const pullDisabled = computed(() => props.pulling || props.pushing || props.committing || !props.repositoryState?.upstream || props.repositoryState.upstreamGone)
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
  grid-template-rows: auto auto auto auto auto;
  padding: 14px 14px 12px;

  > * {
    position: relative;
  }
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
  font-size: 17px;
  letter-spacing: 0;
  margin: 0;
}

.commit-readiness {
  align-items: center;
  background: color-mix(in srgb, var(--lumina-surface-2) 86%, transparent);
  border: 1px solid var(--lumina-card-border);
  border-radius: 7px;
  color: var(--lumina-text-secondary);
  display: inline-flex;
  font-size: 12px;
  gap: 7px;
  height: 30px;
  padding: 0 10px;

  &.ready {
    color: var(--lumina-primary);
  }

  &.warning {
    color: #b7791f;
  }
}

.readiness-dot {
  background: currentColor;
  border-radius: 999px;
  box-shadow: 0 0 0 4px color-mix(in srgb, currentColor 14%, transparent);
  height: 7px;
  width: 7px;
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
  min-height: 92px;
  resize: vertical;
}

.commit-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.meta-pill {
  align-items: center;
  background: color-mix(in srgb, var(--lumina-surface-3) 84%, transparent);
  border: 1px solid var(--lumina-card-border);
  border-radius: 7px;
  color: var(--lumina-text-secondary);
  display: inline-flex;
  font-size: 11px;
  height: 24px;
  padding: 0 8px;
}

.sync-cockpit {
  background: color-mix(in srgb, var(--lumina-surface-2) 58%, transparent);
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 78%, transparent);
  border-radius: 9px;
  display: grid;
  gap: 10px;
  grid-template-columns: 20px minmax(0, 1fr);
  padding: 10px;
}

.cockpit-rail {
  align-items: center;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-top: 4px;
}

.rail-node {
  background: var(--lumina-surface-3);
  border: 1px solid var(--lumina-card-border);
  border-radius: 999px;
  height: 8px;
  width: 8px;
}

.rail-node--active {
  background: var(--lumina-primary);
  border-color: var(--lumina-primary);
}

.rail-line {
  background: var(--lumina-card-border);
  flex: 1;
  min-height: 18px;
  width: 1px;
}

.cockpit-grid {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.sync-card {
  border-left: 2px solid color-mix(in srgb, var(--lumina-primary) 32%, var(--lumina-card-border));
  display: grid;
  gap: 5px;
  min-width: 0;
  padding-left: 9px;

  span {
    color: var(--lumina-text-secondary);
    font-size: 10px;
  }

  strong {
    color: var(--lumina-text);
    font-size: 12px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &.warning {
    border-left-color: #c47a20;
  }
}

.sync-card--split strong {
  font-family: Consolas, 'Liberation Mono', Menlo, monospace;
}

.sync-advice {
  color: var(--lumina-text-secondary);
  font-size: 11px;
  grid-column: 2;
  line-height: 1.45;
  margin: 0;
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

.sync-actions {
  display: flex;
  gap: 8px;
}

.primary {
  background: linear-gradient(180deg, color-mix(in srgb, var(--lumina-primary) 88%, #fff), var(--lumina-primary));
  border-color: var(--lumina-primary);
  box-shadow: 0 6px 14px color-mix(in srgb, var(--lumina-primary) 18%, transparent);
  color: #fff;
  min-width: 108px;
}

@media (max-width: 980px) {
  .cockpit-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
