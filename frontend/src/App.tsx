import { useEffect, useState, useCallback } from 'react'
import { TopBar } from './components/layout/TopBar'
import { Sidebar } from './components/layout/Sidebar'
import { MainLayout } from './components/layout/MainLayout'
import { useThemeStore } from './stores/useThemeStore'
import { ImageGrid } from './components/gallery/ImageGrid'
import { SettingsPage } from './components/settings/SettingsPage'
import { getImages } from './lib/api'

type Page = 'gallery' | 'settings' | 'ai' | 'dedup'

interface AppImage {
  id: number
  thumbnail_path: string
  file_name: string
  ai_tags?: string[]
  ai_status?: 'pending' | 'processing' | 'completed' | 'failed'
}

function App() {
  const [currentPage, setCurrentPage] = useState<Page>('gallery')
  const { theme } = useThemeStore()
  const [images, setImages] = useState<AppImage[]>([])
  const [loading, setLoading] = useState(true)

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

  useEffect(() => {
    loadImages()
  }, [loadImages])

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
                <ImageGrid images={images} />
              )
            )}
            {currentPage === 'settings' && <SettingsPage />}
            {currentPage === 'ai' && <div className="p-8 text-center text-gray-500">AI 打标模块开发中</div>}
            {currentPage === 'dedup' && <div className="p-8 text-center text-gray-500">去重模块开发中</div>}
          </main>
        </div>
      </MainLayout>
    </div>
  )
}

export default App