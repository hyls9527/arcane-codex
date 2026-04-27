import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { type AppImage } from '../types/image'
import { type SearchResult } from '../lib/api'

export interface ImageState {
  images: AppImage[]
  selectedIds: Set<number>
  filters: {
    ai_status?: string
    date_from?: string
    date_to?: string
    category?: string
    tags?: string[]
  }
  page: number
  pageSize: number
  total: number
  loading: boolean
  error: string | null
  searchQuery: string
  searchResults: SearchResult[]
  searchLoading: boolean
  hasSearched: boolean

  setImages: (images: AppImage[]) => void
  setSelectedIds: (ids: Set<number>) => void
  toggleSelect: (id: number) => void
  selectAll: () => void
  deselectAll: () => void
  setFilters: (filters: Partial<ImageState['filters']>) => void
  setPage: (page: number) => void
  setPageSize: (pageSize: number) => void
  setTotal: (total: number) => void
  addImages: (images: AppImage[]) => void
  removeImages: (ids: number[]) => void
  setLoading: (loading: boolean) => void
  setError: (error: string | null) => void
  setSearchQuery: (query: string) => void
  setSearchResults: (results: SearchResult[]) => void
  setSearchLoading: (loading: boolean) => void
  setHasSearched: (searched: boolean) => void
  clearSearch: () => void
  loadImages: () => Promise<void>
}

export const useImageStore = create<ImageState>()(
  persist(
    (set, get) => ({
      images: [],
      selectedIds: new Set(),
      filters: {},
      page: 1,
      pageSize: 50,
      total: 0,
      loading: false,
      error: null,
      searchQuery: '',
      searchResults: [],
      searchLoading: false,
      hasSearched: false,

      setImages: (images) => set({ images }),
      setSelectedIds: (selectedIds) => set({ selectedIds }),
      toggleSelect: (id) => set((state) => {
        const newSelected = new Set(state.selectedIds)
        if (newSelected.has(id)) {
          newSelected.delete(id)
        } else {
          newSelected.add(id)
        }
        return { selectedIds: newSelected }
      }),
      selectAll: () => set((state) => ({
        selectedIds: new Set(state.images.map(img => img.id))
      })),
      deselectAll: () => set({ selectedIds: new Set() }),
      setFilters: (filters) => set((state) => ({
        filters: { ...state.filters, ...filters },
        page: 1,
      })),
      setPage: (page) => set({ page }),
      setPageSize: (pageSize) => set({ pageSize, page: 1 }),
      setTotal: (total) => set({ total }),
      addImages: (images) => set((state) => ({
        images: [...images, ...state.images]
      })),
      removeImages: (ids) => set((state) => ({
        images: state.images.filter(img => !ids.includes(img.id)),
        selectedIds: new Set([...state.selectedIds].filter(id => !ids.includes(id)))
      })),
      setLoading: (loading) => set({ loading }),
      setError: (error) => set({ error }),
      setSearchQuery: (searchQuery) => set({ searchQuery }),
      setSearchResults: (searchResults) => set({ searchResults }),
      setSearchLoading: (searchLoading) => set({ searchLoading }),
      setHasSearched: (hasSearched) => set({ hasSearched }),
      clearSearch: () => set({ searchQuery: '', searchResults: [], searchLoading: false, hasSearched: false }),
      loadImages: async () => {
        const { getImages } = await import('../lib/api')
        const { filters, page, pageSize } = get()
        const hasFilters = filters.ai_status || filters.category || filters.date_from || filters.date_to || (filters.tags && filters.tags.length > 0)
        try {
          set({ loading: true, error: null })
          const result = await getImages({
            page,
            page_size: pageSize,
            filters: hasFilters ? filters : undefined,
          })
          if (result && Array.isArray(result)) {
            set({ images: result, loading: false })
          } else {
            set({ error: 'common.loadFailed', loading: false })
          }
        } catch (err) {
          const message = err instanceof Error ? err.message : 'common.unknownError'
          set({ error: `errors.loadImagesFailed: ${message}`, loading: false })
        }
      },
    }),
    {
      name: 'image-store',
      partialize: (state) => ({
        filters: state.filters,
        pageSize: state.pageSize,
      }),
    }
  )
)
