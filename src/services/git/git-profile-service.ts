import { invoke } from '@tauri-apps/api/core'
import type { GitProjectProfileFile } from '@/types/git-profile'

export async function ensureGitProjectProfile(repoPath: string): Promise<GitProjectProfileFile> {
  return invoke<GitProjectProfileFile>('ensure_git_project_profile', { repoPath })
}

export async function loadGitProjectProfile(repoPath: string): Promise<GitProjectProfileFile> {
  return invoke<GitProjectProfileFile>('load_git_project_profile', { repoPath })
}

export async function saveGitProjectProfile(payload: {
  repoPath: string
  content: string
}): Promise<GitProjectProfileFile> {
  return invoke<GitProjectProfileFile>('save_git_project_profile', { payload })
}
