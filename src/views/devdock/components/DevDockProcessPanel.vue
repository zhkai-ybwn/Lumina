<template>
  <aside class="process-panel">
    <header class="panel-header">
      <div>
        <span>{{ t('devdock.processes.title') }}</span>
        <strong>{{ t('devdock.processes.subtitle') }}</strong>
      </div>
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
          <button class="process-open-btn" type="button" :disabled="!processUrl(process)" @click="$emit('openUrl', process)">
            {{ t('devdock.actions.open') }}
          </button>
        </div>
        <div class="process-link-row">
          <code>{{ processUrl(process) || process.command }}</code>
          <button type="button" :disabled="!processUrl(process)" @click="$emit('copyUrl', process)">{{ t('devdock.actions.copy') }}</button>
        </div>
        <div class="process-actions">
          <button type="button" @click="$emit('openLogs', process.id)">{{ t('devdock.actions.logs') }}</button>
          <button type="button" :disabled="isBusy(process.id)" @click="$emit('restart', process.id)">{{ t('devdock.actions.restart') }}</button>
          <button class="danger" type="button" :disabled="process.status.state !== 'running' || isBusy(process.id)" @click="$emit('stop', process.id)">
            {{ t('devdock.actions.stop') }}
          </button>
        </div>
      </article>
    </section>
  </aside>
</template>

<script setup lang="ts">
import { useLocale } from '@/hooks/useLocale'
import type { ProjectProcessSnapshot } from '@/services/project/project-service'

defineProps<{
  isBusy: (processId: string) => boolean
  processStatusLabel: (process: ProjectProcessSnapshot) => string
  processUrl: (process: ProjectProcessSnapshot) => string
  processes: ProjectProcessSnapshot[]
}>()

defineEmits<{
  (e: 'copyUrl', process: ProjectProcessSnapshot): void
  (e: 'openLogs', processId: string): void
  (e: 'openUrl', process: ProjectProcessSnapshot): void
  (e: 'restart', processId: string): void
  (e: 'stop', processId: string): void
}>()

const { t } = useLocale()
</script>

<style scoped lang="scss">
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
}

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
</style>
