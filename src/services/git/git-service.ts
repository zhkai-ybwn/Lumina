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

export async function loadGitFileHeadDiff(repoPath: string, filePath: string): Promise<GitFileDiffResponse> {
  return await invoke<GitFileDiffResponse>('load_git_file_head_diff', { payload: { repoPath, filePath } })
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

export interface GitLogEntry {
  hash: string
  shortHash: string
  authorName: string
  authorEmail: string
  date: string
  subject: string
}

export interface GitCommitChangedFile {
  status: string
  path: string
  originalPath?: string | null
  added: number | null
  removed: number | null
}

export interface GitCommitDetail extends GitLogEntry {
  body: string
  shortStat: string
  changedFiles: GitCommitChangedFile[]
}

export interface GitCommitFileDiffResponse {
  hash: string
  filePath: string
  diff: string
}

export async function fetchGitChanges(repoPath: string): Promise<GitCommandResult> {
  return await invoke<GitCommandResult>('fetch_git_changes', { payload: { repoPath } })
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

export async function openGitFileExternal(repoPath: string, filePath: string): Promise<GitCommandResult> {
  return await invoke<GitCommandResult>('open_git_file_external', { payload: { repoPath, filePath } })
}

export async function markGitFilesResolved(repoPath: string, filePaths: string[]): Promise<GitCommandResult> {
  return await invoke<GitCommandResult>('mark_git_files_resolved', { payload: { repoPath, filePaths } })
}

export async function abortGitMerge(repoPath: string): Promise<GitCommandResult> {
  return await invoke<GitCommandResult>('abort_git_merge', { payload: { repoPath } })
}

export async function loadGitLog(repoPath: string, filePath?: string): Promise<GitLogEntry[]> {
  return await invoke<GitLogEntry[]>('load_git_log', { payload: { repoPath, filePath: filePath || null } })
}

export async function loadGitCommitDetail(repoPath: string, hash: string): Promise<GitCommitDetail> {
  return await invoke<GitCommitDetail>('load_git_commit_detail', { payload: { repoPath, hash } })
}

export async function loadGitCommitFileDiff(repoPath: string, hash: string, filePath: string): Promise<GitCommitFileDiffResponse> {
  return await invoke<GitCommitFileDiffResponse>('load_git_commit_file_diff', { payload: { repoPath, hash, filePath } })
}
