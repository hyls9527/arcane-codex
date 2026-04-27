// Tauri API Integration Layer
// This module provides a clean interface to communicate with the Rust backend

import { invoke } from '@tauri-apps/api/core'

// ===== Image Management =====

export interface ImportResult {
  success_count: number
  duplicate_count: number
  error_count: number
  image_ids: number[]
}

export async function importImages(filePaths: string[]): Promise<ImportResult> {
  return invoke<ImportResult>('import_images', { filePaths })
}

export interface ImageQuery {
  page: number
  page_size: number
  filters?: {
    ai_status?: string
    date_from?: string
    date_to?: string
    category?: string
    tags?: string[]
  }
}

export async function getImages(query: ImageQuery) {
  return invoke('get_images', {
    page: query.page,
    pageSize: query.page_size,
    filters: query.filters,
  })
}

export async function getImageDetail(id: number) {
  return invoke('get_image_detail', { id })
}

export async function deleteImages(ids: number[]): Promise<number> {
  return invoke<number>('delete_images', { ids })
}

// ===== AI Processing =====

export interface AIStatus {
  status: 'idle' | 'processing' | 'paused' | 'completed' | 'failed'
  total: number
  completed: number
  failed: number
  retrying: number
  eta_seconds?: number
}

export async function startAIProcessing(concurrency?: number): Promise<void> {
  return invoke('start_ai_processing', { concurrency })
}

export async function pauseAIProcessing(): Promise<void> {
  return invoke('pause_ai_processing')
}

export async function resumeAIProcessing(): Promise<void> {
  return invoke('resume_ai_processing')
}

export async function getAIStatus(): Promise<AIStatus> {
  return invoke<AIStatus>('get_ai_status')
}

export async function retryFailedAI(): Promise<number> {
  return invoke<number>('retry_failed_ai')
}

// ===== Semantic Search =====

export interface SearchResult {
  image_id: number
  score: number
}

export async function semanticSearch(
  query: string,
  limit: number = 50
): Promise<SearchResult[]> {
  return invoke<SearchResult[]>('semantic_search', { query, limit })
}

// ===== Deduplication =====

export interface DuplicateImage {
  id: number
  thumbnail_path: string
  file_name: string
  width?: number
  height?: number
  file_size?: number
}

export interface DuplicateGroup {
  id: string
  images: DuplicateImage[]
  image_ids?: number[]
  similarity: number
  keepId?: number
}

export async function scanDuplicates(threshold: number = 90): Promise<DuplicateGroup[]> {
  return invoke<DuplicateGroup[]>('scan_duplicates', { threshold })
}

export async function deleteDuplicates(keepIds: number[]): Promise<number> {
  return invoke<number>('delete_duplicates', { keepIds })
}

// ===== Settings =====

export interface AppConfig {
  key: string
  value: string
}

export async function getConfig(key: string): Promise<AppConfig | null> {
  return invoke<AppConfig | null>('get_config', { key })
}

export async function getAllConfigs(): Promise<AppConfig[]> {
  return invoke<AppConfig[]>('get_all_configs')
}

export async function setConfig(key: string, value: string): Promise<void> {
  return invoke('set_config', { key, value })
}

// Batch set configs atomically
export async function setConfigs(entries: [string, string][]): Promise<void> {
  for (const [key, value] of entries) {
    await invoke('set_config', { key, value })
  }
}

export async function backupDatabase(outputPath: string): Promise<string> {
  return invoke<string>('backup_database', { outputPath })
}

export async function restoreDatabase(backupPath: string): Promise<void> {
  return invoke('restore_database', { backupPath })
}

export async function testLmStudioConnection(url: string): Promise<boolean> {
  return invoke<boolean>('test_lm_studio_connection', { url })
}
