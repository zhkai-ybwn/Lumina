<template>
  <section class="status-bar">
    <div class="status-bar__main">
      <div class="repo-switcher" :title="repoPath || t('gitAssistant.repo.emptyPath')">
        <button class="repo-switcher__main" type="button" @click="$emit('manage-repos')">
          <span>{{ t('gitAssistant.repo.currentRepoShort') }}</span>
          <strong>{{ currentRecentLabel }}</strong>
        </button>
        <button class="repo-switcher__manage" type="button" :title="t('gitAssistant.repo.recentRepoManage')" @click="$emit('manage-repos')">
          <Icon icon="solar:settings-linear" />
        </button>
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
      <div v-if="summary.conflicted" class="metric-pill metric-pill--danger">
        <span>{{ t('gitAssistant.repo.summaryConflicted') }}</span>
        <strong>{{ summary.conflicted }}</strong>
      </div>
      <div class="metric-pill metric-pill--accent">
        <span>{{ t('gitAssistant.repo.summaryRecommended') }}</span>
        <strong>{{ recommendedCount }}</strong>
      </div>
    </div>

    <div class="status-bar__actions">
      <span class="sync-pill" :class="syncTone">
        <span class="sync-dot"></span>
        {{ syncLabel }}
      </span>
      <button class="tool-btn" :disabled="pullDisabled" @click="$emit('pull')">
        {{ pulling ? t('gitAssistant.ai.pulling') : t('gitAssistant.ai.pull') }}
      </button>
      <button class="tool-btn" :disabled="fetchDisabled" @click="$emit('fetch')">
        {{ fetching ? t('gitAssistant.ai.fetching') : t('gitAssistant.ai.fetch') }}
      </button>
      <button class="tool-btn" :disabled="pushDisabled" @click="$emit('push')">
        {{ pushing ? t('gitAssistant.ai.pushing') : t('gitAssistant.ai.push') }}
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
import type { GitRepositoryState } from '@/services/git/git-service'
import type { GitAssistantSummary } from '../git-assistant.types'

const props = defineProps<{
  repoPath: string
  branch: string
  loading: boolean
  fetching: boolean
  pushing: boolean
  pulling: boolean
  summary: GitAssistantSummary
  recommendedCount: number
  repositoryState: GitRepositoryState | null
  recentRepos: RecentGitRepo[]
}>()

defineEmits<{
  (e: 'pick-directory'): void
  (e: 'refresh'): void
  (e: 'fetch'): void
  (e: 'pull'): void
  (e: 'push'): void
  (e: 'manage-repos'): void
}>()

const { t } = useLocale()

const repoName = computed(() => {
  const normalized = props.repoPath.replace(/\\/g, '/')
  const parts = normalized.split('/').filter(Boolean)
  return parts[parts.length - 1] ?? ''
})

const currentRecentLabel = computed(() => {
  const current = props.recentRepos.find(repo => normalizePath(repo.path) === normalizePath(props.repoPath))
  return current?.name || repoName.value || t('gitAssistant.repo.recentRepoPlaceholder')
})

const syncLabel = computed(() => {
  const state = props.repositoryState
  if (!state?.remoteName) return t('gitAssistant.sync.remoteMissing')
  if (!state.hasCommits) return t('gitAssistant.sync.firstCommit')
  if (state.upstreamGone) return t('gitAssistant.sync.upstreamGone')
  if (!state.upstream) return t('gitAssistant.sync.notSet')
  if (state.ahead > 0 && state.behind > 0) return t('gitAssistant.sync.diverged', { ahead: state.ahead, behind: state.behind })
  if (state.ahead > 0) return t('gitAssistant.sync.ahead', { count: state.ahead })
  if (state.behind > 0) return t('gitAssistant.sync.behind', { count: state.behind })
  return t('gitAssistant.sync.syncedShort')
})

const syncTone = computed(() => {
  const state = props.repositoryState
  if (!state?.remoteName || !state.hasCommits || state.upstreamGone || !state.upstream) return 'warning'
  if (state.behind > 0) return 'danger'
  if (state.ahead > 0) return 'accent'
  return 'ready'
})

const pushDisabled = computed(() =>
  props.pushing || props.pulling || props.fetching || props.loading || !props.repositoryState?.hasCommits || !props.repositoryState.remoteName,
)
const pullDisabled = computed(() =>
  props.pulling || props.pushing || props.fetching || props.loading || !props.repositoryState?.upstream || props.repositoryState.upstreamGone,
)
const fetchDisabled = computed(() =>
  props.fetching || props.pulling || props.pushing || props.loading || !props.repositoryState?.remoteName,
)

export interface RecentGitRepo {
  path: string
  name: string
}

function normalizePath(path: string) {
  return path.replace(/\\/g, '/').toLowerCase()
}
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
.repo-switcher,
.metric-pill,
.tool-btn,
.sync-pill {
  border: 1px solid var(--lumina-card-border);
  border-radius: 8px;
  font-size: 11px;
  height: 28px;
}

.context-pill,
.metric-pill {
  align-items: center;
  background: color-mix(in srgb, var(--lumina-surface-3) 82%, transparent);
  display: inline-flex;
  gap: 8px;
  padding: 0 10px;
  white-space: nowrap;
}

.sync-pill {
  align-items: center;
  background: color-mix(in srgb, var(--lumina-surface-3) 82%, transparent);
  color: var(--lumina-text-secondary);
  display: inline-flex;
  gap: 7px;
  padding: 0 10px;
  white-space: nowrap;

  &.ready {
    color: var(--lumina-primary);
  }

  &.accent {
    background: var(--lumina-primary-soft);
    color: var(--lumina-primary);
  }

  &.warning {
    color: var(--lumina-warning);
  }

  &.danger {
    color: var(--lumina-danger);
  }
}

.sync-dot {
  background: currentColor;
  border-radius: 999px;
  height: 6px;
  width: 6px;
}

.context-pill {
  max-width: 240px;
  min-width: 0;
}

.repo-switcher {
  align-items: center;
  background: color-mix(in srgb, var(--lumina-surface-3) 82%, transparent);
  display: inline-flex;
  max-width: 320px;
  min-width: 230px;
  overflow: hidden;
  padding: 0 4px 0 0;
}

.repo-switcher__main {
  align-items: center;
  background: transparent;
  border: 0;
  color: inherit;
  cursor: pointer;
  display: inline-flex;
  flex: 1;
  gap: 8px;
  height: 100%;
  min-width: 0;
  padding: 0 6px 0 10px;
  text-align: left;

  span {
    color: var(--lumina-text-secondary);
    flex: 0 0 auto;
  }

  strong {
    flex: 1;
    font-weight: 500;
    min-width: 0;
    overflow: hidden;
    text-align: left;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &:hover {
    background: color-mix(in srgb, var(--lumina-button-secondary-hover) 70%, transparent);
  }
}

.repo-switcher__manage {
  align-items: center;
  background: transparent;
  border: 0;
  border-radius: 6px;
  color: var(--lumina-text-secondary);
  cursor: pointer;
  display: inline-flex;
  flex: 0 0 auto;
  height: 22px;
  justify-content: center;
  padding: 0;
  transition:
    background 0.18s ease,
    color 0.18s ease;
  width: 22px;

  :deep(svg) {
    height: 15px;
    width: 15px;
  }

  &:hover {
    background: color-mix(in srgb, var(--lumina-button-secondary-hover) 82%, transparent);
    color: var(--lumina-text);
  }
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

.metric-pill--danger {
  background: color-mix(in srgb, var(--lumina-danger) 12%, transparent);
  color: var(--lumina-danger);

  strong {
    color: var(--lumina-danger);
  }
}

.tool-btn {
  align-items: center;
  background: color-mix(in srgb, var(--lumina-button-secondary-bg) 92%, transparent);
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

  &:disabled {
    cursor: not-allowed;
    opacity: 0.52;
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
