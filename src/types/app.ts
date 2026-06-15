export interface AppConfig {
  id: string
  name: string
  icon: string
  groupId: string
  order: number
  enabled: boolean
  pinned?: boolean
  route?: string
}