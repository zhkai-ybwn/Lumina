import { invoke } from '@tauri-apps/api/core'
import type { AiModelConfig } from '@/types/ai-settings'

export async function testAiModelConnection(model: AiModelConfig): Promise<string> {
  return invoke<string>('test_ai_model_connection', { model })
}

