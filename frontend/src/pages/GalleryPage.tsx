import { useCallback, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, AlertCircle, Search, Link2 } from 'lucide-react'
import { ImageGrid } from '../components/gallery/ImageGrid'
import { ImageFilter } from '../components/gallery/ImageFilter'
import { DropZone } from '../components/gallery/DropZone'
import { useImageStore } from '../stores/useImageStore'
import { type AppImage } from '../types/image'
import {
  importImages,
  searchImages,
  checkBrokenLinks,
} from '../lib/api'

interface GalleryPageProps {
  images: AppImage[]
  loading: boolean
  error: string | null
  onLoadImages: () => Promise<void>
  addToast: (message: string, type: 'error' | 'success' | 'info') => void
  onImageClick: (image: AppImage) => void
}

export function GalleryPage({
  images,
  loading,
  error,
  onLoadImages,
  addToast,
  onImageClick,
}: GalleryPageProps) {
  const { t } = useTranslation()
  const {
    filters,
    searchQuery,
    searchResults,
    searchLoading,
    hasSearched,
    setSearchQuery,
    setSearchResults,
    setSearchLoading,
    setHasSearched,
    clearSearch,
  } = useImageStore()

  const hasFilters = !!(filters.ai_status || filters.category || filters.date_from || filters.date_to || (filters.tags && filters.tags.length > 0))

  useEffect(() => {
    if (hasFilters) {
      onLoadImages()
    }
  }, [filters.ai_status, filters.category, filters.date_from, filters.date_to, filters.tags])

  const handleSearch = useCallback(async (query: string) => {
    setSearchQuery(query)
    if (!query.trim()) {
      clearSearch()
      return
    }
    try {
      setSearchLoading(true)
      const results = await searchImages(query, { page: 0, page_size: 50 })
      setSearchResults(results || [])
      setHasSearched(true)
    } catch (err) {
      addToast(`${t('errors.searchFailed')}: ${err instanceof Error ? err.message : t('common.unknownError')}`, 'error')
      setSearchResults([])
      setHasSearched(true)
    } finally {
      setSearchLoading(false)
    }
  }, [addToast, t, setSearchQuery, setSearchResults, setSearchLoading, setHasSearched, clearSearch])

  const handleFilesSelected = useCallback(async (files: File[]) => {
    if (files.length === 0) return
    try {
      const paths = files.map(f => (f as any).path || f.name)
      await importImages(paths)
      await onLoadImages()
      addToast(t('gallery.importSuccess'), 'success')
    } catch {
      addToast(t('errors.importFailed'), 'error')
    }
  }, [onLoadImages, addToast, t])

  const handleImageClick = useCallback((id: number) => {
    const image = images.find(img => img.id === id)
    if (image) onImageClick(image)
  }, [images, onImageClick])

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
        <p className="text-gray-500">{t('common.loading')}</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3 text-red-600 dark:text-red-400">
        <AlertCircle className="w-8 h-8" />
        <p className="text-sm">{error}</p>
        <button onClick={onLoadImages} className="btn-primary mt-2">{t('common.retry')}</button>
      </div>
    )
  }

  return (
    <>
      <div className="mb-4">
        <DropZone onFilesSelected={handleFilesSelected} />
      </div>
      <div className="flex items-center gap-3 mb-4">
        <ImageFilter />
        <button
          onClick={async () => {
            try {
              const result = await checkBrokenLinks()
              if (result.broken_count > 0) {
                addToast(t('gallery.brokenLinksFound', { count: result.broken_count }), 'error')
                await onLoadImages()
              } else {
                addToast(t('gallery.noBrokenLinks'), 'success')
              }
            } catch {
              addToast(t('errors.brokenLinksCheckFailed'), 'error')
            }
          }}
          className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-dark-200 transition-colors"
        >
          <Link2 className="w-4 h-4" />
          {t('gallery.checkBrokenLinks')}
        </button>
      </div>

      {hasSearched && searchQuery.trim() && (
        <div className="mb-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-lg font-medium">{t('gallery.searchResults', { query: searchQuery })}</h3>
            <span className="text-sm text-gray-500">{searchResults.length} {t('gallery.resultsCount')}</span>
          </div>
          {searchLoading ? (
            <div className="flex items-center justify-center py-8 gap-2">
              <Loader2 className="w-5 h-5 animate-spin text-primary-500" />
              <span className="text-sm text-gray-500">{t('common.searching')}</span>
            </div>
          ) : searchResults.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-8 text-gray-500">
              <Search className="w-12 h-12 mb-2 opacity-50" />
              <p className="text-sm">{t('gallery.noResults', { query: searchQuery })}</p>
            </div>
          ) : (
            <div className="h-[calc(100%-200px)]">
              <ImageGrid
                images={searchResults.map(r => ({
                  id: r.image_id,
                  thumbnail_path: '',
                  file_name: r.description || String(r.image_id),
                  ai_tags: r.tags || [],
                  ai_status: 'completed' as const,
                  ai_category: r.category || '',
                  ai_description: r.description,
                }))}
                onImageClick={handleImageClick}
              />
            </div>
          )}
        </div>
      )}

      {(!hasSearched || !searchQuery.trim()) && (
        <div className="h-[calc(100%-200px)]">
          <ImageGrid images={images} onImageClick={handleImageClick} />
        </div>
      )}
    </>
  )
}
