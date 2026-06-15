<template>
  <section class="change-explorer">
    <header class="change-header">
      <div>
        <span>{{ t('gitAssistant.files.title') }}</span>
        <small>{{ t('gitAssistant.files.refreshHint') }}</small>
      </div>
      <strong>{{ t('gitAssistant.files.selectedTotal', { selected: reviewSelectedRaws.length, total: totalCount }) }}</strong>
    </header>

    <div class="check-toolbar">
      <span>{{ t('gitAssistant.files.check') }}:</span>
      <button type="button" @click="$emit('set-review-selection', visibleRaws)">
        {{ t('gitAssistant.files.selectAll') }}
      </button>
      <button type="button" @click="$emit('set-review-selection', [])">
        {{ t('gitAssistant.files.selectNone') }}
      </button>
      <button type="button" @click="$emit('set-review-selection', unversionedRaws)">
        {{ t('gitAssistant.files.filters.untracked') }}
      </button>
      <button type="button" @click="$emit('set-review-selection', versionedRaws)">
        {{ t('gitAssistant.files.filters.versioned') }}
      </button>
      <button type="button" @click="$emit('set-review-selection', addedRaws)">
        {{ t('gitAssistant.files.filters.added') }}
      </button>
      <button type="button" @click="$emit('set-review-selection', deletedRaws)">
        {{ t('gitAssistant.files.filters.deleted') }}
      </button>
      <button type="button" @click="$emit('set-review-selection', modifiedRaws)">
        {{ t('gitAssistant.files.filters.modified') }}
      </button>
      <button type="button" @click="$emit('set-review-selection', visibleRaws)">
        {{ t('gitAssistant.files.filters.files') }}
      </button>
    </div>

    <div v-if="!hasSnapshot && !loading" class="panel-empty">{{ t('gitAssistant.files.emptyNoRepo') }}</div>
    <div v-else-if="loading" class="panel-empty">{{ t('gitAssistant.files.emptyLoading') }}</div>
    <div v-else-if="!groups.length" class="panel-empty">{{ t('gitAssistant.files.emptyNoMatch') }}</div>

    <div v-else class="file-table">
      <div class="table-header" :style="gridStyle">
        <span class="check-cell header-cell">
          <input
            ref="headerCheckbox"
            :checked="allVisibleSelected"
            type="checkbox"
            @change="$emit('set-review-selection', allVisibleSelected ? [] : visibleRaws)"
          />
        </span>
        <span
          v-for="column in resizableColumns"
          :key="column.key"
          class="header-cell"
        >
          {{ t(column.labelKey) }}
          <i class="column-resizer" @mousedown.prevent="startColumnResize(column.key, $event)"></i>
        </span>
      </div>

      <div
        v-for="file in visibleFiles"
        :key="file.raw"
        class="file-row"
        :class="{ active: activeFileRaw === file.raw }"
        :style="gridStyle"
        @dblclick="$emit('select-file', file.raw)"
        @click="$emit('select-file', file.raw)"
      >
        <label class="commit-check" @click.stop>
          <input
            :checked="reviewSelectedRaws.includes(file.raw)"
            type="checkbox"
            @change="
              $emit('toggle-review-selection', {
                raw: file.raw,
                checked: ($event.target as HTMLInputElement).checked,
              })
            "
          />
        </label>

        <div class="path-cell mono" :title="file.path">
          <span v-if="file.recommended" class="attention-dot" :title="t('gitAssistant.files.recommended')"></span>
          {{ file.path }}
        </div>
        <div class="extension-cell mono">{{ file.extension || '-' }}</div>
        <div class="status-cell" :class="`tone-${statusMeta[file.type].tone}`">
          {{ t(statusMeta[file.type].labelKey) }}
        </div>
        <div class="line-cell added-lines">{{ formatLineCount(file.addedLines) }}</div>
        <div class="line-cell removed-lines">{{ formatLineCount(file.removedLines) }}</div>
        <div class="score-cell">{{ file.score }}</div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onUnmounted, reactive, ref, watchEffect } from 'vue'
import { useLocale } from '@/hooks/useLocale'
import { STATUS_META } from '../git-assistant.config'
import type {
  GitAssistantFileGroup,
  GitAssistantStatusFilter,
  GitAssistantSummary,
} from '../git-assistant.types'

const props = defineProps<{
  hasSnapshot: boolean
  loading: boolean
  keyword: string
  statusFilter: GitAssistantStatusFilter
  recommendedOnly: boolean
  summary: GitAssistantSummary
  groups: GitAssistantFileGroup[]
  filteredCount: number
  totalCount: number
  activeFileRaw: string | null
  reviewSelectedRaws: string[]
}>()

defineEmits<{
  (e: 'update:keyword', value: string): void
  (e: 'update:status-filter', value: string): void
  (e: 'update:recommended-only', value: boolean): void
  (e: 'select-file', raw: string): void
  (e: 'toggle-review-selection', payload: { raw: string; checked: boolean }): void
  (e: 'set-review-selection', raws: string[]): void
}>()

const statusMeta = STATUS_META
const { t } = useLocale()

const visibleFiles = computed(() => props.groups.flatMap(group => group.files))
const visibleRaws = computed(() => visibleFiles.value.map(file => file.raw))
const selectedRawSet = computed(() => new Set(props.reviewSelectedRaws))
const selectedVisibleCount = computed(() => visibleRaws.value.filter(raw => selectedRawSet.value.has(raw)).length)
const allVisibleSelected = computed(() => visibleRaws.value.length > 0 && selectedVisibleCount.value === visibleRaws.value.length)
const partiallyVisibleSelected = computed(() => selectedVisibleCount.value > 0 && !allVisibleSelected.value)
const unversionedRaws = computed(() => visibleFiles.value.filter(file => file.type === 'untracked').map(file => file.raw))
const versionedRaws = computed(() => visibleFiles.value.filter(file => file.type !== 'untracked').map(file => file.raw))
const addedRaws = computed(() => visibleFiles.value.filter(file => file.type === 'added').map(file => file.raw))
const deletedRaws = computed(() => visibleFiles.value.filter(file => file.type === 'deleted').map(file => file.raw))
const modifiedRaws = computed(() => visibleFiles.value.filter(file => file.type === 'modified').map(file => file.raw))
const headerCheckbox = ref<HTMLInputElement | null>(null)
const columnWidths = reactive({
  path: 620,
  extension: 92,
  status: 96,
  added: 78,
  removed: 78,
  score: 58,
})

const resizableColumns = [
  { key: 'path', labelKey: 'gitAssistant.files.tablePath' },
  { key: 'extension', labelKey: 'gitAssistant.files.tableExtension' },
  { key: 'status', labelKey: 'gitAssistant.files.tableStatus' },
  { key: 'added', labelKey: 'gitAssistant.files.tableAdded' },
  { key: 'removed', labelKey: 'gitAssistant.files.tableRemoved' },
  { key: 'score', labelKey: 'gitAssistant.files.tableScore' },
] as const

type ResizableColumnKey = keyof typeof columnWidths

const gridStyle = computed(() => ({
  gridTemplateColumns: `34px minmax(160px, ${columnWidths.path}px) ${columnWidths.extension}px ${columnWidths.status}px ${columnWidths.added}px ${columnWidths.removed}px ${columnWidths.score}px`,
}))

watchEffect(() => {
  if (headerCheckbox.value) {
    headerCheckbox.value.indeterminate = partiallyVisibleSelected.value
  }
})

let resizingColumn: ResizableColumnKey | null = null
let resizeStartX = 0
let resizeStartWidth = 0

function startColumnResize(column: ResizableColumnKey, event: MouseEvent) {
  resizingColumn = column
  resizeStartX = event.clientX
  resizeStartWidth = columnWidths[column]
  window.addEventListener('mousemove', handleColumnResize)
  window.addEventListener('mouseup', stopColumnResize)
}

function handleColumnResize(event: MouseEvent) {
  if (!resizingColumn) return
  const nextWidth = resizeStartWidth + event.clientX - resizeStartX
  columnWidths[resizingColumn] = Math.max(48, nextWidth)
}

function stopColumnResize() {
  resizingColumn = null
  window.removeEventListener('mousemove', handleColumnResize)
  window.removeEventListener('mouseup', stopColumnResize)
}

function formatLineCount(value: number | null) {
  return value === null ? '-' : String(value)
}

onUnmounted(stopColumnResize)
</script>

<style scoped lang="scss">
.change-explorer {
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: 10px;
  box-shadow: var(--lumina-shadow-sm);
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr);
  min-height: 0;
}

.change-header {
  align-items: center;
  border-bottom: 1px solid var(--lumina-card-border);
  display: flex;
  justify-content: space-between;
  min-height: 34px;
  padding: 0 10px;

  div {
    align-items: center;
    display: flex;
    gap: 8px;
    min-width: 0;
  }

  span {
    font-size: 13px;
    font-weight: 700;
  }

  small,
  strong {
    color: var(--lumina-text-secondary);
    font-size: 11px;
    font-weight: 500;
  }
}

.check-toolbar {
  align-items: center;
  border-bottom: 1px solid var(--lumina-card-border);
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  min-height: 32px;
  padding: 0 10px;

  span {
    color: var(--lumina-text-secondary);
    font-size: 12px;
  }

  button {
    color: var(--lumina-text);
    cursor: pointer;
    font-size: 12px;
    font-weight: 700;
    padding: 0;

    &:hover {
      color: var(--lumina-primary);
      text-decoration: underline;
      text-underline-offset: 2px;
    }
  }
}

.panel-empty {
  align-items: center;
  background: var(--lumina-empty-bg);
  border: 1px dashed var(--lumina-empty-border);
  border-radius: 8px;
  color: var(--lumina-text-secondary);
  display: flex;
  font-size: 12px;
  justify-content: center;
  margin: 10px;
  padding: 18px;
  text-align: center;
}

.file-table {
  min-height: 0;
  overflow: auto;
}

.table-header,
.file-row {
  display: grid;
}

.table-header {
  background: var(--lumina-surface-2);
  border-bottom: 1px solid var(--lumina-card-border);
  color: var(--lumina-text-secondary);
  font-size: 11px;
  height: 28px;
  position: sticky;
  top: 0;
  z-index: 1;
}

.header-cell {
  align-items: center;
  border-right: 1px solid var(--lumina-card-border);
  display: flex;
  min-width: 0;
  padding: 0 8px;
  position: relative;
}

.column-resizer {
  bottom: 0;
  cursor: col-resize;
  position: absolute;
  right: -3px;
  top: 0;
  width: 6px;
  z-index: 2;
}

.file-row {
  border-bottom: 1px solid color-mix(in srgb, var(--lumina-card-border) 72%, transparent);
  cursor: default;
  min-height: 28px;

  &:hover {
    background: var(--lumina-button-secondary-hover);
  }

  &.active {
    background: color-mix(in srgb, var(--lumina-primary-soft) 76%, transparent);
  }
}

.commit-check,
.path-cell,
.extension-cell,
.status-cell,
.line-cell,
.score-cell {
  align-items: center;
  border-right: 1px solid color-mix(in srgb, var(--lumina-card-border) 72%, transparent);
  display: flex;
  min-width: 0;
}

.commit-check {
  justify-content: center;
}

.path-cell {
  color: var(--lumina-text);
  font-size: 12px;
  gap: 7px;
  overflow: hidden;
  padding: 0 8px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.extension-cell,
.status-cell,
.line-cell,
.score-cell {
  font-size: 11px;
  padding: 0 8px;
}

.line-cell {
  justify-content: flex-end;
}

.added-lines {
  color: var(--lumina-success);
}

.removed-lines {
  color: var(--lumina-danger);
}

.score-cell {
  color: var(--lumina-text-secondary);
  justify-content: flex-end;
}

.attention-dot {
  background: var(--lumina-primary);
  border-radius: 999px;
  box-shadow: 0 0 0 3px var(--lumina-primary-soft);
  flex: 0 0 auto;
  height: 6px;
  width: 6px;
}

.tone-warning {
  color: var(--lumina-warning);
}

.tone-success {
  color: var(--lumina-success);
}

.tone-danger {
  color: var(--lumina-danger);
}

.tone-info {
  color: var(--lumina-primary);
}

.tone-muted {
  color: var(--lumina-text-secondary);
}

.mono {
  font-family: SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
}
</style>
