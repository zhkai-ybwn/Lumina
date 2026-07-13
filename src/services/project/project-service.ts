import { invoke } from '@tauri-apps/api/core'

export interface ProjectManifest {
  projectPath: string
  packageJsonPath: string
  name?: string | null
  version?: string | null
  packageManager: string
  scripts: ProjectScript[]
  dependenciesCount: number
  devDependenciesCount: number
  detectedStack: string[]
}

export interface ProjectScript {
  name: string
  command: string
}

export interface ProjectProcessStatus {
  state: 'running' | 'exited' | 'stopped' | 'unknown'
  exitCode?: number | null
  exitedAt?: number | null
}

export interface ProjectProcessSnapshot {
  id: string
  projectPath: string
  projectName: string
  scriptName: string
  command: string
  packageManager: string
  pid: number
  status: ProjectProcessStatus
  startedAt: number
  exitedAt?: number | null
  exitCode?: number | null
  ports: number[]
  urls: string[]
  logCount: number
  lastLogLine?: string | null
}

export interface ProjectProcessLogLine {
  stream: 'stdout' | 'stderr'
  text: string
  timestamp: number
}

export interface ProjectProcessLogs {
  process: ProjectProcessSnapshot
  lines: ProjectProcessLogLine[]
}

export async function loadProjectManifest(projectPath: string): Promise<ProjectManifest> {
  return await invoke<ProjectManifest>('load_project_manifest', { projectPath })
}

export async function startProjectProcess(payload: {
  projectPath: string
  projectName?: string
  scriptName: string
  packageManager: string
}): Promise<ProjectProcessSnapshot> {
  return await invoke<ProjectProcessSnapshot>('start_project_process', { payload })
}

export async function listProjectProcesses(): Promise<ProjectProcessSnapshot[]> {
  return await invoke<ProjectProcessSnapshot[]>('list_project_processes')
}

export async function stopProjectProcess(processId: string): Promise<ProjectProcessSnapshot> {
  return await invoke<ProjectProcessSnapshot>('stop_project_process', { processId })
}

export async function restartProjectProcess(processId: string): Promise<ProjectProcessSnapshot> {
  return await invoke<ProjectProcessSnapshot>('restart_project_process', { processId })
}

export async function loadProjectProcessLogs(processId: string): Promise<ProjectProcessLogs> {
  return await invoke<ProjectProcessLogs>('load_project_process_logs', { processId })
}

export async function openProjectUrl(url: string): Promise<void> {
  await invoke('open_project_url', { url })
}

export async function stopAllProjectProcesses(): Promise<ProjectProcessSnapshot[]> {
  return await invoke<ProjectProcessSnapshot[]>('stop_all_project_processes')
}
