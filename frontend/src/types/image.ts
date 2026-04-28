export type AIStatusEnum = 'pending' | 'processing' | 'completed' | 'failed'

export interface AppImage {
  id: number
  file_path: string
  file_name: string
  thumbnail_path?: string
  ai_status: AIStatusEnum | string
  ai_tags?: string[]
  ai_description?: string
  ai_category?: string
  created_at?: string
  width?: number
  height?: number
  file_size?: number
  exif_data?: Record<string, string | number | undefined>
}

export type Page = 'gallery' | 'settings' | 'ai' | 'dedup' | 'dashboard'

export interface Toast {
  id: number
  message: string
  type: 'error' | 'success' | 'info'
}
