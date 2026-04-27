import { useState, useCallback, useRef } from 'react'
import { X, ZoomIn, ZoomOut, RotateCw, Download, Trash2 } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from '@/utils/cn'

interface ImageViewerProps {
  image: {
    id: number
    file_path: string
    file_name: string
    width?: number
    height?: number
    ai_tags?: string[]
    ai_description?: string
    ai_category?: string
    exif_data?: Record<string, any>
  }
  onClose: () => void
  onDelete?: (id: number) => void
  onExport?: (id: number) => void
}

export function ImageViewer({
  image,
  onClose,
  onDelete,
  onExport,
}: ImageViewerProps) {
  const [scale, setScale] = useState(1)
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const [isDragging, setIsDragging] = useState(false)
  const dragStart = useRef({ x: 0, y: 0 })
  
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault()
    const delta = e.deltaY > 0 ? -0.1 : 0.1
    setScale(prev => Math.max(0.1, Math.min(5, prev + delta)))
  }, [])
  
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (scale > 1) {
      setIsDragging(true)
      dragStart.current = { x: e.clientX - position.x, y: e.clientY - position.y }
    }
  }, [scale, position])
  
  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (isDragging) {
      setPosition({
        x: e.clientX - dragStart.current.x,
        y: e.clientY - dragStart.current.y,
      })
    }
  }, [isDragging])
  
  const handleMouseUp = useCallback(() => {
    setIsDragging(false)
  }, [])
  
  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-50 bg-black/90 flex"
        onClick={onClose}
      >
        {/* Close Button */}
        <button
          onClick={onClose}
          className="absolute top-4 right-4 p-2 rounded-full bg-white/10 hover:bg-white/20 text-white z-50"
          aria-label="关闭预览"
        >
          <X className="w-6 h-6" />
        </button>
        
        {/* Image Container */}
        <div
          className="flex-1 flex items-center justify-center overflow-hidden"
          onWheel={handleWheel}
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
          onClick={(e) => e.stopPropagation()}
        >
          <motion.img
            src={image.file_path}
            alt={image.file_name}
            className={cn(
              'max-w-full max-h-full object-contain transition-transform',
              isDragging && 'cursor-grabbing'
            )}
            style={{
              transform: `translate(${position.x}px, ${position.y}px) scale(${scale})`,
            }}
          />
        </div>
        
        {/* Info Panel */}
        <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-6">
          <div className="max-w-4xl mx-auto text-white">
            <h2 className="text-xl font-semibold mb-2">{image.file_name}</h2>
            
            {/* AI Description */}
            {image.ai_description && (
              <p className="text-gray-200 mb-3">{image.ai_description}</p>
            )}
            
            {/* AI Tags */}
            {image.ai_tags && image.ai_tags.length > 0 && (
              <div className="flex flex-wrap gap-2 mb-3">
                {image.ai_tags.map((tag, i) => (
                  <span
                    key={i}
                    className="px-3 py-1 bg-white/20 rounded-full text-sm"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            )}
            
            {/* Metadata */}
            <div className="flex items-center gap-4 text-sm text-gray-300">
              {image.width && image.height && (
                <span>{image.width} x {image.height}</span>
              )}
              {image.ai_category && (
                <span>分类: {image.ai_category}</span>
              )}
            </div>
          </div>
        </div>
        
        {/* Toolbar */}
        <div className="absolute top-4 left-4 flex items-center gap-2">
          <button
            onClick={() => setScale(prev => Math.max(0.1, prev - 0.2))}
            className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white"
            aria-label="缩小"
          >
            <ZoomOut className="w-5 h-5" />
          </button>
          
          <span className="text-white px-2">{Math.round(scale * 100)}%</span>
          
          <button
            onClick={() => setScale(prev => Math.min(5, prev + 0.2))}
            className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white"
            aria-label="放大"
          >
            <ZoomIn className="w-5 h-5" />
          </button>
          
          <button
            onClick={() => {
              setScale(1)
              setPosition({ x: 0, y: 0 })
            }}
            className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white"
            aria-label="重置缩放"
          >
            <RotateCw className="w-5 h-5" />
          </button>
          
          <div className="w-px h-6 bg-white/20 mx-2" />
          
          {onExport && (
            <button
              onClick={() => onExport(image.id)}
              className="p-2 rounded-full bg-white/10 hover:bg-white/20 text-white"
              aria-label="导出"
            >
              <Download className="w-5 h-5" />
            </button>
          )}
          
          {onDelete && (
            <button
              onClick={() => onDelete(image.id)}
              className="p-2 rounded-full bg-red-500/20 hover:bg-red-500/40 text-red-400"
              aria-label="删除"
            >
              <Trash2 className="w-5 h-5" />
            </button>
          )}
        </div>
      </motion.div>
    </AnimatePresence>
  )
}
