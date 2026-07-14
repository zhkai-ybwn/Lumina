<template>
  <section class="panel-section">
    <h2>{{ t('settings.closeBehavior.title') }}</h2>
    <p>{{ t('settings.closeBehavior.description') }}</p>

    <div
      class="close-action-options"
      role="radiogroup"
      :aria-label="t('settings.closeBehavior.title')"
    >
      <button
        v-for="option in options"
        :key="option.value"
        class="close-action-option"
        :class="{ active: preferencesStore.closeAction === option.value }"
        type="button"
        role="radio"
        :aria-checked="preferencesStore.closeAction === option.value"
        @click="preferencesStore.setCloseAction(option.value)"
      >
        <span class="close-action-option__icon" aria-hidden="true"
          ><Icon :icon="option.icon"
        /></span>
        <span class="close-action-option__content">
          <strong>{{ option.label }}</strong>
          <small>{{ option.hint }}</small>
        </span>
        <span class="close-action-option__indicator" aria-hidden="true"></span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePreferencesStore, type CloseAction } from '@/stores/preferences'

const { t } = useI18n({ useScope: 'global' })
const preferencesStore = usePreferencesStore()

const options = computed(() => [
  {
    value: 'ask' as CloseAction,
    icon: 'solar:question-circle-linear',
    label: t('settings.closeBehavior.ask'),
    hint: t('settings.closeBehavior.askHint'),
  },
  {
    value: 'hideToTray' as CloseAction,
    icon: 'solar:monitor-smartphone-linear',
    label: t('settings.closeBehavior.hideToTray'),
    hint: t('settings.closeBehavior.hideToTrayHint'),
  },
  {
    value: 'exit' as CloseAction,
    icon: 'solar:power-linear',
    label: t('settings.closeBehavior.exit'),
    hint: t('settings.closeBehavior.exitHint'),
  },
])
</script>

<style scoped lang="scss">
.panel-section {
  background: var(--lumina-card-bg);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-lg);
  box-shadow: var(--lumina-shadow-sm);
  max-width: 760px;
  padding: var(--lumina-gap-lg);

  h2 {
    font-size: 18px;
    margin: 0 0 var(--lumina-gap-sm);
  }

  > p {
    color: var(--lumina-text-secondary);
    line-height: 1.55;
    margin: 0 0 var(--lumina-gap-lg);
  }
}

.close-action-options {
  display: grid;
  gap: 10px;
}

.close-action-option {
  align-items: center;
  background: var(--lumina-surface-1);
  border: 1px solid var(--lumina-card-border);
  border-radius: var(--lumina-radius-md);
  color: var(--lumina-text);
  cursor: pointer;
  display: flex;
  gap: 12px;
  padding: 14px;
  text-align: left;
  transition:
    background 0.15s ease,
    border-color 0.15s ease,
    box-shadow 0.15s ease;
  width: 100%;

  &:hover {
    background: var(--lumina-surface-2);
    border-color: var(--lumina-input-border);
  }

  &:focus-visible {
    box-shadow: 0 0 0 3px var(--lumina-accent-ring);
    outline: none;
  }

  &.active {
    background: color-mix(in srgb, var(--lumina-primary) 8%, var(--lumina-surface-1));
    border-color: var(--lumina-primary);

    .close-action-option__indicator {
      background: var(--lumina-primary);
      border-color: var(--lumina-primary);
      box-shadow: inset 0 0 0 3px var(--lumina-surface-1);
    }
  }
}

.close-action-option__icon {
  align-items: center;
  background: var(--lumina-surface-2);
  border-radius: var(--lumina-radius-sm);
  color: var(--lumina-primary);
  display: inline-flex;
  flex: 0 0 auto;
  height: 34px;
  justify-content: center;
  width: 34px;

  svg {
    height: 18px;
    width: 18px;
  }
}

.close-action-option__content {
  display: grid;
  gap: 3px;
  min-width: 0;

  strong {
    font-size: 14px;
    font-weight: 600;
  }

  small {
    color: var(--lumina-text-secondary);
    font-size: 12px;
    line-height: 1.45;
  }
}

.close-action-option__indicator {
  border: 1px solid var(--lumina-input-border);
  border-radius: 50%;
  height: 16px;
  margin-left: auto;
  width: 16px;
}
</style>
