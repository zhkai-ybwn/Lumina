import { defineStore } from 'pinia'
import { i18n, type AppLocale } from '@/i18n'

export type ThemeMode = 'light' | 'dark' | 'system'

const STORAGE_KEYS = {
  locale: 'lumina.preferences.locale',
  themeMode: 'lumina.preferences.themeMode',
} as const

let systemThemeMedia: MediaQueryList | null = null

function resolveSystemTheme() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export const usePreferencesStore = defineStore('preferences', {
  state: () => ({
    locale: (localStorage.getItem(STORAGE_KEYS.locale) as AppLocale | null) ?? 'zh-CN',
    themeMode: (localStorage.getItem(STORAGE_KEYS.themeMode) as ThemeMode | null) ?? 'system',
  }),

  getters: {
    resolvedTheme: state => (state.themeMode === 'system' ? resolveSystemTheme() : state.themeMode),
  },

  actions: {
    initPreferences() {
      this.syncLocale()
      this.syncThemeToDom()
      this.bindSystemTheme()
    },

    setLocale(locale: AppLocale) {
      this.locale = locale
      localStorage.setItem(STORAGE_KEYS.locale, locale)
      this.syncLocale()
    },

    setThemeMode(mode: ThemeMode) {
      this.themeMode = mode
      localStorage.setItem(STORAGE_KEYS.themeMode, mode)
      this.syncThemeToDom()
    },

    syncLocale() {
      i18n.global.locale.value = this.locale
      document.documentElement.setAttribute('lang', this.locale)
    },

    syncThemeToDom() {
      document.documentElement.setAttribute('data-theme', this.resolvedTheme)
      document.documentElement.setAttribute('data-theme-mode', this.themeMode)
    },

    bindSystemTheme() {
      if (systemThemeMedia) return

      systemThemeMedia = window.matchMedia('(prefers-color-scheme: dark)')
      systemThemeMedia.addEventListener('change', () => {
        if (this.themeMode === 'system') {
          this.syncThemeToDom()
        }
      })
    },
  },
})
