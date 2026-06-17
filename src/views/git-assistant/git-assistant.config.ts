import type { GitFileStatus } from '@/types/git'
export {
  GIT_COMMIT_MESSAGE_HISTORY_STORAGE_KEY,
  GIT_RECENT_REPOS_STORAGE_KEY,
  GIT_REPO_STORAGE_KEY,
} from '@/constants/git'
import type { GitAssistantStatusFilter } from './git-assistant.types'

export const STATUS_FILTER_OPTIONS: Array<{
  value: GitAssistantStatusFilter
  labelKey: string
}> = [
  { value: 'all', labelKey: 'gitAssistant.files.filters.all' },
  { value: 'untracked', labelKey: 'gitAssistant.files.filters.untracked' },
  { value: 'versioned', labelKey: 'gitAssistant.files.filters.versioned' },
  { value: 'added', labelKey: 'gitAssistant.files.filters.added' },
  { value: 'modified', labelKey: 'gitAssistant.files.filters.modified' },
  { value: 'deleted', labelKey: 'gitAssistant.files.filters.deleted' },
  { value: 'recommended', labelKey: 'gitAssistant.files.filters.recommended' },
]

export const STATUS_META: Record<
  GitFileStatus['type'],
  {
    labelKey: string
    shortLabel: string
    tone: 'warning' | 'success' | 'danger' | 'info' | 'muted'
  }
> = {
  modified: { labelKey: 'gitAssistant.status.modified', shortLabel: 'M', tone: 'warning' },
  added: { labelKey: 'gitAssistant.status.added', shortLabel: 'A', tone: 'success' },
  deleted: { labelKey: 'gitAssistant.status.deleted', shortLabel: 'D', tone: 'danger' },
  renamed: { labelKey: 'gitAssistant.status.renamed', shortLabel: 'R', tone: 'info' },
  copied: { labelKey: 'gitAssistant.status.copied', shortLabel: 'C', tone: 'info' },
  untracked: { labelKey: 'gitAssistant.status.untracked', shortLabel: 'U', tone: 'muted' },
  'updated-but-unmerged': { labelKey: 'gitAssistant.status.updated-but-unmerged', shortLabel: 'U', tone: 'danger' },
  unknown: { labelKey: 'gitAssistant.status.unknown', shortLabel: '?', tone: 'muted' },
}

export const ATTENTION_SCORE_CONFIG = {
  recommendedThreshold: 6,
  highRiskDirectories: ['src/core/', 'src/services/', 'src/store/', 'src/views/'],
  highRiskFiles: ['package.json', 'pnpm-lock.yaml', 'yarn.lock', 'package-lock.json', '.env'],
  docsExtensions: ['md', 'txt'],
  configExtensions: ['json', 'yaml', 'yml', 'toml', 'ini'],
}
