import { invoke } from '@tauri-apps/api/core'

export function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error)
}

export function reportError(context: string, error: unknown) {
  const message = errorMessage(error)
  console.error(`[${context}]`, error)
  void invoke('log_frontend_error', { context, message }).catch(() => undefined)
  return message
}
