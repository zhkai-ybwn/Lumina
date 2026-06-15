export type GitChangeType =
  | 'modified'
  | 'added'
  | 'deleted'
  | 'renamed'
  | 'copied'
  | 'untracked'
  | 'updated-but-unmerged'
  | 'unknown'

export interface GitFileStatus {
  raw: string
  x: string // index status
  y: string // worktree status
  path: string
  originalPath?: string // rename 时可用
  type: GitChangeType
  staged: boolean
  unstaged: boolean
}

export interface GitRepoInfoResponse {
  repoPath: string
  repoRoot: string
  branch: string
  status: string[]
  stagedFiles: string[]
  stagedDiff: string
}
