import { invoke } from '@tauri-apps/api/core'

export interface GitSnapshot {
  repoPath: string
  repoRoot: string
  branch: string
  repositoryState: GitRepositoryState
  status: string[]
  stagedFiles: string[]
  stagedDiff: string
  fileStats: GitFileStat[]
}

export interface GitRepositoryState {
  hasCommits: boolean
  remoteName?: string | null
  remoteUrl?: string | null
  upstream?: string | null
  upstreamGone: boolean
  ahead: number
  behind: number
}

export interface GitFileStat {
  path: string
  added: number | null
  removed: number | null
}

export async function loadGitSnapshot(repoPath: string): Promise<GitSnapshot> {
  return await invoke<GitSnapshot>('load_git_snapshot', { repoPath })
}

export interface GitFileDiffResponse {
  filePath: string
  staged: boolean
  diff: string
}

export async function loadGitFileDiff(payload: {
  repoPath: string
  filePath: string
  staged: boolean
}): Promise<GitFileDiffResponse> {
  return await invoke<GitFileDiffResponse>('load_git_file_diff', { payload })
}

export async function commitGitChanges(payload: {
  repoPath: string
  title: string
  body: string
  selectedFiles: string[]
}): Promise<GitCommandResult> {
  return await invoke<GitCommandResult>('commit_git_changes', { payload })
}

export interface GitCommandResult {
  command: string
  message: string
  stdout: string
  stderr: string
  suggestion?: string | null
}

export async function pushGitChanges(repoPath: string): Promise<GitCommandResult> {
  return await invoke<GitCommandResult>('push_git_changes', { payload: { repoPath } })
}

export async function pullGitChanges(repoPath: string): Promise<GitCommandResult> {
  return await invoke<GitCommandResult>('pull_git_changes', { payload: { repoPath } })
}

export async function configureGitOrigin(repoPath: string, remoteUrl: string): Promise<GitCommandResult> {
  return await invoke<GitCommandResult>('configure_git_origin', { payload: { repoPath, remoteUrl } })
}

export async function repairGitUpstream(repoPath: string): Promise<GitCommandResult> {
  return await invoke<GitCommandResult>('repair_git_upstream', { payload: { repoPath } })
}
