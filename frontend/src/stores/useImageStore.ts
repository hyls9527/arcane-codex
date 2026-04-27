import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export interface ImageState {
  // Image list
  images: Array<{
    id: number
    file_path: string
    file_name: string
    thumbnail_path?: string
    ai_status: string
    ai_tags?: string[]
    ai_description?: string
    created_at: string
  }>
  
  // Selection
  selectedIds: Set<number>
  
  // Filters
  filters: {
    ai_status?: string
    date_from?: string
    date_to?: string
    category?: string
    tags?: string[]
  }
  
  // Pagination
  page: number
  pageSize: number
  total: number
  
  // Actions
  setImages: (images: ImageState['images']) => void
  setSelectedIds: (ids: Set<number>) => void
  toggleSelect: (id: number) => void
  selectAll: () => void
  deselectAll: () => void
  setFilters: (filters: Partial<ImageState['filters']>) => void
  setPage: (page: number) => void
  setPageSize: (pageSize: number) => void
  setTotal: (total: number) => void
  addImages: (images: ImageState['images']) => void
  removeImages: (ids: number[]) => void
}

export const useImageStore = create<ImageState>()(
  persist(
    (set, _get) => ({
      images: [],
      selectedIds: new Set(),
      filters: {},
      page: 1,
      pageSize: 50,
      total: 0,
      
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
        page: 1, // Reset page when filters change
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
