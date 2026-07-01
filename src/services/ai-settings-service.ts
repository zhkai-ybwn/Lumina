import { invoke } from '@tauri-apps/api/core'
import type { AiModelConfig, AiSettings } from '@/types/ai-settings'

export async function testAiModelConnection(model: AiModelConfig): Promise<string> {
  return invoke<string>('test_ai_model_connection', { model })
}

export async function loadAiSettingsFile(): Promise<AiSettings | null> {
  return invoke<AiSettings | null>('load_ai_settings')
}

export async function saveAiSettingsFile(settings: AiSettings): Promise<void> {
  return invoke<void>('save_ai_settings', { settings })
}
