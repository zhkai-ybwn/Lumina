import { ref } from 'vue'
import { useLocale } from '@/hooks/useLocale'
import {
  abortGitMerge,
  abortGitRebase,
  configureGitOrigin,
  continueGitMerge,
  continueGitRebase,
  fetchGitChanges,
  type GitCommandResult,
  markGitFilesResolved,
  pullGitChanges,
  pushGitChanges,
  rebaseGitChanges,
  repairGitUpstream,
  syncGitStatus,
} from '@/services/git/git-service'

export interface GitCommandDialogState {
  visible: boolean
  title: string
  phase: string
  running: boolean
  success: boolean | null
  command: string
  stdout: string
  stderr: string
  message: string
  suggestion: string
  nextActionLabel: string
  nextAction: '' | 'push' | 'pull'
}

export function useGitRemote(
  getDisplayRepoPath: () => string,
  getRepositoryState: () => { remoteName?: string; hasCommits?: boolean; upstreamGone?: boolean; upstream?: string; ahead?: number; behind?: number; remoteUrl?: string } | null,
  setError: (msg: string) => void,
  loadSnapshotByPath: (path: string) => Promise<void>,
  clearReviewSelection: () => void,
) {
  const { t } = useLocale()

  const fetchLoading = ref(false)
  const pushLoading = ref(false)
  const pullLoading = ref(false)
  const rebaseLoading = ref(false)
  const remoteLoading = ref(false)
  const conflictLoading = ref(false)
  const remoteUrlDraft = ref('')

  const gitCommandDialog = ref<GitCommandDialogState>({
    visible: false,
    title: '',
    phase: '',
    running: false,
    success: null,
    command: '',
    stdout: '',
    stderr: '',
    message: '',
    suggestion: '',
    nextActionLabel: '',
    nextAction: '',
  })

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

  async function handleConfigureOrigin() {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath || !remoteUrlDraft.value.trim()) return

    remoteLoading.value = true
    setError('')
    const state = getRepositoryState()
    const nextAction: '' | 'push' = state?.hasCommits ? 'push' : ''
    startGitCommand(t('gitAssistant.gitCommand.remoteTitle'), t('gitAssistant.gitCommand.configuringRemote'), nextAction)
    try {
      const result = await configureGitOrigin(displayRepoPath, remoteUrlDraft.value)
      finishGitCommand(result, nextAction === 'push' ? t('gitAssistant.gitCommand.pushNext') : '')
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
    } finally {
      remoteLoading.value = false
    }
  }

  async function handleRepairUpstream() {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath) return

    remoteLoading.value = true
    setError('')
    startGitCommand(t('gitAssistant.gitCommand.upstreamTitle'), t('gitAssistant.gitCommand.repairingUpstream'), 'push')
    try {
      const result = await repairGitUpstream(displayRepoPath)
      finishGitCommand(result, t('gitAssistant.gitCommand.pushNext'))
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
    } finally {
      remoteLoading.value = false
    }
  }

  async function handlePush() {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath) return

    pushLoading.value = true
    setError('')
    startGitCommand(t('gitAssistant.gitCommand.pushTitle'), t('gitAssistant.gitCommand.checkingRemote'))
    try {
      const syncStatus = await syncGitStatus(displayRepoPath)
      const action = syncStatus.recommendedAction
      if (action === 'pull' || action === 'resolveDivergence') {
        finishGitCommand(syncStatus, t('gitAssistant.gitCommand.pullNext'))
        gitCommandDialog.value = {
          ...gitCommandDialog.value,
          success: action === 'pull' ? null : false,
          nextAction: 'pull',
        }
        await loadSnapshotByPath(displayRepoPath)
        return
      }

      if (action === 'configureRemote') {
        finishGitCommand(syncStatus)
        gitCommandDialog.value = {
          ...gitCommandDialog.value,
          success: false,
        }
        await loadSnapshotByPath(displayRepoPath)
        return
      }

      gitCommandDialog.value = {
        ...gitCommandDialog.value,
        phase: t('gitAssistant.gitCommand.pushing'),
        command: syncStatus.command,
        stdout: syncStatus.stdout,
        stderr: syncStatus.stderr,
        message: syncStatus.message,
        suggestion: syncStatus.suggestion ?? '',
      }
      const result = await pushGitChanges(displayRepoPath)
      finishGitCommand(result)
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
    } finally {
      pushLoading.value = false
    }
  }

  async function handleFetch() {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath) return

    fetchLoading.value = true
    setError('')
    startGitCommand(t('gitAssistant.gitCommand.fetchTitle'), t('gitAssistant.gitCommand.fetching'))
    try {
      const result = await fetchGitChanges(displayRepoPath)
      finishGitCommand(result)
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
    } finally {
      fetchLoading.value = false
    }
  }

  async function handlePull() {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath) return

    pullLoading.value = true
    setError('')
    startGitCommand(t('gitAssistant.gitCommand.pullTitle'), t('gitAssistant.gitCommand.pulling'))
    try {
      const result = await pullGitChanges(displayRepoPath)
      finishGitCommand(result)
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
    } finally {
      pullLoading.value = false
    }
  }

  async function handleRebase() {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath) return

    rebaseLoading.value = true
    setError('')
    startGitCommand(t('gitAssistant.gitCommand.rebaseTitle'), t('gitAssistant.gitCommand.rebasing'))
    try {
      const result = await rebaseGitChanges(displayRepoPath)
      finishGitCommand(result)
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
      await loadSnapshotByPath(displayRepoPath)
    } finally {
      rebaseLoading.value = false
    }
  }

  async function handleMarkResolved(filePaths: string[]) {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath || !filePaths.length) return

    conflictLoading.value = true
    setError('')
    startGitCommand(t('gitAssistant.gitCommand.resolveTitle'), t('gitAssistant.gitCommand.resolvingConflicts'))
    try {
      const result = await markGitFilesResolved(displayRepoPath, filePaths)
      finishGitCommand(result)
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
    } finally {
      conflictLoading.value = false
    }
  }

  async function handleAbortMerge() {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath || !window.confirm(t('gitAssistant.conflict.abortConfirm'))) return

    conflictLoading.value = true
    setError('')
    startGitCommand(t('gitAssistant.gitCommand.abortMergeTitle'), t('gitAssistant.gitCommand.abortingMerge'))
    try {
      const result = await abortGitMerge(displayRepoPath)
      finishGitCommand(result)
      clearReviewSelection()
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
    } finally {
      conflictLoading.value = false
    }
  }

  async function handleContinueMerge() {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath) return

    conflictLoading.value = true
    setError('')
    startGitCommand(t('gitAssistant.gitCommand.mergeTitle'), t('gitAssistant.gitCommand.merging'))
    try {
      const result = await continueGitMerge(displayRepoPath)
      finishGitCommand(result)
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
    } finally {
      conflictLoading.value = false
    }
  }

  async function handleContinueRebase() {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath) return

    conflictLoading.value = true
    setError('')
    startGitCommand(t('gitAssistant.gitCommand.rebaseTitle'), t('gitAssistant.gitCommand.rebasing'))
    try {
      const result = await continueGitRebase(displayRepoPath)
      finishGitCommand(result)
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
      await loadSnapshotByPath(displayRepoPath)
    } finally {
      conflictLoading.value = false
    }
  }

  async function handleAbortRebase() {
    const displayRepoPath = getDisplayRepoPath()
    if (!displayRepoPath || !window.confirm(t('gitAssistant.conflict.abortRebaseConfirm'))) return

    conflictLoading.value = true
    setError('')
    startGitCommand(t('gitAssistant.gitCommand.rebaseTitle'), t('gitAssistant.gitCommand.aborting'))
    try {
      const result = await abortGitRebase(displayRepoPath)
      finishGitCommand(result)
      await loadSnapshotByPath(displayRepoPath)
    } catch (err) {
      console.error(err)
      failGitCommand(err)
    } finally {
      conflictLoading.value = false
    }
  }

  function handleCommandNextAction() {
    if (gitCommandDialog.value.nextAction === 'push') {
      void handlePush()
    } else if (gitCommandDialog.value.nextAction === 'pull') {
      void handlePull()
    }
  }

  return {
    fetchLoading,
    pushLoading,
    pullLoading,
    rebaseLoading,
    remoteLoading,
    conflictLoading,
    remoteUrlDraft,
    gitCommandDialog,
    handleConfigureOrigin,
    handleRepairUpstream,
    handlePush,
    handleFetch,
    handlePull,
    handleRebase,
    handleMarkResolved,
    handleAbortMerge,
    handleContinueMerge,
    handleContinueRebase,
    handleAbortRebase,
    handleCommandNextAction,
    startGitCommand,
    finishGitCommand,
    failGitCommand,
  }
}
