<template>
  <div v-if="visible" class="dialog-mask">
    <section class="command-dialog">
      <header class="dialog-header">
        <div>
          <span class="dialog-kicker">Lumina Git Operation</span>
          <h3>{{ title }}</h3>
          <p>{{ repoPath }}</p>
        </div>
        <div class="status-chip" :class="statusTone">
          <span></span>
          {{ statusLabel }}
        </div>
      </header>

      <div class="operation-stage">
        <div class="stage-node stage-node--source">
          <strong>Git</strong>
          <span>working tree</span>
        </div>
        <div class="stage-beam" :class="{ running, failed: success === false }"></div>
        <div class="stage-node stage-node--target">
          <strong>{{ phase }}</strong>
          <span>{{ commandSummary }}</span>
        </div>
      </div>

      <div class="progress-track">
        <div class="progress-bar" :class="{ running, failed: success === false }"></div>
      </div>

      <section class="log-panel">
        <div class="log-toolbar">
          <span>Command Output</span>
          <strong>{{ logLineCount }}</strong>
        </div>
        <pre class="log-box">{{ logText }}</pre>
      </section>

      <footer class="dialog-actions">
        <button
          v-if="nextActionLabel"
          class="action-btn next-action"
          :disabled="running"
          type="button"
          @click="$emit('next-action')"
        >
          {{ nextActionLabel }}
        </button>
        <span v-else class="dialog-actions__spacer"></span>
        <button class="action-btn primary" :disabled="running" type="button" @click="$emit('close')">
          {{ closeLabel }}
        </button>
        <button class="action-btn" disabled type="button">{{ abortLabel }}</button>
      </footer>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  visible: boolean
  title: string
  repoPath: string
  phase: string
  running: boolean
  success: boolean | null
  command: string
  stdout: string
  stderr: string
  message: string
  suggestion: string
  nextActionLabel: string
  closeLabel: string
  abortLabel: string
}>()

defineEmits<{
  (e: 'close'): void
  (e: 'next-action'): void
}>()

const logText = computed(() => {
  const lines = []
  if (props.command) {
    lines.push(...props.command.split('\n').map(command => `> ${command}`))
  }
  if (props.stdout.trim()) lines.push(props.stdout.trim())
  if (props.stderr.trim()) lines.push(props.stderr.trim())
  if (props.message.trim() && !props.stdout.includes(props.message)) lines.push(props.message.trim())
  if (props.suggestion.trim()) lines.push(`建议: ${props.suggestion.trim()}`)
  if (!lines.length && props.running) lines.push('Running git command...')
  return lines.join('\n\n')
})

const statusTone = computed(() => {
  if (props.running) return 'running'
  if (props.success === false) return 'failed'
  if (props.success === true) return 'done'
  return ''
})

const statusLabel = computed(() => {
  if (props.running) return 'Running'
  if (props.success === false) return 'Failed'
  if (props.success === true) return 'Completed'
  return 'Ready'
})

const commandSummary = computed(() => {
  const firstCommand = props.command.split('\n').find(Boolean)
  return firstCommand || props.message || 'preparing'
})

const logLineCount = computed(() => logText.value.split('\n').filter(Boolean).length)
</script>

<style scoped lang="scss">
.dialog-mask {
  align-items: center;
  background: rgba(18, 22, 25, 0.28);
  backdrop-filter: blur(10px);
  display: flex;
  inset: 0;
  justify-content: center;
  padding: 24px;
  position: absolute;
  z-index: 40;
}

.command-dialog {
  background: color-mix(in srgb, var(--lumina-surface-1) 92%, transparent);
  backdrop-filter: blur(22px);
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 78%, transparent);
  border-radius: 12px;
  box-shadow:
    0 24px 68px rgba(0, 0, 0, 0.24),
    inset 0 1px 0 rgba(255, 255, 255, 0.32);
  display: grid;
  gap: 14px;
  grid-template-rows: auto auto auto minmax(240px, 1fr) auto;
  max-height: min(680px, 86vh);
  overflow: hidden;
  width: min(880px, 92vw);
}

.dialog-header {
  align-items: flex-start;
  border-bottom: 1px solid var(--lumina-card-border);
  display: flex;
  gap: 14px;
  justify-content: space-between;
  padding: 16px 18px;

  .dialog-kicker {
    color: var(--lumina-primary);
    display: block;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    margin-bottom: 4px;
    text-transform: uppercase;
  }

  h3 {
    font-size: 18px;
    letter-spacing: 0;
    margin: 0 0 4px;
  }

  p {
    color: var(--lumina-text-secondary);
    font-size: 12px;
    margin: 0;
  }
}

.status-chip {
  align-items: center;
  border: 1px solid var(--lumina-card-border);
  border-radius: 7px;
  color: var(--lumina-text-secondary);
  display: inline-flex;
  font-size: 12px;
  gap: 7px;
  height: 30px;
  padding: 0 10px;

  span {
    background: currentColor;
    border-radius: 999px;
    height: 7px;
    width: 7px;
  }

  &.running {
    color: var(--lumina-primary);
  }

  &.done {
    color: #2f8f5b;
  }

  &.failed {
    color: var(--lumina-danger);
  }
}

.operation-stage {
  align-items: center;
  display: grid;
  gap: 12px;
  grid-template-columns: 150px minmax(80px, 1fr) 240px;
  padding: 0 18px;
}

.stage-node {
  background: color-mix(in srgb, var(--lumina-surface-2) 62%, transparent);
  border: 1px solid var(--lumina-card-border);
  border-radius: 9px;
  display: grid;
  gap: 4px;
  min-width: 0;
  padding: 12px;

  strong,
  span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  strong {
    color: var(--lumina-text);
    font-size: 13px;
  }

  span {
    color: var(--lumina-text-secondary);
    font-family: Consolas, 'Liberation Mono', Menlo, monospace;
    font-size: 11px;
  }
}

.stage-node--source {
  border-left: 2px solid var(--lumina-primary);
}

.stage-node--target {
  border-left: 2px solid #526a86;
}

.stage-beam {
  background: var(--lumina-card-border);
  height: 1px;
  overflow: hidden;
  position: relative;

  &.running::after {
    animation: beam-run 1.1s ease-in-out infinite;
    background: #fff;
    content: '';
    height: 2px;
    left: 0;
    position: absolute;
    top: 0;
    width: 38%;
  }

  &.failed {
    background: var(--lumina-danger);
  }
}

.progress-track {
  background: color-mix(in srgb, var(--lumina-surface-2) 62%, transparent);
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 82%, transparent);
  border-radius: 999px;
  height: 12px;
  margin: 0 18px;
  overflow: hidden;
}

.progress-bar {
  background: var(--lumina-primary);
  height: 100%;
  transition: width 0.2s ease;
  width: 100%;

  &.running {
    animation: progress-slide 1.2s linear infinite;
    width: 45%;
  }

  &.failed {
    background: var(--lumina-danger);
  }
}

.log-box {
  background: transparent;
  border: 0;
  color: var(--lumina-text);
  font-family: Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 13px;
  line-height: 1.45;
  margin: 0;
  overflow: auto;
  padding: 10px;
  white-space: pre-wrap;
}

.log-panel {
  background: color-mix(in srgb, var(--lumina-diff-bg) 94%, transparent);
  border: 1px solid var(--lumina-card-border);
  border-radius: 10px;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  margin: 0 18px;
  min-height: 0;
  overflow: hidden;
}

.log-toolbar {
  align-items: center;
  border-bottom: 1px solid var(--lumina-card-border);
  color: var(--lumina-text-secondary);
  display: flex;
  font-size: 11px;
  justify-content: space-between;
  min-height: 32px;
  padding: 0 10px;

  strong {
    color: var(--lumina-text);
    font-family: Consolas, 'Liberation Mono', Menlo, monospace;
    font-size: 11px;
  }
}

.dialog-actions {
  align-items: center;
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding: 0 18px 16px;
}

.dialog-actions__spacer {
  flex: 1 1 auto;
}

.action-btn {
  background: color-mix(in srgb, var(--lumina-button-secondary-bg) 92%, transparent);
  border: 1px solid color-mix(in srgb, var(--lumina-card-border) 86%, var(--lumina-text-secondary));
  border-radius: 8px;
  color: var(--lumina-text);
  cursor: pointer;
  height: 34px;
  padding: 0 12px;

  &:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
}

.next-action {
  margin-right: auto;
  min-width: 132px;
}

.primary {
  background: linear-gradient(180deg, color-mix(in srgb, var(--lumina-primary) 88%, #fff), var(--lumina-primary));
  border-color: var(--lumina-primary);
  box-shadow: 0 8px 18px color-mix(in srgb, var(--lumina-primary) 24%, transparent);
  color: #fff;
}

@keyframes progress-slide {
  0% {
    transform: translateX(-100%);
  }

  100% {
    transform: translateX(230%);
  }
}

@keyframes beam-run {
  0% {
    transform: translateX(-120%);
  }

  100% {
    transform: translateX(320%);
  }
}
</style>
