<template>
  <div class="git-assistant-page">
    <div v-if="error" class="error-banner">
      {{ error }}
    </div>

    <GitStatusBar
      :repo-path="displayRepoPath"
      :branch="snapshot?.branch ?? ''"
      :loading="loading"
      :summary="summary"
      :recommended-count="recommendedFiles.length"
      :status-text="repoStatusText"
      @pick-directory="handlePickDirectory"
      @refresh="handleRefresh"
    />

    <section class="workspace-body">
      <section class="commit-area">
        <GitCommitAssistant
          class="commit-workbench"
          :generating="aiLoading"
          :generate-disabled="!snapshot"
          :committing="commitLoading"
          :pushing="pushLoading"
          :pulling="pullLoading"
          :submit-disabled="!snapshot || !reviewSelectedRaws.length || !commitTitle.trim()"
          :title="commitTitle"
          :body="commitBody"
          :selected-count="reviewSelectedRaws.length"
          :recommended-files="recommendedFiles"
          :status-text="commitStatusText"
          @submit="handleCommit"
          @push="handlePush"
          @pull="handlePull"
          @update:title="commitTitle = $event"
          @update:body="commitBody = $event"
        />

        <aside class="commit-side">
          <div class="ai-panel-title">
            <span>{{ t('gitAssistant.ai.actionsTitle') }}</span>
            <strong>{{ reviewSelectedRaws.length }}</strong>
          </div>

          <label class="model-field">
            <span>{{ t('gitAssistant.ai.currentModel') }}</span>
            <select
              :value="aiSettings.taskModelMap['commit-message'] || aiSettings.defaultModelId"
              :disabled="!aiSettings.enabledModels.length"
              @change="aiSettings.setTaskModel('commit-message', ($event.target as HTMLSelectElement).value)"
            >
              <option v-if="!aiSettings.enabledModels.length" value="">
                {{ t('gitAssistant.ai.noModelConfigured') }}
              </option>
              <option v-for="model in aiSettings.enabledModels" :key="model.id" :value="model.id">
                {{ model.name }}
              </option>
            </select>
          </label>

          <label class="ai-toggle">
            <input v-model="autoSendPromptToApi" type="checkbox" />
            <span>{{ t('gitAssistant.ai.autoSendPrompt') }}</span>
          </label>

          <div class="ai-stats">
            <div class="side-row">
              <span>{{ t('gitAssistant.workbench.selectedForReview') }}</span>
              <strong>{{ reviewSelectedRaws.length }}</strong>
            </div>
            <div class="side-row">
              <span>{{ t('gitAssistant.repo.summaryRecommended') }}</span>
              <strong>{{ recommendedFiles.length }}</strong>
            </div>
          </div>

          <div class="ai-actions">
            <button class="ai-action primary-action" type="button" :disabled="!snapshot || aiLoading" @click="handleGenerateAiAnalysis">
              {{ aiLoading ? t('gitAssistant.ai.generating') : t('gitAssistant.ai.generate') }}
            </button>
            <button class="ai-action" type="button" :disabled="!promptPreview" @click="promptDrawerOpen = true">
              {{ t('gitAssistant.ai.viewPrompt') }}
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
        @toggle-review-selection="toggleReviewSelection"
        @set-review-selection="setReviewSelection"
      />

        <GitDiffViewer
          v-if="showDiff"
          class="diff-drawer"
          :has-snapshot="Boolean(snapshot)"
          :active-file="selectedFile"
          :diff-text="currentDiff"
          :loading="diffLoading"
          :current-mode="diffMode"
          @update:mode="diffMode = $event"
        />
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
import { useLocale } from '@/hooks/useLocale'
import {
  buildGitCommitPrompt,
  generateGitAiAnalysisFromPrompt,
  type GitCommitPromptFileTrace,
  type GitCommitPromptPreview,
} from '@/services/git/git-ai-service'
import {
  commitGitChanges,
  type GitCommandResult,
  loadGitFileDiff,
  loadGitSnapshot,
  pullGitChanges,
  pushGitChanges,
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
import { ATTENTION_SCORE_CONFIG, GIT_REPO_STORAGE_KEY } from './git-assistant.config'
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
const error = ref('')
const repoPath = ref('')
const snapshot = ref<GitSnapshot | null>(null)

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
const promptGenerationStep = ref('')
const autoSendPromptToApi = ref(true)
let promptProgressTimers: number[] = []

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
  nextAction: '' as '' | 'push',
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

const repoStatusText = computed(() => {
  if (!snapshot.value) return t('gitAssistant.repo.statusUnloaded')
  if (!summary.value.total) return t('gitAssistant.repo.statusClean')
  if (!summary.value.staged) return t('gitAssistant.repo.statusNoStaged', { count: summary.value.total })

  return t('gitAssistant.repo.statusReady', {
    count: summary.value.total,
    recommended: recommendedFiles.value.length,
  })
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

const commitStatusText = computed(() => {
  return t('gitAssistant.ai.reviewSelected', { count: reviewSelectedRaws.value.length })
})

const currentModelName = computed(() => aiSettings.getModelForTask('commit-message')?.name ?? t('gitAssistant.ai.noModelConfigured'))

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
    reviewSelectedRaws.value = []
    localStorage.setItem(GIT_REPO_STORAGE_KEY, path)

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

function handleStatusFilterChange(value: string) {
  statusFilter.value = value as GitAssistantStatusFilter
}

function handleSelectFile(raw: string) {
  activeFileRaw.value = raw
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

function startGitCommand(title: string, phase: string, nextAction: '' | 'push' = '') {
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
  gitCommandDialog.value = {
    ...gitCommandDialog.value,
    running: false,
    success: false,
    stderr: message,
    message: t('gitAssistant.gitCommand.failed'),
    suggestion: '',
    nextActionLabel: '',
  }
  error.value = message
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
  }
}
</script>

<style scoped lang="scss">
.git-assistant-page {
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--lumina-surface-2) 78%, transparent), var(--lumina-bg)),
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
  grid-template-columns: minmax(0, 1fr) 360px;
  min-height: 0;
}

.commit-workbench {
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: 10px;
  box-shadow: var(--lumina-shadow-sm);
}

.commit-side {
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: 10px;
  box-shadow: var(--lumina-shadow-sm);
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
}

.ai-panel-title {
  align-items: center;
  display: flex;
  justify-content: space-between;

  span {
    color: var(--lumina-text);
    font-size: 14px;
    font-weight: 700;
  }

  strong {
    color: var(--lumina-primary);
    font-size: 20px;
    line-height: 1;
  }
}

.model-field {
  display: grid;
  gap: 6px;

  span {
    color: var(--lumina-text-secondary);
    font-size: 11px;
  }

  select {
    background: var(--lumina-input-bg);
    border: 1px solid var(--lumina-input-border);
    border-radius: 8px;
    color: var(--lumina-text);
    height: 32px;
    min-width: 0;
    padding: 0 8px;

    &:focus {
      border-color: var(--lumina-primary);
      box-shadow: 0 0 0 2px var(--lumina-accent-ring);
      outline: none;
    }
  }
}

.ai-stats {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
}

.side-row {
  background: var(--lumina-surface-2);
  border: 1px solid var(--lumina-card-border);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
  padding: 8px;

  span {
    color: var(--lumina-text-secondary);
    font-size: 11px;
  }

  strong {
    color: var(--lumina-text);
    font-size: 12px;
    font-weight: 600;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.ai-toggle {
  align-items: center;
  color: var(--lumina-text-secondary);
  cursor: pointer;
  display: flex;
  font-size: 12px;
  gap: 8px;
  min-height: 24px;

  input {
    accent-color: var(--lumina-primary);
    height: 14px;
    margin: 0;
    width: 14px;
  }
}

.ai-actions {
  display: grid;
  gap: 8px;
  grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr);
}

.ai-action,
.secondary-btn {
  background: var(--lumina-button-secondary-bg);
  border: 1px solid var(--lumina-card-border);
  border-radius: 8px;
  color: var(--lumina-text);
  cursor: pointer;
  height: 30px;
  padding: 0 10px;
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
  background: var(--lumina-primary);
  border-color: var(--lumina-primary);
  color: #fff;
}

.change-table {
  min-height: 0;
}

.diff-drawer {
  min-height: 260px;
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
}
</style>
