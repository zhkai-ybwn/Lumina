<template>
  <div v-if="visible" class="dialog-mask">
    <section class="command-dialog">
      <header class="dialog-header">
        <div>
          <h3>{{ title }}</h3>
          <p>{{ repoPath }}</p>
        </div>
      </header>

      <div class="command-icons">
        <div class="command-icon">Git</div>
        <div class="command-icon command-icon--folder">{{ phase }}</div>
      </div>

      <div class="progress-track">
        <div class="progress-bar" :class="{ running, failed: success === false }"></div>
      </div>

      <pre class="log-box">{{ logText }}</pre>

      <footer class="dialog-actions">
        <button
          v-if="nextActionLabel"
          class="action-btn"
          :disabled="running"
          type="button"
          @click="$emit('next-action')"
        >
          {{ nextActionLabel }}
        </button>
        <span v-else></span>
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
</script>

<style scoped lang="scss">
.dialog-mask {
  align-items: center;
  background: rgba(0, 0, 0, 0.24);
  display: flex;
  inset: 0;
  justify-content: center;
  padding: 24px;
  position: absolute;
  z-index: 40;
}

.command-dialog {
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: 10px;
  box-shadow: 0 18px 48px rgba(0, 0, 0, 0.22);
  display: grid;
  gap: 16px;
  grid-template-rows: auto auto auto minmax(240px, 1fr) auto;
  max-height: min(680px, 86vh);
  width: min(820px, 92vw);
}

.dialog-header {
  border-bottom: 1px solid var(--lumina-card-border);
  padding: 14px 16px;

  h3 {
    font-size: 15px;
    margin: 0 0 4px;
  }

  p {
    color: var(--lumina-text-secondary);
    font-size: 12px;
    margin: 0;
  }
}

.command-icons {
  display: flex;
  gap: 64px;
  padding: 0 18px;
}

.command-icon {
  align-items: center;
  background: var(--lumina-surface-2);
  border: 1px solid var(--lumina-card-border);
  border-radius: 8px;
  color: var(--lumina-primary);
  display: inline-flex;
  font-size: 12px;
  font-weight: 700;
  height: 44px;
  justify-content: center;
  min-width: 58px;
  padding: 0 10px;
}

.command-icon--folder {
  color: var(--lumina-text);
}

.progress-track {
  background: var(--lumina-surface-2);
  border: 1px solid var(--lumina-card-border);
  height: 30px;
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
  background: var(--lumina-diff-bg);
  border: 1px solid var(--lumina-card-border);
  color: var(--lumina-text);
  font-family: Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 13px;
  line-height: 1.45;
  margin: 0 18px;
  overflow: auto;
  padding: 10px;
  white-space: pre-wrap;
}

.dialog-actions {
  align-items: center;
  display: grid;
  gap: 12px;
  grid-template-columns: 1fr 120px 120px;
  padding: 0 18px 16px;
}

.action-btn {
  background: var(--lumina-button-secondary-bg);
  border: 1px solid var(--lumina-card-border);
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

.primary {
  background: var(--lumina-primary);
  border-color: var(--lumina-primary);
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
</style>
