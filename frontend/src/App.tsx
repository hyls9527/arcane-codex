import { useEffect, useState, useCallback } from 'react'
import { TopBar } from './components/layout/TopBar'
import { Sidebar } from './components/layout/Sidebar'
import { MainLayout } from './components/layout/MainLayout'
import { useThemeStore } from './stores/useThemeStore'
import { ImageGrid } from './components/gallery/ImageGrid'
import { ImageViewer } from './components/gallery/ImageViewer'
import { DropZone } from './components/gallery/DropZone'
import { SettingsPage } from './components/settings/SettingsPage'
import { AIProgressPanel } from './components/ai/AIProgressPanel'
import { DedupManager } from './components/dedup/DedupManager'
import {
  getImages,
  importImages,
  getAIStatus,
  startAIProcessing,
  pauseAIProcessing,
  resumeAIProcessing,
  retryFailedAI,
  scanDuplicates,
  deleteDuplicates,
  type AIStatus,
  type DuplicateGroup,
} from './lib/api'

type Page = 'gallery' | 'settings' | 'ai' | 'dedup'

interface AppImage {
  id: number
  thumbnail_path: string
  file_name: string
  ai_tags?: string[]
  ai_status?: 'pending' | 'processing' | 'completed' | 'failed'
  file_path?: string
  width?: number
  height?: number
  file_size?: number
}

function App() {
  const [currentPage, setCurrentPage] = useState<Page>('gallery')
  const { theme } = useThemeStore()
  const [images, setImages] = useState<AppImage[]>([])
  const [loading, setLoading] = useState(true)

  // ImageViewer state
  const [viewingImage, setViewingImage] = useState<AppImage | null>(null)

  // AI Processing state
  const [aiStatus, setAiStatus] = useState<AIStatus>({
    status: 'idle',
    total: 0,
    completed: 0,
    failed: 0,
    retrying: 0,
  })
  const [aiLoading, setAiLoading] = useState(false)

  // Dedup state
  const [dedupGroups, setDedupGroups] = useState<DuplicateGroup[]>([])
  const [dedupLoading, setDedupLoading] = useState(false)

  const loadImages = useCallback(async () => {
    try {
      setLoading(true)
      const result = await getImages({ page: 1, page_size: 100 })
      if (result && Array.isArray(result)) {
        setImages(result)
      }
    } catch (err) {
      console.error('Failed to load images:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  const loadAIStatus = useCallback(async () => {
    try {
      const status = await getAIStatus()
      setAiStatus(status)
    } catch (err) {
      console.error('Failed to load AI status:', err)
    }
  }, [])

  const handleFilesSelected = useCallback(async (files: File[]) => {
    if (files.length === 0) return
    try {
      const paths = files.map(f => (f as any).path || f.name)
      await importImages(paths)
      await loadImages()
    } catch (err) {
      console.error('Failed to import images:', err)
    }
  }, [loadImages])

  const handleImageClick = useCallback((id: number) => {
    const image = images.find(img => img.id === id)
    if (image) {
      setViewingImage(image)
    }
  }, [images])

  const handleViewerClose = useCallback(() => {
    setViewingImage(null)
  }, [])

  const handleViewerDelete = useCallback(async (id: number) => {
    try {
      const { deleteImages } = await import('./lib/api')
      await deleteImages([id])
      setViewingImage(null)
      await loadImages()
    } catch (err) {
      console.error('Failed to delete image:', err)
    }
  }, [loadImages])

  useEffect(() => {
    loadImages()
    loadAIStatus()
  }, [loadImages, loadAIStatus])

  useEffect(() => {
    if (theme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      document.documentElement.classList.toggle('dark', prefersDark)

      const listener = (e: MediaQueryListEvent) => {
        document.documentElement.classList.toggle('dark', e.matches)
      }
      window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', listener)
      return () => window.matchMedia('(prefers-color-scheme: dark)').removeEventListener('change', listener)
    } else {
      document.documentElement.classList.toggle('dark', theme === 'dark')
    }
  }, [theme])

  return (
    <div className="h-screen w-screen overflow-hidden bg-background text-foreground">
      <MainLayout>
        <Sidebar onNavigate={setCurrentPage} currentPage={currentPage} />
        <div className="flex flex-col flex-1">
          <TopBar />
          <main className="flex-1 overflow-auto p-4">
            {currentPage === 'gallery' && (
              loading ? (
                <div className="flex items-center justify-center h-full text-gray-500">加载中...</div>
              ) : (
                <>
                  <div className="mb-4">
                    <DropZone onFilesSelected={handleFilesSelected} />
                  </div>
                  <div className="h-[calc(100%-200px)]">
                    <ImageGrid
                      images={images}
                      onImageClick={handleImageClick}
                    />
                  </div>
                </>
              )
            )}
            {currentPage === 'settings' && <SettingsPage />}
            {currentPage === 'ai' && (
              <div className="max-w-2xl mx-auto">
                <AIProgressPanel
                  status={aiStatus}
                  isLoading={aiLoading}
                  onStart={async () => {
                    setAiLoading(true)
                    try {
                      await startAIProcessing()
                      await loadAIStatus()
                    } catch (err) {
                      console.error('Failed to start AI processing:', err)
                    } finally {
                      setAiLoading(false)
                    }
                  }}
                  onPause={async () => {
                    try {
                      await pauseAIProcessing()
                      await loadAIStatus()
                    } catch (err) {
                      console.error('Failed to pause AI processing:', err)
                    }
                  }}
                  onResume={async () => {
                    try {
                      await resumeAIProcessing()
                      await loadAIStatus()
                    } catch (err) {
                      console.error('Failed to resume AI processing:', err)
                    }
                  }}
                  onRetry={async () => {
                    try {
                      await retryFailedAI()
                      await loadAIStatus()
                    } catch (err) {
                      console.error('Failed to retry AI:', err)
                    }
                  }}
                />
              </div>
            )}
            {currentPage === 'dedup' && (
              <DedupManager
                groups={dedupGroups}
                isLoading={dedupLoading}
                onScan={async (threshold) => {
                  setDedupLoading(true)
                  try {
                    const groups = await scanDuplicates(threshold)
                    setDedupGroups(groups)
                  } catch (err) {
                    console.error('Failed to scan duplicates:', err)
                  } finally {
                    setDedupLoading(false)
                  }
                }}
                onDelete={async (groupIds) => {
                  try {
                    const groupsToDelete = dedupGroups.filter(g => groupIds.includes(g.id))
                    const idsToDelete = groupsToDelete.flatMap(g => g.image_ids || g.images?.map(img => img.id) || [])
                    if (idsToDelete.length > 0) {
                      await deleteDuplicates(idsToDelete)
                      await loadImages()
                      // Remove deleted groups
                      setDedupGroups(prev => prev.filter(g => !groupIds.includes(g.id)))
                    }
                  } catch (err) {
                    console.error('Failed to delete duplicates:', err)
                  }
                }}
              />
            )}
          </main>
        </div>
      </MainLayout>

      {/* Image Viewer Modal */}
      {viewingImage && (
        <ImageViewer
          image={{
            id: viewingImage.id,
            file_path: viewingImage.thumbnail_path || '',
            file_name: viewingImage.file_name,
            width: viewingImage.width,
            height: viewingImage.height,
            ai_tags: viewingImage.ai_tags,
          }}
          onClose={handleViewerClose}
          onDelete={handleViewerDelete}
        />
      )}
    </div>
  )
}

export default App