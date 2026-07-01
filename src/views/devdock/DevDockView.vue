<template>
  <div class="devdock-page">
    <WorkbenchTopbar>
      <WorkbenchIdentity :label="t('devdock.overview.eyebrow')" :value="t('devdock.overview.title')" />
      <WorkbenchSwitch
        active-key="devdock"
        :aria-label="t('workbench.switcherLabel')"
        :items="workbenchSwitchItems"
        @select="handleWorkbenchSelect"
      />
      <div class="toolbar-tags" :aria-label="t('devdock.projects.summary', { count: projects.length, scanned: scannedCount })">
        <WorkbenchTag :label="t('devdock.projects.totalLabel')" :value="projects.length" />
        <WorkbenchTag :label="t('devdock.projects.scannedLabel')" :value="scannedCount" tone="primary" />
      </div>

      <template #actions>
        <WorkbenchButton :disabled="!projects.length || loadingAll" @click="scanAllProjects">
          {{ loadingAll ? t('devdock.actions.scanning') : t('devdock.actions.scanAll') }}
        </WorkbenchButton>
        <WorkbenchButton variant="primary" @click="handleAddProject">
          {{ t('devdock.actions.addProject') }}
        </WorkbenchButton>
      </template>
    </WorkbenchTopbar>

    <section class="devdock-shell">
      <main class="project-list-panel">
        <header class="panel-header">
          <div>
            <span>{{ t('devdock.projects.title') }}</span>
            <strong>{{ t('devdock.projects.subtitle') }}</strong>
          </div>
        </header>

        <section v-if="!projects.length" class="empty-page">
          <strong>{{ t('devdock.empty.title') }}</strong>
          <p>{{ t('devdock.empty.description') }}</p>
          <button class="tool-btn primary" type="button" @click="handleAddProject">
            {{ t('devdock.actions.addProject') }}
          </button>
        </section>

        <section v-else class="project-list">
          <article v-for="project in sortedProjects" :key="project.path" class="project-row">
            <header class="project-row-summary" role="button" tabindex="0" @click="toggleProject(project.path)" @keydown.enter.prevent="toggleProject(project.path)" @keydown.space.prevent="toggleProject(project.path)">
              <button class="expand-btn" type="button" :aria-expanded="expandedProjects.has(project.path)" tabindex="-1">
                <Icon :icon="expandedProjects.has(project.path) ? 'solar:alt-arrow-down-linear' : 'solar:alt-arrow-right-linear'" />
              </button>

              <div class="project-identity">
                <div v-if="editingAliasPath === project.path" class="project-alias-edit">
                  <input
                    :ref="el => setAliasInputRef(el, project.path)"
                    class="project-alias-input"
                    type="text"
                    :value="project.name"
                    :placeholder="t('devdock.project.aliasPlaceholder')"
                    @input="event => renameProject(project.path, (event.target as HTMLInputElement).value)"
                    @blur="finishEditAlias(project)"
                    @keydown.enter="finishEditAlias(project)"
                    @keydown.escape="cancelEditAlias(project)"
                    @click.stop
                    @mousedown.stop
                    @keydown.stop
                  />
                </div>
                <template v-else>
                  <span class="project-alias-text" :title="project.name || project.path">{{ project.name || t('devdock.project.aliasPlaceholder') }}</span>
                </template>
                <span :title="project.path">{{ project.path }}</span>
              </div>

              <div class="project-row-actions" @click.stop>
                <button class="row-action" type="button" @click="startEditAlias(project.path)">
                  {{ t('devdock.actions.rename') }}
                </button>
                <button class="row-action" type="button" :disabled="project.loading" @click="scanProject(project, { touch: true })">
                  {{ project.loading ? t('devdock.actions.scanning') : t('devdock.actions.scan') }}
                </button>
                <button class="row-action danger" type="button" @click="removeProject(project.path)">
                  {{ t('devdock.actions.removeProject') }}
                </button>
              </div>
            </header>

            <section v-if="project.error" class="project-error">
              <span>{{ project.error }}</span>
              <button type="button" :aria-label="t('common.dismiss')" @click="project.error = ''">
                <Icon icon="solar:close-circle-linear" />
              </button>
            </section>

            <section v-if="expandedProjects.has(project.path)" class="project-row-details">
              <div class="project-meta">
                <span>{{ project.manifest?.version || '--' }}</span>
                <span>{{ dependencyLabel(project) }}</span>
              </div>

              <div v-if="project.loading" class="project-empty">
                {{ t('devdock.actions.scanning') }}
              </div>
              <div v-else-if="!project.manifest" class="project-empty">
                <span>{{ t('devdock.empty.noManifest') }}</span>
              </div>
              <div v-else-if="project.manifest.scripts.length" class="script-list">
                <article v-for="script in sortedScripts(project)" :key="script.name" class="script-row">
                  <button
                    class="pin-btn"
                    type="button"
                    :class="{ active: isScriptPinned(project.path, script.name) }"
                    :aria-label="isScriptPinned(project.path, script.name) ? t('devdock.actions.unpinScript') : t('devdock.actions.pinScript')"
                    @click="togglePinnedScript(project.path, script.name)"
                  >
                    <Icon :icon="isScriptPinned(project.path, script.name) ? 'solar:pin-bold' : 'solar:pin-linear'" />
                  </button>
                  <div>
                    <strong>{{ script.name }}</strong>
                    <code>{{ script.command }}</code>
                  </div>
                  <button
                    class="script-run-btn"
                    type="button"
                    :class="{ running: isScriptRunning(project.path, script.name) }"
                    :disabled="isScriptStarting(project.path, script.name) || isScriptRunning(project.path, script.name)"
                    @click="startScript(project, script)"
                  >
                    {{ scriptButtonLabel(project.path, script.name) }}
                  </button>
                </article>
              </div>
              <div v-else class="project-empty">
                {{ t('devdock.scripts.empty') }}
              </div>
            </section>
          </article>
        </section>
      </main>

      <aside class="process-panel">
        <header class="panel-header">
          <div>
            <span>{{ t('devdock.processes.title') }}</span>
            <strong>{{ t('devdock.processes.subtitle') }}</strong>
          </div>
          <button class="recent-command-btn" type="button" :aria-label="t('devdock.recentCommands.open')" @click="recentDrawerOpen = true">
            <Icon icon="solar:history-linear" />
            <b>{{ recentCommands.length }}</b>
          </button>
        </header>

        <section v-if="!processes.length" class="process-empty">
          <strong>{{ t('devdock.processes.emptyTitle') }}</strong>
          <p>{{ t('devdock.processes.emptyDescription') }}</p>
        </section>
        <section v-else class="process-list">
          <article v-for="process in processes" :key="process.id" class="process-row">
            <div class="process-row-head">
              <div class="process-row-main">
                <strong>{{ process.projectName }} · {{ process.scriptName }}</strong>
                <span>
                  {{ processStatusLabel(process) }} · {{ t('devdock.processes.pid', { pid: process.pid }) }}
                  <template v-if="process.ports.length"> · {{ t('devdock.processes.ports', { ports: process.ports.join(', ') }) }}</template>
                </span>
              </div>
              <button class="process-open-btn" type="button" :disabled="!processUrl(process)" @click="openProcessUrl(process)">
                {{ t('devdock.actions.open') }}
              </button>
            </div>
            <div class="process-link-row">
              <code>{{ processUrl(process) || process.command }}</code>
              <button type="button" :disabled="!processUrl(process)" @click="copyProcessUrl(process)">{{ t('devdock.actions.copy') }}</button>
            </div>
            <div class="process-actions">
              <button type="button" @click="openProcessLogs(process.id)">{{ t('devdock.actions.logs') }}</button>
              <button type="button" :disabled="isProcessBusy(process.id)" @click="restartProcess(process.id)">{{ t('devdock.actions.restart') }}</button>
              <button class="danger" type="button" :disabled="process.status.state !== 'running' || isProcessBusy(process.id)" @click="stopProcess(process.id)">
                {{ t('devdock.actions.stop') }}
              </button>
            </div>
          </article>
        </section>
      </aside>
    </section>

    <WorkbenchDrawer
      v-if="recentDrawerOpen"
      fixed
      size="narrow"
      :title="t('devdock.recentCommands.title')"
      :description="t('devdock.recentCommands.description')"
      :close-label="t('common.dismiss')"
      @close="recentDrawerOpen = false"
    >
      <section v-if="recentCommands.length" class="recent-command-list">
        <article v-for="command in recentCommands" :key="`${command.projectPath}:${command.scriptName}`" class="recent-command-row">
          <div>
            <strong>{{ command.projectName }} · {{ command.scriptName }}</strong>
            <code>{{ command.command }}</code>
          </div>
          <button
            class="script-run-btn"
            type="button"
            :class="{ running: isRecentCommandRunning(command) }"
            :disabled="isRecentCommandRunning(command)"
            @click="startRecentCommand(command)"
          >
            {{ isRecentCommandRunning(command) ? t('devdock.actions.running') : t('devdock.actions.run') }}
          </button>
        </article>
      </section>
      <section v-else class="process-empty">
        <strong>{{ t('devdock.recentCommands.emptyTitle') }}</strong>
        <p>{{ t('devdock.recentCommands.emptyDescription') }}</p>
      </section>
    </WorkbenchDrawer>

    <WorkbenchDrawer
      v-if="logDrawerOpen"
      fixed
      size="log"
      :title="`${processLogs?.process.projectName ?? ''} · ${processLogs?.process.scriptName ?? ''}`"
      :description="processLogDescription"
      :close-label="t('common.dismiss')"
      @close="closeProcessLogs"
    >
      <section v-if="processLogs" class="process-log-list wb-log" aria-live="polite">
        <pre v-for="line in visibleLogLines" :key="`${line.timestamp}:${line.stream}:${line.text}`" class="wb-log-line" :class="line.stream"><span class="wb-log-stream">{{ line.showStream ? line.stream : '' }}</span><span v-html="renderLogLine(line.text)" /></pre>
        <pre v-if="!visibleLogLines.length" class="wb-log-line log-pending"><span class="wb-log-stream">WAIT</span><span>{{ t('devdock.processes.waitingLogs') }}</span></pre>
      </section>
      <section v-else class="process-empty">
        <strong>{{ t('devdock.processes.emptyLogsTitle') }}</strong>
        <p>{{ t('devdock.processes.emptyLogsDescription') }}</p>
      </section>
    </WorkbenchDrawer>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import { AnsiUp } from 'ansi_up'
import { useLocale } from '@/hooks/useLocale'
import { GIT_RECENT_REPOS_STORAGE_KEY, GIT_REPO_STORAGE_KEY } from '@/constants/git'
import WorkbenchButton from '@/components/workbench/WorkbenchButton.vue'
import WorkbenchDrawer from '@/components/workbench/WorkbenchDrawer.vue'
import WorkbenchIdentity from '@/components/workbench/WorkbenchIdentity.vue'
import WorkbenchSwitch from '@/components/workbench/WorkbenchSwitch.vue'
import WorkbenchTag from '@/components/workbench/WorkbenchTag.vue'
import WorkbenchTopbar from '@/components/workbench/WorkbenchTopbar.vue'
import {
  listProjectProcesses,
  loadProjectManifest,
  loadProjectProcessLogs,
  openProjectUrl,
  restartProjectProcess,
  startProjectProcess,
  stopProjectProcess,
  type ProjectManifest,
  type ProjectProcessLogs,
  type ProjectProcessSnapshot,
  type ProjectScript,
} from '@/services/project/project-service'

const DEVDOC_PINNED_SCRIPTS_STORAGE_KEY = 'lumina.devdock.pinnedScripts'
const DEVDOC_RECENT_COMMANDS_STORAGE_KEY = 'lumina.devdock.recentCommands'
const SCRIPT_PRIORITY = ['dev', 'serve', 'start', 'tauri:dev', 'preview', 'build', 'test', 'lint']
const ansiUp = new AnsiUp()
ansiUp.escape_html = true

interface DevDockProject {
  path: string
  name: string
  loading: boolean
  error: string
  manifest: ProjectManifest | null
  openedAt: number
}

interface StoredProject {
  path?: string
  name?: string
  openedAt?: number
}

interface RecentCommand {
  projectPath: string
  projectName: string
  scriptName: string
  command: string
  packageManager: string
  usedAt: number
}

const { t } = useLocale()
const router = useRouter()
const projects = ref<DevDockProject[]>([])
const expandedProjects = reactive(new Set<string>())
const pinnedScripts = ref(new Set<string>())
const recentCommands = ref<RecentCommand[]>([])
const recentDrawerOpen = ref(false)
const processes = ref<ProjectProcessSnapshot[]>([])
const processBusy = reactive(new Set<string>())
const startingScripts = reactive(new Set<string>())
const processLogs = ref<ProjectProcessLogs | null>(null)
const logDrawerOpen = ref(false)
const editingAliasPath = ref<string | null>(null)
const aliasInputRefs = new Map<string, HTMLInputElement>()
let processPollTimer: ReturnType<typeof window.setInterval> | undefined
let logPollTimer: ReturnType<typeof window.setInterval> | undefined
const workbenchSwitchItems = computed(() => [
  { key: 'git', label: t('workbench.git') },
  { key: 'devdock', label: t('workbench.devdock') },
])
const loadingAll = computed(() => projects.value.some(project => project.loading))
const scannedCount = computed(() => projects.value.filter(project => project.manifest).length)
const sortedProjects = computed(() => [...projects.value].sort((left, right) => right.openedAt - left.openedAt))
const processLogDescription = computed(() => {
  if (!processLogs.value) return ''
  const ports = processLogs.value.process.ports
  return ports.length
    ? `${processLogs.value.process.command} · ${t('devdock.processes.ports', { ports: ports.join(', ') })}`
    : processLogs.value.process.command
})
const visibleLogLines = computed(() => {
  const lines = processLogs.value?.lines.filter(line => line.text.trim()) ?? []
  return lines.map((line, index) => ({
    ...line,
    showStream: index === 0 || lines[index - 1].stream !== line.stream,
  }))
})

onMounted(() => {
  pinnedScripts.value = loadPinnedScripts()
  recentCommands.value = loadRecentCommands()
  projects.value = loadStoredProjects()
  void scanAllProjects()
  void refreshProcesses()
  processPollTimer = window.setInterval(() => {
    void refreshProcesses()
  }, 3000)
})

onUnmounted(() => {
  if (processPollTimer) {
    window.clearInterval(processPollTimer)
  }
  stopLogPolling()
})

function openGitAssistant() {
  router.push({ name: 'git-assistant' })
}

function handleWorkbenchSelect(key: string) {
  if (key === 'git') {
    openGitAssistant()
  }
}

async function handleAddProject() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: t('devdock.actions.addProject'),
  })

  if (typeof selected !== 'string') return
  const project = rememberProject(selected)
  expandedProjects.add(project.path)
  await scanProject(project, { touch: true })
}

async function scanAllProjects() {
  await Promise.all(projects.value.map(project => scanProject(project)))
}

async function scanProject(project: DevDockProject, options: { touch?: boolean } = {}) {
  if (options.touch) {
    touchProject(project)
  }
  project.loading = true
  project.error = ''
  try {
    project.manifest = await loadProjectManifest(project.path)
  } catch (err) {
    project.manifest = null
    project.error = err instanceof Error ? err.message : String(err)
  } finally {
    project.loading = false
  }
}

function toggleProject(path: string) {
  const project = projects.value.find(project => project.path === path)
  if (project) touchProject(project)
  if (expandedProjects.has(path)) {
    expandedProjects.delete(path)
    return
  }
  expandedProjects.add(path)
}

function renameProject(path: string, name: string) {
  const normalized = normalizePath(path)
  const project = projects.value.find(project => normalizePath(project.path) === normalized)
  if (project) {
    project.name = name
  }
}

function normalizeProjectAlias(project: DevDockProject) {
  project.name = project.name.trim() || project.manifest?.name || getProjectDisplayName(project.path)
  persistProjects()
}

function setAliasInputRef(el: HTMLInputElement | null, path: string) {
  if (el) {
    aliasInputRefs.set(path, el)
  } else {
    aliasInputRefs.delete(path)
  }
}

function startEditAlias(path: string) {
  editingAliasPath.value = path
  nextTick(() => {
    const input = aliasInputRefs.get(path)
    if (input) {
      input.focus()
      input.select()
    }
  })
}

function finishEditAlias(project: DevDockProject) {
  normalizeProjectAlias(project)
  editingAliasPath.value = null
}

function cancelEditAlias(project: DevDockProject) {
  editingAliasPath.value = null
}

function removeProject(path: string) {
  const normalized = normalizePath(path)
  expandedProjects.delete(path)
  projects.value = projects.value.filter(project => normalizePath(project.path) !== normalized)
  persistProjects()
}

function rememberProject(path: string) {
  const normalized = normalizePath(path)
  const existing = projects.value.find(project => normalizePath(project.path) === normalized)
  if (existing) return existing

  const project: DevDockProject = {
    path,
    name: getProjectDisplayName(path),
    loading: false,
    error: '',
    manifest: null,
    openedAt: Date.now(),
  }
  projects.value = [project, ...projects.value]
  localStorage.setItem(GIT_REPO_STORAGE_KEY, path)
  persistProjects()
  return project
}

function loadStoredProjects() {
  const stored = readRecentProjects()
  const current = localStorage.getItem(GIT_REPO_STORAGE_KEY)
  if (current) {
    stored.unshift({ path: current, name: getProjectDisplayName(current) })
  }

  const seen = new Set<string>()
  return stored
    .filter(project => typeof project.path === 'string' && project.path.trim())
    .filter(project => {
      const normalized = normalizePath(project.path || '')
      if (seen.has(normalized)) return false
      seen.add(normalized)
      return true
    })
    .map(project => ({
      path: project.path as string,
      name: project.name || getProjectDisplayName(project.path as string),
      loading: false,
      error: '',
      manifest: null,
      openedAt: typeof project.openedAt === 'number' ? project.openedAt : 0,
    }))
    .sort((left, right) => right.openedAt - left.openedAt)
}

function readRecentProjects(): StoredProject[] {
  try {
    const raw = localStorage.getItem(GIT_RECENT_REPOS_STORAGE_KEY)
    return raw ? (JSON.parse(raw) as StoredProject[]) : []
  } catch (err) {
    console.error(err)
    return []
  }
}

function persistProjects() {
  const stored = projects.value.map(project => ({
    path: project.path,
    name: project.name || getProjectDisplayName(project.path),
    openedAt: project.openedAt,
  }))
  localStorage.setItem(GIT_RECENT_REPOS_STORAGE_KEY, JSON.stringify(stored))
}

function touchProject(project: DevDockProject) {
  project.openedAt = Date.now()
  localStorage.setItem(GIT_REPO_STORAGE_KEY, project.path)
  persistProjects()
}

function sortedScripts(project: DevDockProject) {
  const scripts = project.manifest?.scripts ?? []
  return [...scripts].sort((left, right) => {
    const leftPinned = isScriptPinned(project.path, left.name)
    const rightPinned = isScriptPinned(project.path, right.name)
    if (leftPinned !== rightPinned) return leftPinned ? -1 : 1

    const leftPriority = getScriptPriority(left.name)
    const rightPriority = getScriptPriority(right.name)
    if (leftPriority !== rightPriority) return leftPriority - rightPriority

    return left.name.localeCompare(right.name)
  })
}

function getScriptPriority(name: string) {
  const index = SCRIPT_PRIORITY.indexOf(name)
  return index === -1 ? SCRIPT_PRIORITY.length : index
}

function getScriptKey(projectPath: string, scriptName: string) {
  return `${normalizePath(projectPath)}::${scriptName}`
}

function isScriptPinned(projectPath: string, scriptName: string) {
  return pinnedScripts.value.has(getScriptKey(projectPath, scriptName))
}

function togglePinnedScript(projectPath: string, scriptName: string) {
  const next = new Set(pinnedScripts.value)
  const key = getScriptKey(projectPath, scriptName)
  if (next.has(key)) {
    next.delete(key)
  } else {
    next.add(key)
  }
  pinnedScripts.value = next
  localStorage.setItem(DEVDOC_PINNED_SCRIPTS_STORAGE_KEY, JSON.stringify([...next]))
}

function loadPinnedScripts() {
  try {
    const raw = localStorage.getItem(DEVDOC_PINNED_SCRIPTS_STORAGE_KEY)
    return new Set<string>(raw ? JSON.parse(raw) : [])
  } catch (err) {
    console.error(err)
    return new Set<string>()
  }
}

async function startScript(project: DevDockProject, script: ProjectScript) {
  recordRecentCommand(project, script)
  await startProjectCommand({
    projectPath: project.path,
    projectName: project.manifest?.name || project.name,
    scriptName: script.name,
    packageManager: project.manifest?.packageManager || 'npm',
  })
}

async function startRecentCommand(command: RecentCommand) {
  await startProjectCommand(command)
  recentDrawerOpen.value = false
}

async function startProjectCommand(command: {
  projectPath: string
  projectName?: string
  scriptName: string
  packageManager: string
}) {
  const key = getScriptKey(command.projectPath, command.scriptName)
  startingScripts.add(key)
  try {
    const process = await startProjectProcess(command)
    updateProcess(process)
    showProcessLogShell(process)
    startLogPolling(process.id)
    void refreshProcessLogs(process.id)
  } catch (err) {
    console.error(err)
  } finally {
    startingScripts.delete(key)
    await refreshProcesses()
  }
}

async function refreshProcesses() {
  try {
    processes.value = await listProjectProcesses()
  } catch (err) {
    console.error(err)
  }
}

async function stopProcess(processId: string) {
  processBusy.add(processId)
  try {
    await stopProjectProcess(processId)
    processes.value = processes.value.filter(process => process.id !== processId)
  } catch (err) {
    console.error(err)
  } finally {
    processBusy.delete(processId)
    await refreshProcesses()
  }
}

async function restartProcess(processId: string) {
  processBusy.add(processId)
  try {
    updateProcess(await restartProjectProcess(processId))
  } catch (err) {
    console.error(err)
  } finally {
    processBusy.delete(processId)
    await refreshProcesses()
  }
}

async function openProcessLogs(processId: string) {
  try {
    processLogs.value = await loadProjectProcessLogs(processId)
    logDrawerOpen.value = true
    startLogPolling(processId)
  } catch (err) {
    console.error(err)
  }
}

function showProcessLogShell(process: ProjectProcessSnapshot) {
  processLogs.value = {
    process,
    lines: [],
  }
  logDrawerOpen.value = true
}

function closeProcessLogs() {
  logDrawerOpen.value = false
  stopLogPolling()
}

function startLogPolling(processId: string) {
  stopLogPolling()
  logPollTimer = window.setInterval(() => {
    if (!logDrawerOpen.value) {
      stopLogPolling()
      return
    }
    void refreshProcessLogs(processId)
  }, 1000)
}

function stopLogPolling() {
  if (logPollTimer) {
    window.clearInterval(logPollTimer)
    logPollTimer = undefined
  }
}

async function refreshProcessLogs(processId: string) {
  try {
    processLogs.value = await loadProjectProcessLogs(processId)
    updateProcess(processLogs.value.process)
  } catch (err) {
    console.error(err)
    stopLogPolling()
  }
}

function updateProcess(process: ProjectProcessSnapshot) {
  if (process.status.state !== 'running') {
    processes.value = processes.value.filter(item => item.id !== process.id)
    return
  }
  processes.value = [process, ...processes.value.filter(item => item.id !== process.id)].sort((left, right) => right.startedAt - left.startedAt)
}

function recordRecentCommand(project: DevDockProject, script: ProjectScript) {
  touchProject(project)
  const command: RecentCommand = {
    projectPath: project.path,
    projectName: project.manifest?.name || project.name,
    scriptName: script.name,
    command: script.command,
    packageManager: project.manifest?.packageManager || 'npm',
    usedAt: Date.now(),
  }
  recentCommands.value = [
    command,
    ...recentCommands.value.filter(
      item => normalizePath(item.projectPath) !== normalizePath(command.projectPath) || item.scriptName !== command.scriptName,
    ),
  ].slice(0, 12)
  localStorage.setItem(DEVDOC_RECENT_COMMANDS_STORAGE_KEY, JSON.stringify(recentCommands.value))
}

function loadRecentCommands() {
  try {
    const raw = localStorage.getItem(DEVDOC_RECENT_COMMANDS_STORAGE_KEY)
    const parsed = raw ? (JSON.parse(raw) as RecentCommand[]) : []
    return parsed
      .filter(command => command.projectPath && command.scriptName && command.command)
      .map(command => ({
        ...command,
        packageManager: command.packageManager || 'npm',
      }))
      .sort((left, right) => right.usedAt - left.usedAt)
      .slice(0, 12)
  } catch (err) {
    console.error(err)
    return []
  }
}

function isScriptStarting(projectPath: string, scriptName: string) {
  return startingScripts.has(getScriptKey(projectPath, scriptName))
}

function isScriptRunning(projectPath: string, scriptName: string) {
  const normalized = normalizePath(projectPath)
  return processes.value.some(
    process => normalizePath(process.projectPath) === normalized && process.scriptName === scriptName && process.status.state === 'running',
  )
}

function isRecentCommandRunning(command: RecentCommand) {
  return isScriptRunning(command.projectPath, command.scriptName)
}

function scriptButtonLabel(projectPath: string, scriptName: string) {
  if (isScriptStarting(projectPath, scriptName)) return t('devdock.actions.starting')
  if (isScriptRunning(projectPath, scriptName)) return t('devdock.actions.running')
  return t('devdock.actions.run')
}

function isProcessBusy(processId: string) {
  return processBusy.has(processId)
}

function processStatusLabel(process: ProjectProcessSnapshot) {
  if (process.status.state === 'running') return t('devdock.processes.running')
  if (process.status.state === 'stopped') return t('devdock.processes.stopped')
  if (process.status.state === 'exited') return t('devdock.processes.exited', { code: process.status.exitCode ?? '--' })
  return t('devdock.processes.unknown')
}

function processUrl(process: ProjectProcessSnapshot) {
  const port = process.ports[0]
  return port ? `http://localhost:${port}` : ''
}

async function openProcessUrl(process: ProjectProcessSnapshot) {
  const url = processUrl(process)
  if (url) {
    await openProjectUrl(url)
  }
}

async function copyProcessUrl(process: ProjectProcessSnapshot) {
  const url = processUrl(process)
  if (!url) return
  await navigator.clipboard.writeText(url)
}

function renderLogLine(text: string) {
  return ansiUp.ansi_to_html(text)
}

function stackLabel(project: DevDockProject) {
  if (!project.manifest) return t('devdock.overview.stackPending')
  return project.manifest.detectedStack.length ? project.manifest.detectedStack.join(' / ') : t('devdock.overview.stackUnknown')
}

function dependencyLabel(project: DevDockProject) {
  if (!project.manifest) return '--'
  return t('devdock.overview.dependencyCount', {
    dependencies: project.manifest.dependenciesCount,
    devDependencies: project.manifest.devDependenciesCount,
  })
}

function getProjectDisplayName(path: string) {
  const normalized = path.replace(/\\/g, '/')
  const parts = normalized.split('/').filter(Boolean)
  return parts[parts.length - 1] || path
}

function normalizePath(path: string) {
  return path.replace(/\\/g, '/').toLowerCase()
}
</script>

<style scoped lang="scss">
.devdock-page {
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--lumina-bg) 82%, var(--lumina-surface-1)), var(--lumina-bg)),
    var(--lumina-bg);
  color: var(--lumina-text);
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
  padding: 8px;
}

.project-list-panel,
.process-panel {
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-lg);
  box-shadow: var(--lumina-shadow-sm);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
  overflow: hidden;
}

.toolbar-tags {
  align-items: center;
  display: flex;
  flex: 1;
  gap: 8px;
  min-width: 0;
  overflow: hidden;
}

.tool-btn,
.row-action {
  align-items: center;
  background: var(--lumina-button-secondary-bg);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-sm);
  color: var(--lumina-text);
  cursor: pointer;
  display: inline-flex;
  flex: 0 0 auto;
  font-size: 11px;
  height: 28px;
  justify-content: center;
  padding: 0 10px;
  white-space: nowrap;

  &:hover:not(:disabled) {
    background: var(--lumina-button-secondary-hover);
  }

  &:disabled {
    cursor: not-allowed;
    opacity: 0.56;
  }

  &.danger {
    color: var(--lumina-danger);

    &:hover:not(:disabled) {
      background: color-mix(in srgb, var(--lumina-danger) 8%, transparent);
    }
  }
}

.project-row-actions {
  display: flex;
  gap: 6px;
  justify-self: end;
}

.primary {
  background: var(--lumina-primary);
  border-color: var(--lumina-primary);
  color: #fff;
}

.devdock-shell {
  display: grid;
  flex: 1;
  gap: 8px;
  grid-template-columns: minmax(0, 1fr) 360px;
  min-height: 0;
}

.panel-header {
  align-items: center;
  border-bottom: 1px solid var(--lumina-card-border);
  display: flex;
  justify-content: space-between;
  min-height: 44px;
  padding: 8px 12px;

  div {
    display: grid;
    gap: 2px;
  }

  span {
    color: var(--lumina-text-secondary);
    font-size: 11px;
  }

  strong {
    font-size: 14px;
  }

  b {
    color: var(--lumina-primary);
    font-size: 18px;
  }
}

.empty-page,
.process-empty {
  align-content: center;
  color: var(--lumina-text-secondary);
  display: grid;
  gap: 10px;
  justify-content: center;
  min-height: 0;
  padding: 24px;
  text-align: center;

  strong {
    color: var(--lumina-text);
    font-size: 15px;
  }

  p {
    line-height: 1.55;
    margin: 0;
    max-width: 520px;
  }

  .tool-btn {
    justify-self: center;
  }
}

.project-list {
  min-height: 0;
  overflow: auto;
  padding: 8px;
}

.project-row {
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-md);
  overflow: hidden;

  & + & {
    margin-top: 8px;
  }
}

.project-row-summary {
  align-items: center;
  cursor: pointer;
  display: grid;
  gap: 8px;
  grid-template-columns: 28px 1fr auto;
  min-height: 46px;
  outline: none;
  padding: 7px 9px;

  &:hover,
  &:focus-visible {
    background: color-mix(in srgb, var(--lumina-primary) 5%, transparent);
  }
}

.expand-btn,
.icon-btn,
.project-error button {
  align-items: center;
  background: transparent;
  border: 0;
  border-radius: var(--lumina-radius-sm);
  color: var(--lumina-text-secondary);
  cursor: pointer;
  display: inline-flex;
  height: 26px;
  justify-content: center;
  padding: 0;
  width: 26px;

  &:hover {
    background: var(--lumina-button-secondary-hover);
    color: var(--lumina-text);
  }

  svg {
    height: 16px;
    width: 16px;
  }
}

.icon-btn:hover {
  color: var(--lumina-danger);
}

.project-identity {
  display: grid;
  gap: 4px;
  min-width: 0;

  span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  span {
    color: var(--lumina-text-secondary);
    font-size: 11px;
  }
}

.project-alias-text {
  color: var(--lumina-text);
  font-size: 13px;
  font-weight: 700;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-alias-edit {
  min-width: 0;
  width: 100%;
}

.project-alias-input {
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--lumina-radius-sm);
  color: var(--lumina-text);
  font: inherit;
  font-size: 13px;
  font-weight: 700;
  height: 26px;
  min-width: 0;
  padding: 0 6px;
  width: 100%;

  &::placeholder {
    color: var(--lumina-text-secondary);
    font-weight: 500;
  }

  &:hover {
    background: var(--lumina-surface-2);
    border-color: var(--lumina-card-border);
  }

  &:focus {
    background: var(--lumina-input-bg);
    border-color: var(--lumina-primary);
    box-shadow: 0 0 0 2px var(--lumina-accent-ring);
    outline: none;
  }
}


.project-error {
  align-items: center;
  background: color-mix(in srgb, var(--lumina-danger) 10%, var(--lumina-surface-2));
  border-top: 1px solid color-mix(in srgb, var(--lumina-danger) 24%, transparent);
  color: var(--lumina-danger);
  display: flex;
  font-size: 11px;
  gap: 8px;
  justify-content: space-between;
  min-height: 30px;
  padding: 5px 10px 5px 44px;
}

.project-row-details {
  border-top: 1px solid var(--lumina-card-border);
  display: grid;
  gap: 8px;
  grid-template-rows: auto auto;
  padding: 8px 10px 10px 44px;
}

.project-meta {
  display: flex;
  gap: 6px;
  min-width: 0;
  overflow: hidden;

  span {
    background: var(--lumina-surface-2);
    border: 1px solid var(--lumina-card-border);
    border-radius: var(--lumina-radius-sm);
    color: var(--lumina-text-secondary);
    font-size: 11px;
    min-width: 0;
    overflow: hidden;
    padding: 6px 7px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.project-empty {
  align-content: center;
  background: var(--lumina-empty-bg);
  border: 1px dashed var(--lumina-empty-border);
  border-radius: var(--lumina-radius-md);
  color: var(--lumina-text-secondary);
  display: grid;
  gap: 8px;
  justify-content: center;
  min-height: 120px;
  padding: 18px;
  text-align: center;
}

.script-list {
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-md);
  min-height: 0;
  overflow: hidden;
}

.script-row {
  align-items: center;
  border-bottom: 1px solid color-mix(in srgb, var(--lumina-card-border) 72%, transparent);
  display: grid;
  gap: 10px;
  grid-template-columns: 28px minmax(0, 1fr) 76px;
  min-height: 44px;
  padding: 7px 10px;

  div {
    display: grid;
    gap: 4px;
    min-width: 0;
  }

  strong {
    font-size: 13px;
  }

  code {
    background: transparent;
    color: color-mix(in srgb, var(--lumina-text-secondary) 82%, var(--lumina-surface-3));
    font-family: SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
    font-size: 10px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pin-btn,
  .script-run-btn {
    background: var(--lumina-button-secondary-bg);
    border: 1px solid var(--lumina-card-border);
    border-radius: var(--lumina-radius-sm);
    color: var(--lumina-text-secondary);
    height: 28px;
  }

  .pin-btn {
    align-items: center;
    cursor: pointer;
    display: inline-flex;
    justify-content: center;
    padding: 0;
    width: 28px;

    &:hover,
    &.active {
      background: color-mix(in srgb, var(--lumina-primary) 10%, var(--lumina-surface-1));
      border-color: color-mix(in srgb, var(--lumina-primary) 42%, var(--lumina-card-border));
      color: var(--lumina-primary);
    }
  }

  .script-run-btn {
    cursor: pointer;

    &:hover:not(:disabled) {
      background: var(--lumina-button-secondary-hover);
      color: var(--lumina-text);
    }

    &:disabled {
      cursor: not-allowed;
      opacity: 0.72;
    }

    &.running {
      background: color-mix(in srgb, var(--lumina-success) 12%, var(--lumina-surface-1));
      border-color: color-mix(in srgb, var(--lumina-success) 42%, var(--lumina-card-border));
      color: var(--lumina-success);
      opacity: 1;
    }
  }
}

.process-list {
  align-content: start;
  display: grid;
  gap: 6px;
  grid-auto-rows: min-content;
  min-height: 0;
  overflow: auto;
  padding: 6px;
}

.process-row {
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-sm);
  display: grid;
  gap: 6px;
  padding: 8px;
  position: relative;

  &::before {
    background: var(--lumina-success);
    border-radius: 999px;
    content: '';
    height: 6px;
    left: 7px;
    position: absolute;
    top: 12px;
    width: 6px;
  }
}

.process-row-head {
  align-items: start;
  display: grid;
  gap: 8px;
  grid-template-columns: minmax(0, 1fr) auto;
  min-width: 0;
}

.process-row-main {
  display: grid;
  gap: 3px;
  min-width: 0;

  strong,
  span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  strong {
    font-size: 12px;
    padding-left: 12px;
  }

  span {
    color: var(--lumina-text-secondary);
    font-size: 11px;
  }
}

.process-open-btn {
  background: color-mix(in srgb, var(--lumina-primary) 12%, var(--lumina-surface-1));
  border: 1px solid color-mix(in srgb, var(--lumina-primary) 46%, var(--lumina-card-border));
  border-radius: var(--lumina-radius-sm);
  color: var(--lumina-primary);
  cursor: pointer;
  font-size: 11px;
  font-weight: 650;
  height: 28px;
  min-width: 54px;
  padding: 0 10px;

  &:hover:not(:disabled) {
    background: color-mix(in srgb, var(--lumina-primary) 18%, var(--lumina-surface-1));
  }

  &:disabled {
    cursor: not-allowed;
    opacity: 0.52;
  }
}

.process-link-row {
  align-items: center;
  background: var(--lumina-surface-2);
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 72%, transparent);
  border-radius: var(--lumina-radius-sm);
  display: grid;
  gap: 6px;
  grid-template-columns: minmax(0, 1fr) auto;
  min-height: 28px;
  padding: 3px 4px 3px 8px;

  code {
    color: var(--lumina-text-secondary);
    font-family: SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
    font-size: 10px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  button {
    background: transparent;
    border: 0;
    border-radius: var(--lumina-radius-sm);
    color: var(--lumina-text-secondary);
    cursor: pointer;
    font-size: 11px;
    height: 22px;
    padding: 0 6px;

    &:hover:not(:disabled) {
      background: var(--lumina-button-secondary-hover);
      color: var(--lumina-text);
    }

    &:disabled {
      cursor: not-allowed;
      opacity: 0.5;
    }
  }
}

.process-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  justify-content: flex-end;

  button {
    background: var(--lumina-button-secondary-bg);
    border: 1px solid var(--lumina-card-border);
    border-radius: var(--lumina-radius-sm);
    color: var(--lumina-text-secondary);
    cursor: pointer;
    font-size: 11px;
    height: 26px;
    min-width: 42px;
    padding: 0 7px;

    &:hover:not(:disabled) {
      background: var(--lumina-button-secondary-hover);
      color: var(--lumina-text);
    }

    &:disabled {
      cursor: not-allowed;
      opacity: 0.56;
    }

    &.danger:hover:not(:disabled) {
      border-color: color-mix(in srgb, var(--lumina-danger) 45%, var(--lumina-card-border));
      color: var(--lumina-danger);
    }
  }
}

.recent-command-btn {
  align-items: center;
  background: var(--lumina-button-secondary-bg);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-sm);
  color: var(--lumina-text-secondary);
  cursor: pointer;
  display: inline-flex;
  gap: 6px;
  height: 28px;
  justify-content: center;
  padding: 0 8px;

  &:hover {
    background: var(--lumina-button-secondary-hover);
    color: var(--lumina-text);
  }

  svg {
    height: 15px;
    width: 15px;
  }

  b {
    color: var(--lumina-primary);
    font-size: 12px;
  }
}

.recent-command-list {
  align-content: start;
  display: grid;
  gap: 6px;
  grid-auto-rows: min-content;
  min-height: 0;
  overflow: auto;
  padding: 8px;
}

.recent-command-row {
  align-items: center;
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-sm);
  display: grid;
  gap: 8px;
  grid-template-columns: minmax(0, 1fr) 76px;
  min-height: 44px;
  padding: 6px 8px;

  div {
    display: grid;
    gap: 3px;
    min-width: 0;
  }

  strong,
  code {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  strong {
    font-size: 12px;
  }

  code {
    color: var(--lumina-text-secondary);
    font-family: SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
    font-size: 10px;
  }

  .script-run-btn {
    background: var(--lumina-button-secondary-bg);
    border: 1px solid var(--lumina-card-border);
    border-radius: var(--lumina-radius-sm);
    color: var(--lumina-text-secondary);
    cursor: pointer;
    height: 26px;

    &:hover {
      background: var(--lumina-button-secondary-hover);
      color: var(--lumina-text);
    }

    &.running {
      background: color-mix(in srgb, var(--lumina-success) 12%, var(--lumina-surface-1));
      border-color: color-mix(in srgb, var(--lumina-success) 42%, var(--lumina-card-border));
      color: var(--lumina-success);
      cursor: not-allowed;
    }
  }
}

.process-log-list {
  .stderr {
    color: #ffb4a8;
  }
}
</style>
