<template>
  <div class="git-assistant-page">
    <div v-if="error" class="error-banner">
      {{ error }}
    </div>

    <GitStatusBar
      :repo-path="displayRepoPath"
      :branch="snapshot?.branch ?? ''"
      :loading="loading"
      :pushing="pushLoading"
      :pulling="pullLoading"
      :summary="summary"
      :recommended-count="recommendedFiles.length"
      :repository-state="snapshot?.repositoryState ?? null"
      :recent-repos="recentRepos"
      @pick-directory="handlePickDirectory"
      @refresh="handleRefresh"
      @push="handlePush"
      @pull="handlePull"
      @manage-repos="recentRepoManagerOpen = true"
    />

    <section class="workspace-body">
      <section class="commit-area">
        <GitCommitAssistant
          class="commit-workbench"
          :committing="commitLoading"
          :pushing="pushLoading"
          :pulling="pullLoading"
          :submit-disabled="!snapshot || !reviewSelectedRaws.length || !commitTitle.trim()"
          :title="commitTitle"
          :body="commitBody"
          @submit="handleCommit"
          @update:title="commitTitle = $event"
          @update:body="commitBody = $event"
        />

        <aside class="commit-side">
          <div class="ai-panel-title">
            <span>{{ t('gitAssistant.ai.actionsTitle') }}</span>
          </div>

          <label class="model-field">
            <span>{{ t('gitAssistant.ai.currentModel') }}</span>
            <NSelect
              class="model-select"
              :value="aiSettings.taskModelMap['commit-message'] || aiSettings.defaultModelId"
              :options="modelSelectOptions"
              :disabled="!aiSettings.enabledModels.length"
              size="small"
              :consistent-menu-width="false"
              @update:value="value => aiSettings.setTaskModel('commit-message', String(value ?? ''))"
            />
          </label>

          <NCheckbox v-model:checked="autoSendPromptToApi" class="ai-toggle">
            {{ t('gitAssistant.ai.autoSendPrompt') }}
          </NCheckbox>

          <section v-if="showRemoteTools" class="remote-tools">
            <div class="remote-tools__header">
              <span>{{ t('gitAssistant.remote.title') }}</span>
              <strong>{{ remoteToolStatus }}</strong>
            </div>
            <NInput
              v-if="needsRemoteUrl"
              v-model:value="remoteUrlDraft"
              size="small"
              clearable
              :placeholder="t('gitAssistant.remote.urlPlaceholder')"
            />
            <div class="remote-actions">
              <NButton
                v-if="needsRemoteUrl"
                size="small"
                type="primary"
                :disabled="!remoteUrlDraft.trim() || remoteLoading"
                @click="handleConfigureOrigin"
              >
                {{ t('gitAssistant.remote.connectOrigin') }}
              </NButton>
              <NButton
                v-if="canRepairUpstream"
                size="small"
                :disabled="remoteLoading"
                @click="handleRepairUpstream"
              >
                {{ t('gitAssistant.remote.repairUpstream') }}
              </NButton>
              <NButton
                v-if="canPublishBranch"
                size="small"
                :disabled="pushLoading || remoteLoading"
                @click="handlePush"
              >
                {{ t('gitAssistant.remote.publishBranch') }}
              </NButton>
            </div>
            <p>{{ remoteToolHint }}</p>
          </section>

          <div class="ai-actions">
            <button class="ai-action primary-action" type="button" :disabled="!snapshot || aiLoading" @click="handleGenerateAiAnalysis">
              {{ aiLoading ? t('gitAssistant.ai.generating') : t('gitAssistant.ai.generate') }}
            </button>
            <button class="ai-action" type="button" :disabled="!promptPreview" @click="promptDrawerOpen = true">
              {{ t('gitAssistant.ai.viewPrompt') }}
            </button>
            <button class="ai-action" type="button" :disabled="!filteredCommitMessageHistory.length" @click="historyDrawerOpen = true">
              {{ t('gitAssistant.history.open') }}
            </button>
          </div>
          <div v-if="aiLoading" class="ai-progress">
            <span class="ai-progress__dot"></span>
            <span>{{ promptGenerationStep }}</span>
          </div>
          <button class="ai-action" type="button" disabled>
            {{ t('gitAssistant.ai.reviewCode') }}
          </button>
        </aside>
      </section>

      <GitChangeExplorer
        class="change-table"
        :has-snapshot="Boolean(snapshot)"
        :loading="loading"
        :keyword="keyword"
        :status-filter="statusFilter"
        :recommended-only="recommendedOnly"
        :summary="summary"
        :groups="filteredFileGroups"
        :filtered-count="filteredFiles.length"
        :total-count="allFiles.length"
        :active-file-raw="activeFileRaw"
        :review-selected-raws="reviewSelectedRaws"
        @update:keyword="keyword = $event"
        @update:status-filter="handleStatusFilterChange"
        @update:recommended-only="recommendedOnly = $event"
        @select-file="handleSelectFile"
        @open-diff="handleOpenDiff"
        @toggle-review-selection="toggleReviewSelection"
        @set-review-selection="setReviewSelection"
      />

      <NModal v-model:show="showDiff" class="diff-modal" :mask-closable="true">
        <section class="diff-dialog">
          <button class="modal-close-button" type="button" :aria-label="t('gitAssistant.prompt.close')" @click="showDiff = false">
            &times;
          </button>
          <GitDiffViewer
            class="diff-window"
            :has-snapshot="Boolean(snapshot)"
            :active-file="selectedFile"
            :diff-text="currentDiff"
            :loading="diffLoading"
            :current-mode="diffMode"
            @update:mode="diffMode = $event"
          />
        </section>
      </NModal>

      <NModal v-model:show="recentRepoManagerOpen" class="recent-repo-modal" :mask-closable="true">
        <section class="recent-repo-dialog">
          <button class="modal-close-button" type="button" :aria-label="t('gitAssistant.prompt.close')" @click="recentRepoManagerOpen = false">
            &times;
          </button>
          <header class="recent-repo-dialog__header">
            <div>
              <h3>{{ t('gitAssistant.repo.recentRepoManage') }}</h3>
              <p>{{ t('gitAssistant.repo.recentRepoManageHint') }}</p>
            </div>
          </header>

          <section v-if="recentRepos.length" class="recent-repo-list">
            <article v-for="repo in recentRepos" :key="repo.path" class="recent-repo-item">
              <div class="recent-repo-item__main">
                <NInput
                  :value="repo.name"
                  size="small"
                  :placeholder="t('gitAssistant.repo.recentRepoAliasPlaceholder')"
                  @update:value="value => renameRecentRepo(repo.path, String(value ?? ''))"
                />
                <div class="recent-repo-path mono" :title="repo.path">{{ repo.path }}</div>
              </div>
              <div class="recent-repo-item__actions">
                <NButton size="small" :disabled="normalizePath(repo.path) === normalizePath(displayRepoPath)" @click="handleSwitchRecentRepoFromManager(repo.path)">
                  {{ t('gitAssistant.repo.recentRepoSwitch') }}
                </NButton>
                <NButton size="small" quaternary @click="removeRecentRepo(repo.path)">
                  {{ t('gitAssistant.repo.recentRepoRemove') }}
                </NButton>
              </div>
            </article>
          </section>
          <div v-else class="recent-repo-empty">
            {{ t('gitAssistant.repo.recentRepoEmpty') }}
          </div>
        </section>
      </NModal>
    </section>

    <aside v-if="promptDrawerOpen" class="prompt-drawer">
      <header class="prompt-drawer__header">
        <div>
          <h3>{{ t('gitAssistant.prompt.title') }}</h3>
          <p>{{ t('gitAssistant.prompt.description') }}</p>
        </div>
        <button type="button" @click="promptDrawerOpen = false">{{ t('gitAssistant.prompt.close') }}</button>
      </header>

      <div v-if="promptPreview" class="prompt-drawer__body">
        <details class="prompt-section" open>
          <summary>{{ t('gitAssistant.prompt.overview') }}</summary>
          <section class="prompt-stats">
            <div>
              <span>{{ t('gitAssistant.prompt.selectedFiles') }}</span>
              <strong>{{ promptPreview.trace.selectedFiles.length }}</strong>
            </div>
            <div>
              <span>{{ t('gitAssistant.prompt.rawChars') }}</span>
              <strong>{{ promptPreview.trace.rawChars }}</strong>
            </div>
            <div>
              <span>{{ t('gitAssistant.prompt.cleanedChars') }}</span>
              <strong>{{ promptPreview.trace.cleanedChars }}</strong>
            </div>
            <div>
              <span>{{ t('gitAssistant.prompt.evidenceCount') }}</span>
              <strong>{{ promptPreview.trace.evidenceCount }}</strong>
            </div>
          </section>
        </details>

        <details class="prompt-section" open>
          <summary>{{ t('gitAssistant.prompt.rules') }}</summary>
          <section class="prompt-rules">
            <p>{{ t('gitAssistant.prompt.rulesHint') }}</p>
            <ol>
              <li v-for="rule in promptPreview.trace.rules" :key="rule">{{ rule }}</li>
            </ol>
          </section>
        </details>

        <details class="prompt-section" open>
          <summary>{{ t('gitAssistant.prompt.files') }}</summary>
          <section class="prompt-files">
            <div v-for="group in promptFileGroups" :key="group.kind" class="prompt-file-group">
              <div class="prompt-file-group__title">
                <strong>{{ group.kind }}</strong>
                <span>{{ group.files.length }}</span>
              </div>
              <div class="prompt-file-table">
                <div class="prompt-file-table__head">
                  <span>{{ t('gitAssistant.prompt.columnPath') }}</span>
                  <span>{{ t('gitAssistant.prompt.columnRole') }}</span>
                  <span>{{ t('gitAssistant.prompt.columnScope') }}</span>
                  <span>{{ t('gitAssistant.prompt.columnStrategy') }}</span>
                  <span>{{ t('gitAssistant.prompt.columnEvidence') }}</span>
                  <span>{{ t('gitAssistant.prompt.columnChars') }}</span>
                  <span>{{ t('gitAssistant.prompt.columnReason') }}</span>
                </div>
                <div v-for="file in group.files" :key="file.path" class="prompt-file-table__row">
                  <span class="mono" :title="file.path">{{ file.path }}</span>
                  <span>{{ file.role }}</span>
                  <span>{{ file.scope }}</span>
                  <span>{{ file.strategy }}</span>
                  <span>{{ file.evidenceCount }}</span>
                  <span>{{ file.cleanedChars }} / {{ file.rawChars }}</span>
                  <span :title="file.reason || ''">{{ file.reason || '-' }}</span>
                </div>
              </div>
            </div>
          </section>
        </details>

        <details class="prompt-section" open>
          <summary>{{ t('gitAssistant.prompt.finalPrompt') }}</summary>
          <section class="prompt-text">
            <textarea :value="promptPreview.prompt" readonly spellcheck="false"></textarea>
          </section>
        </details>
      </div>
    </aside>

    <aside v-if="historyDrawerOpen" class="history-drawer">
      <header class="history-drawer__header">
        <div>
          <h3>{{ t('gitAssistant.history.title') }}</h3>
          <p>{{ t('gitAssistant.history.description') }}</p>
        </div>
        <button type="button" @click="historyDrawerOpen = false">{{ t('gitAssistant.prompt.close') }}</button>
      </header>

      <section class="history-list">
        <article v-for="entry in filteredCommitMessageHistory" :key="entry.id" class="history-item">
          <div class="history-item__main">
            <span>{{ formatHistoryTime(entry.createdAt) }} · {{ historySourceLabel(entry.source) }}</span>
            <strong>{{ entry.title }}</strong>
            <p v-if="entry.body">{{ entry.body }}</p>
            <small>{{ entry.repoName }} · {{ t('gitAssistant.history.fileCount', { count: entry.selectedFileCount }) }}</small>
          </div>
          <button type="button" @click="restoreCommitMessage(entry)">{{ t('gitAssistant.history.restore') }}</button>
        </article>
        <div v-if="!filteredCommitMessageHistory.length" class="history-empty">
          {{ t('gitAssistant.history.empty') }}
        </div>
      </section>
    </aside>

    <GitCommandDialog
      :visible="gitCommandDialog.visible"
      :title="gitCommandDialog.title"
      :repo-path="displayRepoPath"
      :phase="gitCommandDialog.phase"
      :running="gitCommandDialog.running"
      :success="gitCommandDialog.success"
      :command="gitCommandDialog.command"
      :stdout="gitCommandDialog.stdout"
      :stderr="gitCommandDialog.stderr"
      :message="gitCommandDialog.message"
      :suggestion="gitCommandDialog.suggestion"
      :next-action-label="gitCommandDialog.nextActionLabel"
      :close-label="t('gitAssistant.gitCommand.close')"
      :abort-label="t('gitAssistant.gitCommand.abort')"
      @close="gitCommandDialog.visible = false"
      @next-action="handleCommandNextAction"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { NButton, NCheckbox, NInput, NModal, NSelect } from 'naive-ui'
import { useLocale } from '@/hooks/useLocale'
import {
  buildGitCommitPrompt,
  generateGitAiAnalysisFromPrompt,
  type GitCommitPromptFileTrace,
  type GitCommitPromptPreview,
} from '@/services/git/git-ai-service'
import {
  commitGitChanges,
  configureGitOrigin,
  type GitCommandResult,
  loadGitFileDiff,
  loadGitSnapshot,
  pullGitChanges,
  pushGitChanges,
  repairGitUpstream,
  type GitSnapshot,
} from '@/services/git/git-service'
import { ensureGitProjectProfile } from '@/services/git/git-profile-service'
import { useAiSettingsStore } from '@/stores/ai-settings'
import type { GitFileStatus } from '@/types/git'
import { parseGitStatusList } from '@/utils/git-status'
import GitChangeExplorer from './components/GitChangeExplorer.vue'
import GitCommandDialog from './components/GitCommandDialog.vue'
import GitCommitAssistant from './components/GitCommitAssistant.vue'
import GitDiffViewer from './components/GitDiffViewer.vue'
import GitStatusBar from './components/GitStatusBar.vue'
import {
  ATTENTION_SCORE_CONFIG,
  GIT_COMMIT_MESSAGE_HISTORY_STORAGE_KEY,
  GIT_RECENT_REPOS_STORAGE_KEY,
  GIT_REPO_STORAGE_KEY,
} from './git-assistant.config'
import type {
  GitAssistantFileGroup,
  GitAssistantFileView,
  GitAssistantStatusFilter,
} from './git-assistant.types'

const loading = ref(false)
const diffLoading = ref(false)
const aiLoading = ref(false)
const commitLoading = ref(false)
const pushLoading = ref(false)
const pullLoading = ref(false)
const remoteLoading = ref(false)
const error = ref('')
const repoPath = ref('')
const snapshot = ref<GitSnapshot | null>(null)
const recentRepos = ref<RecentGitRepo[]>([])
const remoteUrlDraft = ref('')

const keyword = ref('')
const statusFilter = ref<GitAssistantStatusFilter>('all')
const recommendedOnly = ref(false)
const activeFileRaw = ref<string | null>(null)
const reviewSelectedRaws = ref<string[]>([])

const currentDiff = ref('')
const diffMode = ref<'staged' | 'unstaged'>('unstaged')
const showDiff = ref(false)

const commitTitle = ref('')
const commitBody = ref('')
const promptPreview = ref<GitCommitPromptPreview | null>(null)
const promptDrawerOpen = ref(false)
const historyDrawerOpen = ref(false)
const recentRepoManagerOpen = ref(false)
const promptGenerationStep = ref('')
const autoSendPromptToApi = ref(true)
let promptProgressTimers: number[] = []
const MAX_RECENT_REPOS = 8
const MAX_COMMIT_MESSAGE_HISTORY = 20

const gitCommandDialog = ref({
  visible: false,
  title: '',
  phase: '',
  running: false,
  success: null as boolean | null,
  command: '',
  stdout: '',
  stderr: '',
  message: '',
  suggestion: '',
  nextActionLabel: '',
  nextAction: '' as '' | 'push' | 'pull',
})

const { t } = useLocale()
const aiSettings = useAiSettingsStore()

const parsedFiles = computed<GitFileStatus[]>(() => parseGitStatusList(snapshot.value?.status ?? []))

const allFiles = computed<GitAssistantFileView[]>(() => {
  const statsByPath = new Map((snapshot.value?.fileStats ?? []).map(stat => [normalizePath(stat.path), stat]))

  return parsedFiles.value.map(file => {
    const directory = getDirName(file.path)
    const score = scoreFileAttention(file)
    const stats = statsByPath.get(normalizePath(file.path))

    return {
      ...file,
      fileName: getFileName(file.path),
      directory,
      extension: getFileExtension(file.path),
      addedLines: stats?.added ?? null,
      removedLines: stats?.removed ?? null,
      score,
      recommended: score >= ATTENTION_SCORE_CONFIG.recommendedThreshold,
    }
  })
})

const summary = computed(() => {
  const files = parsedFiles.value

  return {
    total: files.length,
    modified: files.filter(file => file.type === 'modified').length,
    added: files.filter(file => file.type === 'added').length,
    deleted: files.filter(file => file.type === 'deleted').length,
    renamed: files.filter(file => file.type === 'renamed').length,
    copied: files.filter(file => file.type === 'copied').length,
    untracked: files.filter(file => file.type === 'untracked').length,
    staged: files.filter(file => file.staged).length,
    unstaged: files.filter(file => file.unstaged).length,
  }
})

const displayRepoPath = computed(() => repoPath.value || snapshot.value?.repoPath || '')
const repositoryState = computed(() => snapshot.value?.repositoryState ?? null)

const recommendedFiles = computed(() => {
  return [...allFiles.value].filter(file => file.recommended).sort((left, right) => right.score - left.score)
})

const promptFileGroups = computed(() => {
  if (!promptPreview.value) {
    return []
  }

  const groups = new Map<string, GitCommitPromptFileTrace[]>()
  for (const file of promptPreview.value.trace.selectedFiles) {
    const key = file.kind || 'other'
    groups.set(key, [...(groups.get(key) ?? []), file])
  }

  return Array.from(groups.entries())
    .map(([kind, files]) => ({ kind, files }))
    .sort((left, right) => right.files.length - left.files.length || left.kind.localeCompare(right.kind))
})

const filteredFiles = computed(() => {
  let files = [...allFiles.value]

  if (statusFilter.value !== 'all') {
    if (statusFilter.value === 'staged') {
      files = files.filter(file => file.staged)
    } else if (statusFilter.value === 'unstaged') {
      files = files.filter(file => file.unstaged)
    } else if (statusFilter.value === 'versioned') {
      files = files.filter(file => file.type !== 'untracked')
    } else if (statusFilter.value === 'recommended') {
      files = files.filter(file => file.recommended)
    } else {
      files = files.filter(file => file.type === statusFilter.value)
    }
  }

  if (recommendedOnly.value) {
    files = files.filter(file => file.recommended)
  }

  const normalizedKeyword = keyword.value.trim().toLowerCase()
  if (normalizedKeyword) {
    files = files.filter(file =>
      `${file.path} ${file.fileName} ${file.directory}`.toLowerCase().includes(normalizedKeyword),
    )
  }

  return files.sort((left, right) => {
    if (right.score !== left.score) return right.score - left.score
    return left.path.localeCompare(right.path)
  })
})

const filteredFileGroups = computed<GitAssistantFileGroup[]>(() => {
  return [
    {
      key: 'all-files',
      label: recommendedOnly.value ? t('gitAssistant.files.groupRecommended') : t('gitAssistant.files.groupAll'),
      files: filteredFiles.value,
    },
  ]
})

const selectedFile = computed(() => {
  if (!activeFileRaw.value) return filteredFiles.value[0] ?? allFiles.value[0] ?? null
  return allFiles.value.find(file => file.raw === activeFileRaw.value) ?? filteredFiles.value[0] ?? null
})

const suggestedCommitTitle = computed(() => {
  if (!summary.value.total) return ''

  const parts: string[] = []
  if (summary.value.modified) parts.push(`${summary.value.modified} modified`)
  if (summary.value.added) parts.push(`${summary.value.added} added`)
  if (summary.value.deleted) parts.push(`${summary.value.deleted} deleted`)
  if (summary.value.untracked) parts.push(`${summary.value.untracked} new`)

  return `chore: update workspace changes (${parts.join(', ')})`
})

const modelSelectOptions = computed(() => {
  if (!aiSettings.enabledModels.length) {
    return [{ label: t('gitAssistant.ai.noModelConfigured'), value: '' }]
  }

  return aiSettings.enabledModels.map(model => ({
    label: model.name,
    value: model.id,
  }))
})
const filteredCommitMessageHistory = computed(() => {
  const currentRepo = normalizePath(displayRepoPath.value).toLowerCase()
  return commitMessageHistory.value.filter(entry => !currentRepo || normalizePath(entry.repoPath).toLowerCase() === currentRepo)
})
const needsRemoteUrl = computed(() => Boolean(snapshot.value && !repositoryState.value?.remoteName))
const canRepairUpstream = computed(() => {
  const state = repositoryState.value
  return Boolean(state?.remoteName && state.hasCommits && (state.upstreamGone || !state.upstream))
})
const canPublishBranch = computed(() => {
  const state = repositoryState.value
  return Boolean(state?.remoteName && state.hasCommits && (!state.upstream || state.upstreamGone))
})
const showRemoteTools = computed(() => needsRemoteUrl.value || canRepairUpstream.value || canPublishBranch.value)
const remoteToolStatus = computed(() => {
  if (needsRemoteUrl.value) return t('gitAssistant.remote.missingOrigin')
  if (repositoryState.value?.upstreamGone) return t('gitAssistant.remote.upstreamGone')
  if (!repositoryState.value?.upstream) return t('gitAssistant.remote.upstreamMissing')
  return t('gitAssistant.remote.ready')
})
const remoteToolHint = computed(() => {
  if (needsRemoteUrl.value) return t('gitAssistant.remote.originHint')
  if (repositoryState.value?.upstreamGone) return t('gitAssistant.remote.upstreamGoneHint')
  if (!repositoryState.value?.upstream) return t('gitAssistant.remote.upstreamMissingHint')
  return t('gitAssistant.remote.readyHint')
})

interface RecentGitRepo {
  path: string
  name: string
  openedAt: number
}

interface CommitMessageHistoryEntry {
  id: string
  repoPath: string
  repoName: string
  title: string
  body: string
  source: 'ai' | 'manual'
  selectedFileCount: number
  createdAt: number
}

const commitMessageHistory = ref<CommitMessageHistoryEntry[]>([])

watch(
  filteredFiles,
  files => {
    const currentExists = files.some(file => file.raw === activeFileRaw.value)

    if (files.length === 0) {
      activeFileRaw.value = null
      currentDiff.value = ''
      return
    }

    if (!currentExists) {
      activeFileRaw.value = files[0]?.raw ?? null
    }
  },
  { immediate: true },
)

watch(
  allFiles,
  files => {
    const fileSet = new Set(files.map(file => file.raw))
    reviewSelectedRaws.value = reviewSelectedRaws.value.filter(raw => fileSet.has(raw))
  },
  { immediate: true },
)

watch(
  suggestedCommitTitle,
  value => {
    if (!commitTitle.value.trim()) {
      commitTitle.value = value
    }
  },
  { immediate: true },
)

watch(
  selectedFile,
  file => {
    if (!file) {
      diffMode.value = 'unstaged'
      return
    }

    if (file.unstaged) {
      diffMode.value = 'unstaged'
    } else if (file.staged) {
      diffMode.value = 'staged'
    }
  },
  { immediate: true },
)

watch([selectedFile, diffMode, showDiff], async ([file, mode, visible]) => {
  if (!visible || !file || !displayRepoPath.value) {
    currentDiff.value = ''
    return
  }

  const staged = mode === 'staged'
  if (staged && !file.staged) {
    currentDiff.value = ''
    return
  }

  if (!staged && !file.unstaged) {
    currentDiff.value = ''
    return
  }

  diffLoading.value = true
  try {
    const result = await loadGitFileDiff({
      repoPath: displayRepoPath.value,
      filePath: file.path,
      staged,
    })
    currentDiff.value = result.diff
  } catch (err) {
    console.error(err)
    currentDiff.value = ''
    error.value = err instanceof Error ? err.message : t('gitAssistant.errorFallback')
  } finally {
    diffLoading.value = false
  }
}, { immediate: true })

function getFileName(path: string) {
  const normalized = normalizePath(path)
  const parts = normalized.split('/')
  return parts[parts.length - 1] || path
}

function getDirName(path: string) {
  const normalized = normalizePath(path)
  const index = normalized.lastIndexOf('/')
  return index === -1 ? '' : normalized.slice(0, index)
}

function normalizePath(path: string) {
  return path.replace(/\\/g, '/')
}

function getFileExtension(path: string) {
  const name = getFileName(path)
  const index = name.lastIndexOf('.')
  return index === -1 ? '' : name.slice(index + 1).toLowerCase()
}

function scoreFileAttention(file: GitFileStatus) {
  let score = 1

  if (file.type === 'deleted' || file.type === 'renamed' || file.type === 'updated-but-unmerged') score += 4
  else if (file.type === 'modified') score += 3
  else if (file.type === 'added') score += 2

  if (file.staged) score += 1
  if (file.unstaged && file.staged) score += 1

  const normalizedPath = file.path.replace(/\\/g, '/').toLowerCase()
  const extension = getFileExtension(file.path)

  if (ATTENTION_SCORE_CONFIG.highRiskDirectories.some(directory => normalizedPath.startsWith(directory))) score += 2
  if (ATTENTION_SCORE_CONFIG.highRiskFiles.some(name => normalizedPath.endsWith(name.toLowerCase()))) score += 2
  if (ATTENTION_SCORE_CONFIG.configExtensions.includes(extension)) score += 1
  if (ATTENTION_SCORE_CONFIG.docsExtensions.includes(extension)) score -= 1

  return Math.max(score, 1)
}

async function loadSnapshotByPath(path: string) {
  if (!path) return

  loading.value = true
  error.value = ''
  currentDiff.value = ''

  try {
    const result = await loadGitSnapshot(path)
    snapshot.value = result
    remoteUrlDraft.value = result.repositoryState.remoteUrl ?? remoteUrlDraft.value
    reviewSelectedRaws.value = []
    localStorage.setItem(GIT_REPO_STORAGE_KEY, path)
    rememberRecentRepo(result.repoRoot || path)

    try {
      await ensureGitProjectProfile(result.repoRoot || path)
    } catch (profileErr) {
      console.error(profileErr)
      error.value = profileErr instanceof Error ? profileErr.message : t('gitAssistant.errorFallback')
    }
  } catch (err) {
    console.error(err)
    error.value = err instanceof Error ? err.message : t('gitAssistant.errorFallback')
  } finally {
    loading.value = false
  }
}

function getRepoDisplayName(path: string) {
  const normalized = normalizePath(path)
  const parts = normalized.split('/').filter(Boolean)
  return parts[parts.length - 1] || path
}

function loadRecentRepos() {
  try {
    const raw = localStorage.getItem(GIT_RECENT_REPOS_STORAGE_KEY)
    if (!raw) {
      recentRepos.value = []
      return
    }

    const parsed = JSON.parse(raw) as Partial<RecentGitRepo>[]
    recentRepos.value = parsed
      .filter(repo => typeof repo.path === 'string' && repo.path.trim())
      .map(repo => ({
        path: repo.path as string,
        name: typeof repo.name === 'string' && repo.name ? repo.name : getRepoDisplayName(repo.path as string),
        openedAt: typeof repo.openedAt === 'number' ? repo.openedAt : 0,
      }))
      .sort((left, right) => right.openedAt - left.openedAt)
      .slice(0, MAX_RECENT_REPOS)
  } catch (err) {
    console.error(err)
    recentRepos.value = []
  }
}

function persistRecentRepos() {
  localStorage.setItem(GIT_RECENT_REPOS_STORAGE_KEY, JSON.stringify(recentRepos.value))
}

function rememberRecentRepo(path: string) {
  const normalizedPath = path.trim()
  if (!normalizedPath) return

  const existingRepo = recentRepos.value.find(repo => normalizePath(repo.path).toLowerCase() === normalizePath(normalizedPath).toLowerCase())
  const nextRepo: RecentGitRepo = {
    path: normalizedPath,
    name: existingRepo?.name || getRepoDisplayName(normalizedPath),
    openedAt: Date.now(),
  }
  recentRepos.value = [
    nextRepo,
    ...recentRepos.value.filter(repo => normalizePath(repo.path).toLowerCase() !== normalizePath(normalizedPath).toLowerCase()),
  ].slice(0, MAX_RECENT_REPOS)
  persistRecentRepos()
}

function renameRecentRepo(path: string, name: string) {
  const normalizedPath = normalizePath(path).toLowerCase()
  recentRepos.value = recentRepos.value.map(repo =>
    normalizePath(repo.path).toLowerCase() === normalizedPath
      ? { ...repo, name: name.trim() || getRepoDisplayName(repo.path) }
      : repo,
  )
  persistRecentRepos()
}

function removeRecentRepo(path: string) {
  const normalizedPath = normalizePath(path).toLowerCase()
  recentRepos.value = recentRepos.value.filter(repo => normalizePath(repo.path).toLowerCase() !== normalizedPath)
  persistRecentRepos()
}

function loadCommitMessageHistory() {
  try {
    const raw = localStorage.getItem(GIT_COMMIT_MESSAGE_HISTORY_STORAGE_KEY)
    if (!raw) {
      commitMessageHistory.value = []
      return
    }

    const parsed = JSON.parse(raw) as Partial<CommitMessageHistoryEntry>[]
    commitMessageHistory.value = parsed
      .filter(entry => typeof entry.title === 'string' && entry.title.trim())
      .map(entry => ({
        id: typeof entry.id === 'string' ? entry.id : createHistoryId(),
        repoPath: typeof entry.repoPath === 'string' ? entry.repoPath : '',
        repoName: typeof entry.repoName === 'string' ? entry.repoName : '',
        title: entry.title as string,
        body: typeof entry.body === 'string' ? entry.body : '',
        source: entry.source === 'manual' ? 'manual' : 'ai',
        selectedFileCount: typeof entry.selectedFileCount === 'number' ? entry.selectedFileCount : 0,
        createdAt: typeof entry.createdAt === 'number' ? entry.createdAt : Date.now(),
      }))
      .sort((left, right) => right.createdAt - left.createdAt)
      .slice(0, MAX_COMMIT_MESSAGE_HISTORY)
  } catch (err) {
    console.error(err)
    commitMessageHistory.value = []
  }
}

function persistCommitMessageHistory() {
  localStorage.setItem(GIT_COMMIT_MESSAGE_HISTORY_STORAGE_KEY, JSON.stringify(commitMessageHistory.value))
}

function createHistoryId() {
  return `commit-message-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`
}

function saveCommitMessageHistory(source: 'ai' | 'manual') {
  const title = commitTitle.value.trim()
  const body = commitBody.value.trim()
  if (!title && !body) return

  const repo = displayRepoPath.value
  const entry: CommitMessageHistoryEntry = {
    id: createHistoryId(),
    repoPath: repo,
    repoName: getRepoDisplayName(repo),
    title,
    body,
    source,
    selectedFileCount: reviewSelectedRaws.value.length,
    createdAt: Date.now(),
  }

  const duplicateKey = `${normalizePath(entry.repoPath).toLowerCase()}::${entry.title}::${entry.body}`
  commitMessageHistory.value = [
    entry,
    ...commitMessageHistory.value.filter(item => `${normalizePath(item.repoPath).toLowerCase()}::${item.title}::${item.body}` !== duplicateKey),
  ].slice(0, MAX_COMMIT_MESSAGE_HISTORY)
  persistCommitMessageHistory()
}

function restoreCommitMessage(entry: CommitMessageHistoryEntry) {
  commitTitle.value = entry.title
  commitBody.value = entry.body
  historyDrawerOpen.value = false
}

function formatHistoryTime(timestamp: number) {
  return new Date(timestamp).toLocaleString()
}

function historySourceLabel(source: 'ai' | 'manual') {
  return source === 'manual' ? t('gitAssistant.history.manual') : t('gitAssistant.history.ai')
}

async function handlePickDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('gitAssistant.repo.chooseDirectory'),
      defaultPath: repoPath.value || undefined,
    })

    if (!selected || Array.isArray(selected)) return

    repoPath.value = selected
    await loadSnapshotByPath(selected)
  } catch (err) {
    console.error(err)
    error.value = err instanceof Error ? err.message : t('gitAssistant.errorFallback')
  }
}

async function handleRefresh() {
  if (!repoPath.value) return
  await loadSnapshotByPath(repoPath.value)
}

async function handleSwitchRecentRepo(path: string) {
  if (!path || path === displayRepoPath.value) return
  repoPath.value = path
  await loadSnapshotByPath(path)
}

async function handleSwitchRecentRepoFromManager(path: string) {
  await handleSwitchRecentRepo(path)
  recentRepoManagerOpen.value = false
}

async function handleConfigureOrigin() {
  if (!displayRepoPath.value || !remoteUrlDraft.value.trim()) return

  remoteLoading.value = true
  error.value = ''
  const nextAction: '' | 'push' = repositoryState.value?.hasCommits ? 'push' : ''
  startGitCommand(t('gitAssistant.gitCommand.remoteTitle'), t('gitAssistant.gitCommand.configuringRemote'), nextAction)
  try {
    const result = await configureGitOrigin(displayRepoPath.value, remoteUrlDraft.value)
    finishGitCommand(result, nextAction === 'push' ? t('gitAssistant.gitCommand.pushNext') : '')
    await loadSnapshotByPath(displayRepoPath.value)
  } catch (err) {
    console.error(err)
    failGitCommand(err)
  } finally {
    remoteLoading.value = false
  }
}

async function handleRepairUpstream() {
  if (!displayRepoPath.value) return

  remoteLoading.value = true
  error.value = ''
  startGitCommand(t('gitAssistant.gitCommand.upstreamTitle'), t('gitAssistant.gitCommand.repairingUpstream'), 'push')
  try {
    const result = await repairGitUpstream(displayRepoPath.value)
    finishGitCommand(result, t('gitAssistant.gitCommand.pushNext'))
    await loadSnapshotByPath(displayRepoPath.value)
  } catch (err) {
    console.error(err)
    failGitCommand(err)
  } finally {
    remoteLoading.value = false
  }
}

function handleStatusFilterChange(value: string) {
  statusFilter.value = value as GitAssistantStatusFilter
}

function handleSelectFile(raw: string) {
  activeFileRaw.value = raw
}

function handleOpenDiff(raw: string) {
  activeFileRaw.value = raw
  showDiff.value = true
}

function toggleReviewSelection(payload: { raw: string; checked: boolean }) {
  if (payload.checked) {
    if (!reviewSelectedRaws.value.includes(payload.raw)) {
      reviewSelectedRaws.value = [...reviewSelectedRaws.value, payload.raw]
    }
    return
  }
  reviewSelectedRaws.value = reviewSelectedRaws.value.filter(raw => raw !== payload.raw)
}

function setReviewSelection(raws: string[]) {
  const validRaws = new Set(allFiles.value.map(file => file.raw))
  reviewSelectedRaws.value = [...new Set(raws.filter(raw => validRaws.has(raw)))]
}

onMounted(async () => {
  loadRecentRepos()
  loadCommitMessageHistory()
  const saved = localStorage.getItem(GIT_REPO_STORAGE_KEY)
  if (!saved) return

  repoPath.value = saved
  await loadSnapshotByPath(saved)
})

async function handleGenerateAiAnalysis() {
  if (!snapshot.value || !displayRepoPath.value) return

  const selectedFiles = allFiles.value
    .filter(file => reviewSelectedRaws.value.includes(file.raw))
    .map(file => file.path)

  if (!selectedFiles.length) {
    error.value = t('gitAssistant.ai.noSelectedFiles')
    return
  }

  aiLoading.value = true
  error.value = ''
  startPromptProgress()

  try {
    promptPreview.value = await buildGitCommitPrompt({
      repoPath: displayRepoPath.value,
      branch: snapshot.value.branch,
      selectedFiles,
    })

    if (autoSendPromptToApi.value) {
      const model = aiSettings.getModelForTask('commit-message')
      if (!model) {
        throw new Error(t('gitAssistant.ai.noModelConfigured'))
      }
      setPromptProgressStep('gitAssistant.ai.progressCallingApi')
      const result = await generateGitAiAnalysisFromPrompt({
        prompt: promptPreview.value.prompt,
        model,
      })
      commitTitle.value = result.title
      commitBody.value = result.body
      saveCommitMessageHistory('ai')
    }

    promptDrawerOpen.value = true
  } catch (err) {
    console.error(err)
    error.value = err instanceof Error ? err.message : t('gitAssistant.errorFallback')
  } finally {
    aiLoading.value = false
    stopPromptProgress()
  }
}

function startPromptProgress() {
  stopPromptProgress()
  promptGenerationStep.value = t('gitAssistant.ai.progressReading')
  promptProgressTimers = [
    window.setTimeout(() => {
      promptGenerationStep.value = t('gitAssistant.ai.progressCleaning')
    }, 400),
    window.setTimeout(() => {
      promptGenerationStep.value = t('gitAssistant.ai.progressBuilding')
    }, 1200),
  ]
}

function setPromptProgressStep(key: string) {
  for (const timer of promptProgressTimers) {
    window.clearTimeout(timer)
  }
  promptProgressTimers = []
  promptGenerationStep.value = t(key)
}

function stopPromptProgress() {
  for (const timer of promptProgressTimers) {
    window.clearTimeout(timer)
  }
  promptProgressTimers = []
  promptGenerationStep.value = ''
}

function startGitCommand(title: string, phase: string, nextAction: '' | 'push' | 'pull' = '') {
  gitCommandDialog.value = {
    visible: true,
    title,
    phase,
    running: true,
    success: null,
    command: '',
    stdout: '',
    stderr: '',
    message: t('gitAssistant.gitCommand.running'),
    suggestion: '',
    nextActionLabel: '',
    nextAction,
  }
}

function finishGitCommand(result: GitCommandResult, nextActionLabel = '') {
  gitCommandDialog.value = {
    ...gitCommandDialog.value,
    running: false,
    success: true,
    command: result.command,
    stdout: result.stdout,
    stderr: result.stderr,
    message: result.message,
    suggestion: result.suggestion ?? '',
    nextActionLabel,
  }
}

function failGitCommand(err: unknown) {
  const message = err instanceof Error ? err.message : String(err)
  const lowerMessage = message.toLowerCase()
  const nextAction: '' | 'push' | 'pull' =
    lowerMessage.includes('non-fast-forward') || lowerMessage.includes('fetch first')
      ? 'pull'
      : lowerMessage.includes('远端还没有') || lowerMessage.includes('push -u') || lowerMessage.includes('publish')
        ? 'push'
        : ''
  gitCommandDialog.value = {
    ...gitCommandDialog.value,
    running: false,
    success: false,
    stderr: message,
    message: t('gitAssistant.gitCommand.failed'),
    suggestion: '',
    nextActionLabel:
      nextAction === 'pull'
        ? t('gitAssistant.gitCommand.pullNext')
        : nextAction === 'push'
          ? t('gitAssistant.gitCommand.pushNext')
          : '',
    nextAction,
  }
}

async function handleCommit() {
  if (!displayRepoPath.value || !commitTitle.value.trim()) return
  const selectedFiles = allFiles.value
    .filter(file => reviewSelectedRaws.value.includes(file.raw))
    .map(file => file.path)

  if (!selectedFiles.length) {
    error.value = t('gitAssistant.ai.noSelectedFiles')
    return
  }

  commitLoading.value = true
  error.value = ''
  saveCommitMessageHistory('manual')
  startGitCommand(t('gitAssistant.gitCommand.commitTitle'), t('gitAssistant.gitCommand.committing'), 'push')

  try {
    const result = await commitGitChanges({
      repoPath: displayRepoPath.value,
      title: commitTitle.value,
      body: commitBody.value,
      selectedFiles,
    })

    commitTitle.value = ''
    commitBody.value = ''
    reviewSelectedRaws.value = []
    finishGitCommand(result, t('gitAssistant.gitCommand.pushNext'))
    await loadSnapshotByPath(displayRepoPath.value)
  } catch (err) {
    console.error(err)
    failGitCommand(err)
    await loadSnapshotByPath(displayRepoPath.value)
  } finally {
    commitLoading.value = false
  }
}

async function handlePush() {
  if (!displayRepoPath.value) return

  pushLoading.value = true
  error.value = ''
  startGitCommand(t('gitAssistant.gitCommand.pushTitle'), t('gitAssistant.gitCommand.pushing'))
  try {
    const result = await pushGitChanges(displayRepoPath.value)
    finishGitCommand(result)
    await loadSnapshotByPath(displayRepoPath.value)
  } catch (err) {
    console.error(err)
    failGitCommand(err)
  } finally {
    pushLoading.value = false
  }
}

async function handlePull() {
  if (!displayRepoPath.value) return

  pullLoading.value = true
  error.value = ''
  startGitCommand(t('gitAssistant.gitCommand.pullTitle'), t('gitAssistant.gitCommand.pulling'))
  try {
    const result = await pullGitChanges(displayRepoPath.value)
    finishGitCommand(result)
    await loadSnapshotByPath(displayRepoPath.value)
  } catch (err) {
    console.error(err)
    failGitCommand(err)
  } finally {
    pullLoading.value = false
  }
}

function handleCommandNextAction() {
  if (gitCommandDialog.value.nextAction === 'push') {
    void handlePush()
  } else if (gitCommandDialog.value.nextAction === 'pull') {
    void handlePull()
  }
}
</script>

<style scoped lang="scss">
.git-assistant-page {
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--lumina-bg) 94%, #fff), var(--lumina-bg)),
    var(--lumina-bg);
  box-sizing: border-box;
  color: var(--lumina-text);
  display: flex;
  flex-direction: column;
  gap: 10px;
  height: 100%;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
  padding: 10px;
  position: relative;
}

.error-banner {
  background: color-mix(in srgb, var(--lumina-danger) 12%, var(--lumina-surface-2));
  border: 1px solid color-mix(in srgb, var(--lumina-danger) 28%, transparent);
  border-radius: 10px;
  color: var(--lumina-danger);
  flex: 0 0 auto;
  font-size: 12px;
  padding: 8px 10px;
}

.workspace-body {
  display: grid;
  flex: 1 1 auto;
  gap: 10px;
  grid-template-rows: auto minmax(0, 1fr) auto;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}

.commit-area {
  display: grid;
  gap: 10px;
  grid-template-columns: minmax(0, 1fr) minmax(390px, 430px);
  min-height: 0;
}

.commit-workbench {
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 78%, transparent);
  border-radius: 12px;
  box-shadow:
    0 10px 28px rgba(0, 0, 0, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.32);
}

.commit-side {
  background: color-mix(in srgb, var(--lumina-surface-1) 88%, transparent);
  backdrop-filter: blur(18px);
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 78%, transparent);
  border-radius: 12px;
  box-shadow:
    0 10px 28px rgba(0, 0, 0, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.32);
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
}

.ai-panel-title {
  span {
    color: var(--lumina-text);
    font-size: 14px;
    font-weight: 650;
  }
}

.model-field {
  display: grid;
  gap: 6px;

  span {
    color: var(--lumina-text-secondary);
    font-size: 11px;
  }

  :deep(.model-select .n-base-selection) {
    background: color-mix(in srgb, var(--lumina-input-bg) 92%, transparent);
    border-radius: 7px;
    min-height: 32px;
  }

  :deep(.model-select .n-base-selection-label) {
    color: var(--lumina-text);
  }
}

.remote-tools {
  background: color-mix(in srgb, var(--lumina-surface-2) 58%, transparent);
  border: 1px solid var(--lumina-card-border);
  border-radius: 9px;
  display: grid;
  gap: 9px;
  padding: 10px;

  p {
    color: var(--lumina-text-secondary);
    font-size: 11px;
    line-height: 1.45;
    margin: 0;
  }
}

.remote-tools__header {
  align-items: center;
  display: flex;
  justify-content: space-between;

  span {
    color: var(--lumina-text);
    font-size: 12px;
    font-weight: 650;
  }

  strong {
    color: var(--lumina-warning);
    font-size: 11px;
    font-weight: 650;
  }
}

.remote-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.ai-toggle {
  color: var(--lumina-text-secondary);
  font-size: 12px;
  min-height: 24px;
}

:deep(.ai-toggle .n-checkbox__label) {
  color: var(--lumina-text-secondary);
  font-size: 12px;
}

.ai-actions {
  display: grid;
  gap: 8px;
  grid-template-columns: 1fr;
}

.ai-action,
.secondary-btn {
  background: color-mix(in srgb, var(--lumina-button-secondary-bg) 92%, transparent);
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 88%, var(--lumina-text-secondary));
  border-radius: 7px;
  color: var(--lumina-text);
  cursor: pointer;
  font-weight: 600;
  min-height: 32px;
  padding: 0 12px;
  width: 100%;

  &:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
}

.ai-progress {
  align-items: center;
  color: var(--lumina-text-secondary);
  display: flex;
  font-size: 11px;
  gap: 7px;
  min-height: 18px;
}

.ai-progress__dot {
  animation: ai-progress-pulse 1s ease-in-out infinite;
  background: var(--lumina-primary);
  border-radius: 999px;
  height: 7px;
  width: 7px;
}

@keyframes ai-progress-pulse {
  0%,
  100% {
    opacity: 0.35;
    transform: scale(0.85);
  }

  50% {
    opacity: 1;
    transform: scale(1);
  }
}

.primary-action {
  background: linear-gradient(180deg, color-mix(in srgb, var(--lumina-primary) 88%, #fff), var(--lumina-primary));
  border-color: var(--lumina-primary);
  box-shadow: 0 6px 14px color-mix(in srgb, var(--lumina-primary) 18%, transparent);
  color: #fff;
}

.change-table {
  min-height: 0;
}

.diff-window {
  height: 100%;
  width: 100%;
}

.diff-dialog {
  background: color-mix(in srgb, var(--lumina-surface-1) 96%, transparent);
  backdrop-filter: blur(22px);
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 78%, transparent);
  border-radius: 12px;
  box-shadow:
    0 24px 68px rgba(0, 0, 0, 0.24),
    inset 0 1px 0 rgba(255, 255, 255, 0.32);
  height: min(820px, calc(100vh - 76px));
  overflow: hidden;
  position: relative;
  width: min(1480px, calc(100vw - 72px));
}

.modal-close-button {
  align-items: center;
  background: color-mix(in srgb, var(--lumina-surface-2) 88%, transparent);
  border: 1px solid var(--lumina-card-border);
  border-radius: 8px;
  color: var(--lumina-text-secondary);
  cursor: pointer;
  display: flex;
  font-size: 22px;
  height: 32px;
  justify-content: center;
  line-height: 1;
  position: absolute;
  right: 12px;
  top: 12px;
  transition:
    background 0.18s ease,
    color 0.18s ease;
  width: 32px;
  z-index: 3;

  &:hover {
    background: var(--lumina-button-secondary-hover);
    color: var(--lumina-text);
  }
}

:deep(.diff-window.diff-viewer) {
  border: 0;
  border-radius: 12px;
  box-shadow: none;
  height: 100%;
  width: 100%;
}

:deep(.diff-window .diff-header) {
  padding-right: 56px;
}

.recent-repo-dialog {
  background: color-mix(in srgb, var(--lumina-surface-1) 96%, transparent);
  backdrop-filter: blur(22px);
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 78%, transparent);
  border-radius: 12px;
  box-shadow:
    0 24px 68px rgba(0, 0, 0, 0.22),
    inset 0 1px 0 rgba(255, 255, 255, 0.32);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  max-height: min(640px, calc(100vh - 96px));
  overflow: hidden;
  position: relative;
  width: min(760px, calc(100vw - 72px));
}

.recent-repo-dialog__header {
  align-items: flex-start;
  border-bottom: 1px solid var(--lumina-card-border);
  display: flex;
  gap: 14px;
  justify-content: space-between;
  padding: 16px 56px 16px 16px;

  h3 {
    font-size: 16px;
    margin: 0 0 4px;
  }

  p {
    color: var(--lumina-text-secondary);
    font-size: 12px;
    line-height: 1.5;
    margin: 0;
  }
}

.recent-repo-list {
  display: grid;
  gap: 10px;
  min-height: 0;
  overflow: auto;
  padding: 14px;
}

.recent-repo-item {
  align-items: center;
  background: color-mix(in srgb, var(--lumina-surface-2) 58%, transparent);
  border: 1px solid var(--lumina-card-border);
  border-radius: 10px;
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(0, 1fr) auto;
  padding: 12px;
}

.recent-repo-item__main {
  display: grid;
  gap: 7px;
  min-width: 0;
}

.recent-repo-path {
  color: var(--lumina-text-secondary);
  font-size: 11px;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-repo-item__actions {
  display: flex;
  gap: 8px;
}

.recent-repo-empty {
  align-items: center;
  color: var(--lumina-text-secondary);
  display: flex;
  font-size: 12px;
  justify-content: center;
  min-height: 180px;
  padding: 20px;
}

.prompt-drawer {
  background: var(--lumina-surface-1);
  border-left: 1px solid var(--lumina-card-border);
  bottom: 0;
  box-shadow: -8px 0 24px rgba(0, 0, 0, 0.12);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  position: absolute;
  right: 0;
  top: 0;
  width: min(1080px, 72vw);
  z-index: 20;
}

.history-drawer {
  background: color-mix(in srgb, var(--lumina-surface-1) 94%, transparent);
  backdrop-filter: blur(20px);
  border-left: 1px solid var(--lumina-card-border);
  bottom: 0;
  box-shadow: -8px 0 24px rgba(0, 0, 0, 0.12);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  position: absolute;
  right: 0;
  top: 0;
  width: min(560px, 46vw);
  z-index: 22;
}

.history-drawer__header {
  align-items: flex-start;
  border-bottom: 1px solid var(--lumina-card-border);
  display: flex;
  gap: 14px;
  justify-content: space-between;
  padding: 14px;

  h3 {
    font-size: 16px;
    margin: 0 0 4px;
  }

  p {
    color: var(--lumina-text-secondary);
    font-size: 12px;
    line-height: 1.5;
    margin: 0;
  }

  button {
    background: color-mix(in srgb, var(--lumina-button-secondary-bg) 92%, transparent);
    border: 1px solid var(--lumina-card-border);
    border-radius: 7px;
    color: var(--lumina-text);
    cursor: pointer;
    height: 30px;
    padding: 0 10px;
  }
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
  overflow: auto;
  padding: 14px;
}

.history-item {
  background: color-mix(in srgb, var(--lumina-surface-2) 58%, transparent);
  border: 1px solid var(--lumina-card-border);
  border-radius: 9px;
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(0, 1fr) auto;
  padding: 12px;

  button {
    align-self: start;
    background: var(--lumina-primary);
    border: 1px solid var(--lumina-primary);
    border-radius: 7px;
    color: #fff;
    cursor: pointer;
    height: 30px;
    padding: 0 12px;
  }
}

.history-item__main {
  display: grid;
  gap: 5px;
  min-width: 0;

  span,
  small {
    color: var(--lumina-text-secondary);
    font-size: 11px;
  }

  strong {
    color: var(--lumina-text);
    font-size: 13px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  p {
    color: var(--lumina-text-secondary);
    display: -webkit-box;
    font-size: 12px;
    line-height: 1.5;
    margin: 0;
    overflow: hidden;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
  }
}

.history-empty {
  align-items: center;
  border: 1px dashed var(--lumina-card-border);
  border-radius: 9px;
  color: var(--lumina-text-secondary);
  display: flex;
  font-size: 12px;
  justify-content: center;
  min-height: 120px;
  padding: 16px;
  text-align: center;
}

.prompt-drawer__header {
  align-items: flex-start;
  border-bottom: 1px solid var(--lumina-card-border);
  display: flex;
  gap: 14px;
  justify-content: space-between;
  padding: 14px;

  h3 {
    font-size: 16px;
    margin: 0 0 4px;
  }

  p {
    color: var(--lumina-text-secondary);
    font-size: 12px;
    margin: 0;
  }

  button {
    border: 1px solid var(--lumina-card-border);
    border-radius: 8px;
    color: var(--lumina-text);
    cursor: pointer;
    height: 30px;
    padding: 0 10px;
  }
}

.prompt-drawer__body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
  overflow: auto;
  padding: 14px;
}

.prompt-section {
  border: 1px solid var(--lumina-card-border);
  border-radius: 8px;
  background: var(--lumina-surface-1);

  summary {
    cursor: pointer;
    font-size: 13px;
    font-weight: 700;
    padding: 10px 12px;
  }
}

.prompt-stats {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(4, minmax(0, 1fr));

  div {
    background: var(--lumina-surface-2);
    border: 1px solid var(--lumina-card-border);
    border-radius: 8px;
    padding: 9px;
  }

  span {
    color: var(--lumina-text-secondary);
    display: block;
    font-size: 11px;
  }

  strong {
    display: block;
    font-size: 18px;
    margin-top: 4px;
  }
}

.prompt-files,
.prompt-text {
  min-width: 0;
}

.prompt-rules {
  color: var(--lumina-text-secondary);
  font-size: 12px;
  padding: 0 12px 12px;

  p {
    margin: 0 0 8px;
  }

  ol {
    margin: 0;
    padding-left: 18px;
  }

  li + li {
    margin-top: 5px;
  }
}

.prompt-files {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 0 12px 12px;
}

.prompt-file-group {
  border: 1px solid var(--lumina-card-border);
  border-radius: 8px;
  overflow: hidden;
}

.prompt-file-group__title {
  align-items: center;
  background: var(--lumina-surface-2);
  border-bottom: 1px solid var(--lumina-card-border);
  display: flex;
  justify-content: space-between;
  padding: 8px 10px;

  strong {
    font-size: 13px;
  }

  span {
    color: var(--lumina-text-secondary);
    font-size: 12px;
  }
}

.prompt-file-table {
  overflow: auto;
  width: 100%;
}

.prompt-file-table__head,
.prompt-file-table__row {
  align-items: center;
  display: grid;
  gap: 10px;
  grid-template-columns: minmax(260px, 2fr) 86px 86px minmax(150px, 1fr) 70px 96px minmax(180px, 1.2fr);
  min-width: 920px;
  min-height: 32px;
  padding: 0 10px;
}

.prompt-file-table__head {
  color: var(--lumina-text-secondary);
  font-size: 11px;
  font-weight: 700;
}

.prompt-file-table__row {
  border-top: 1px solid var(--lumina-card-border);
  font-size: 12px;

  span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  span:not(.mono) {
    color: var(--lumina-text-secondary);
  }
}

.prompt-text {
  padding: 0 12px 12px;

  textarea {
    background: var(--lumina-diff-bg);
    border: 1px solid var(--lumina-card-border);
    border-radius: 8px;
    color: var(--lumina-text);
    font-family: SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
    font-size: 12px;
    line-height: 1.6;
    min-height: 460px;
    padding: 12px;
    resize: none;
    width: 100%;
  }
}

@media (max-width: 980px) {
  .commit-area {
    grid-template-columns: 1fr;
  }

  .prompt-drawer {
    width: 100%;
  }

  .history-drawer {
    width: 100%;
  }
}
</style>
