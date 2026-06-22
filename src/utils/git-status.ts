import type { GitChangeType, GitFileStatus } from '@/types/git'

function mapStatusCharToType(char: string): GitChangeType {
  switch (char) {
    case 'M':
      return 'modified'
    case 'A':
      return 'added'
    case 'D':
      return 'deleted'
    case 'R':
      return 'renamed'
    case 'C':
      return 'copied'
    case 'U':
      return 'updated-but-unmerged'
    default:
      return 'unknown'
  }
}

export function parseGitStatusLine(line: string): GitFileStatus {
  const raw = line

  // 未追踪文件
  if (line.startsWith('??')) {
    return {
      raw,
      x: '?',
      y: '?',
      path: line.slice(3),
      type: 'untracked',
      staged: false,
      unstaged: true,
    }
  }

  // 正常 short status: XY<space>path
  const x = line[0] ?? ' '
  const y = line[1] ?? ' '
  const body = line.slice(3)
  const unmerged = isUnmergedStatus(x, y)

  // 处理 rename: old -> new
  if ((x === 'R' || y === 'R') && body.includes(' -> ')) {
    const [originalPath, path] = body.split(' -> ')
    const type = 'renamed'

    return {
      raw,
      x,
      y,
      path,
      originalPath,
      type,
      staged: x !== ' ',
      unstaged: y !== ' ',
    }
  }

  if (unmerged) {
    return {
      raw,
      x,
      y,
      path: body,
      type: 'updated-but-unmerged',
      staged: false,
      unstaged: true,
    }
  }

  const statusChar = x !== ' ' ? x : y
  const type = mapStatusCharToType(statusChar)

  return {
    raw,
    x,
    y,
    path: body,
    type,
    staged: x !== ' ',
    unstaged: y !== ' ',
  }
}

export function parseGitStatusList(lines: string[]): GitFileStatus[] {
  return lines.filter(line => !!line).map(line => parseGitStatusLine(line))
}

function isUnmergedStatus(x: string, y: string) {
  return ['DD', 'AU', 'UD', 'UA', 'DU', 'AA', 'UU'].includes(`${x}${y}`)
}
