import { createI18n } from 'vue-i18n'
import enUS from './messages/en-US'
import zhCN from './messages/zh-CN'

export type AppLocale = 'zh-CN' | 'en-US'

export const DEFAULT_LOCALE: AppLocale = 'zh-CN'
export const SUPPORTED_LOCALES: AppLocale[] = ['zh-CN', 'en-US']

const savedLocale = (localStorage.getItem('lumina.preferences.locale') as AppLocale | null) ?? DEFAULT_LOCALE

export const i18n = createI18n({
  legacy: false,
  locale: savedLocale,
  fallbackLocale: DEFAULT_LOCALE,
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})
