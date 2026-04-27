import { useVirtualizer } from '@tanstack/react-virtual'
import { useRef } from 'react'
import { ImageCard } from './ImageCard'
import { cn } from '@/utils/cn'

interface ImageGridProps {
  images: Array<{
    id: number
    thumbnail_path: string
    file_name: string
    ai_tags?: string[]
    ai_status?: 'pending' | 'processing' | 'completed' | 'failed'
  }>
  selectedIds?: Set<number>
  onImageClick?: (id: number) => void
  onToggleSelect?: (id: number) => void
  className?: string
  columnCount?: number
  gapSize?: number
}

export function ImageGrid({
  images,
  selectedIds = new Set(),
  onImageClick,
  onToggleSelect,
  className,
  columnCount: _columnCount = 4,
  gapSize: _gapSize = 4,
}: ImageGridProps) {
  const parentRef = useRef<HTMLDivElement>(null)
  
  const virtualizer = useVirtualizer({
    count: images.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 250, // Estimated row height
    overscan: 5,
  })
  
  if (images.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
        <p>暂无图片，拖拽文件开始导入</p>
      </div>
    )
  }
  
  return (
    <div
      ref={parentRef}
      className={cn('overflow-auto h-full scrollbar-thin', className)}
    >
      <div
        className="relative"
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
        }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const image = images[virtualRow.index]
          if (!image) return null
          
          return (
            <div
              key={image.id}
              data-index={virtualRow.index}
              ref={virtualizer.measureElement}
              className={cn(
                'absolute left-0 right-0',
                'grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4'
              )}
              style={{
                transform: `translateY(${virtualRow.start}px)`,
              }}
            >
              <ImageCard
                id={image.id}
                src={image.thumbnail_path || ''}
                fileName={image.file_name}
                tags={image.ai_tags || []}
                aiStatus={image.ai_status}
                isSelected={selectedIds.has(image.id)}
                onClick={onImageClick}
                onToggleSelect={onToggleSelect}
              />
            </div>
          )
        })}
      </div>
    </div>
  )
}
