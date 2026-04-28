import { useEffect, useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { ErrorBoundary } from './components/common/ErrorBoundary'
import { TopBar } from './components/layout/TopBar'
import { Sidebar } from './components/layout/Sidebar'
import { MainLayout } from './components/layout/MainLayout'
import { useThemeStore } from './stores/useThemeStore'
import { useConfigStore } from './stores/useConfigStore'
import type { Theme } from './stores/useThemeStore'
import { useImageStore } from './stores/useImageStore'
import { ImageViewer } from './components/gallery/ImageViewer'
import { ImportProgressBar } from './components/gallery/ImportProgressBar'
import { SettingsPage } from './components/settings/SettingsPage'
import { LMStudioGuide } from './components/ai/LMStudioGuide'
import i18n from './i18n'
import { GalleryPage } from './pages/GalleryPage'
import { AIPage } from './pages/AIPage'
import { DedupPage } from './pages/DedupPage'
import { DashboardPage } from './pages/DashboardPage'
import {
  deleteImages,
  exportData,
  retryFailedAI,
  startAIProcessing,
  archiveImage,
  safeExport,
} from './lib/api'
import { type AppImage, type Page, type Toast } from './types/image'
import { useStateRouter } from './router/state-router'
import { navigate } from './router/events'

function App() {
  const { current: currentPage } = useStateRouter('gallery')
  const { theme } = useConfigStore()
  const { applyTheme } = useThemeStore()
  const { loadConfigs } = useConfigStore()
  const { images, loading, error, searchQuery, loadImages, setSearchQuery } = useImageStore()
  const [toasts, setToasts] = useState<Toast[]>([])
  const [viewingImage, setViewingImage] = useState<AppImage | null>(null)
  const { t } = useTranslation()

  const addToast = useCallback((message: string, type: 'error' | 'success' | 'info' = 'info') => {
    const id = Date.now()
    setToasts(prev => [...prev, { id, message, type }])
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id))
    }, 4000)
  }, [])

  useEffect(() => {
    loadConfigs().then((configs) => {
      if (configs?.language) {
        i18n.changeLanguage(configs.language)
      }
    })
  }, [loadConfigs])

  useEffect(() => {
    loadImages()
  }, [loadImages])

  useEffect(() => {
    applyTheme(theme as Theme)
  }, [theme, applyTheme])

  const handleImageClick = useCallback((image: AppImage) => {
    setViewingImage(image)
  }, [])

  const handleViewerClose = useCallback(() => {
    setViewingImage(null)
  }, [])

  const handleViewerDelete = useCallback(async (id: number) => {
    try {
      await deleteImages([id])
      setViewingImage(null)
      await loadImages()
    } catch {
      addToast(t('errors.deleteFailed'), 'error')
    }
  }, [loadImages, addToast, t])

  const handleViewerExport = useCallback(async (id: number) => {
    try {
      const result = await exportData({
        format: 'json',
        output_path: `C:\\Users\\Public\\Documents\\ArcaneCodex_Export_${id}.json`,
        image_ids: [id],
      })
      addToast(t('gallery.exportSuccess', { count: result.exported_count }), 'success')
    } catch {
      addToast(t('errors.exportFailed'), 'error')
    }
  }, [addToast, t])

  const handleViewerReAnalyze = useCallback(async (id: number) => {
    try {
      await retryFailedAI()
      await startAIProcessing()
      addToast(t('imageViewer.reAnalyzeStarted'), 'success')
    } catch {
      addToast(t('errors.reAnalyzeFailed'), 'error')
    }
  }, [addToast, t])

  const handleViewerArchive = useCallback(async (id: number) => {
    try {
      const result = await archiveImage(id)
      if (result.archived) {
        addToast(t('imageViewer.archiveSuccess', { path: result.dest_path }), 'success')
      }
    } catch (err) {
      addToast(`${t('errors.archiveFailed')}: ${err instanceof Error ? err.message : t('common.unknownError')}`, 'error')
    }
  }, [addToast, t])

  const handleViewerSafeExport = useCallback(async (id: number) => {
    try {
      const destDir = `C:\\Users\\Public\\Documents\\ArcaneCodex_Export`
      const result = await safeExport([id], destDir)
      if (result.exported_count > 0) {
        addToast(t('imageViewer.safeExportSuccess', { count: result.exported_count, dir: destDir }), 'success')
      } else {
        addToast(t('imageViewer.safeExportNoFiles'), 'info')
      }
    } catch (err) {
      addToast(`${t('errors.safeExportFailed')}: ${err instanceof Error ? err.message : t('common.unknownError')}`, 'error')
    }
  }, [addToast, t])

  const handleViewerTagClick = useCallback((tag: string) => {
    setViewingImage(null)
    setSearchQuery(tag)
    navigate({ route: 'gallery', source: 'action' })
  }, [setSearchQuery])

  const handleSearch = useCallback((query: string) => {
    setSearchQuery(query)
    navigate({ route: 'gallery', source: 'action' })
  }, [setSearchQuery])

  return (
    <ErrorBoundary>
      <div className="h-screen w-screen overflow-hidden bg-background text-foreground">
        <MainLayout>
          <Sidebar currentPage={currentPage} />
          <div className="flex flex-col flex-1">
            <TopBar onSearch={handleSearch} searchQuery={searchQuery} />
            <main className="flex-1 overflow-auto p-4">
              {currentPage === 'gallery' && (
                <GalleryPage
                  images={images}
                  loading={loading}
                  error={error}
                  onLoadImages={loadImages}
                  addToast={addToast}
                  onImageClick={handleImageClick}
                />
              )}
              {currentPage === 'settings' && <SettingsPage />}
              {currentPage === 'ai' && <AIPage addToast={addToast} />}
              {currentPage === 'dedup' && (
                <DedupPage addToast={addToast} onImagesChanged={loadImages} />
              )}
              {currentPage === 'dashboard' && <DashboardPage />}
            </main>
          </div>
        </MainLayout>
      </div>

      {viewingImage && (
        <ImageViewer
          image={{
            id: viewingImage.id,
            file_path: viewingImage.thumbnail_path || '',
            file_name: viewingImage.file_name,
            width: viewingImage.width,
            height: viewingImage.height,
            file_size: viewingImage.file_size,
            ai_tags: viewingImage.ai_tags,
            ai_description: viewingImage.ai_description,
            ai_category: viewingImage.ai_category,
            exif_data: viewingImage.exif_data,
          }}
          onClose={handleViewerClose}
          onDelete={handleViewerDelete}
          onExport={handleViewerExport}
          onReAnalyze={handleViewerReAnalyze}
          onArchive={handleViewerArchive}
          onSafeExport={handleViewerSafeExport}
          onTagClick={handleViewerTagClick}
        />
      )}

      <ImportProgressBar onComplete={() => loadImages()} />
      <LMStudioGuide />

      <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
        {toasts.map(toast => (
          <div
            key={toast.id}
            className={`px-4 py-3 rounded-lg shadow-lg text-white text-sm max-w-xs animate-slide-in ${
              toast.type === 'error' ? 'bg-red-500' :
              toast.type === 'success' ? 'bg-green-500' :
              'bg-blue-500'
            }`}
          >
            {toast.message}
          </div>
        ))}
      </div>
    </ErrorBoundary>
  )
}

export default App
