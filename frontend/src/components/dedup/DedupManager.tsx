import { useState } from 'react'
import { Search, Trash2, Check, ArrowRight } from 'lucide-react'
import { cn } from '@/utils/cn'
import { motion, AnimatePresence } from 'framer-motion'

interface DuplicateGroup {
  id: string
  images: Array<{
    id: number
    thumbnail_path: string
    file_name: string
    width?: number
    height?: number
    file_size?: number
  }>
  similarity: number
  keepId?: number
}

interface DedupManagerProps {
  groups?: DuplicateGroup[]
  onScan?: (threshold: number) => void
  onDelete?: (groupIds: string[]) => void
  isLoading?: boolean
}

export function DedupManager({
  groups = [],
  onScan,
  onDelete,
  isLoading = false,
}: DedupManagerProps) {
  const [threshold, setThreshold] = useState(90)
  const [currentGroupIndex, setCurrentGroupIndex] = useState(0)
  const [selectedGroups, setSelectedGroups] = useState<Set<string>>(new Set())
  const [groupDecisions, setGroupDecisions] = useState<Record<string, number>>({})
  
  const currentGroup = groups[currentGroupIndex]
  
  const handleKeepSelection = (groupId: string, imageId: number) => {
    setGroupDecisions(prev => ({
      ...prev,
      [groupId]: imageId,
    }))
  }
  
  const handleNext = () => {
    if (currentGroupIndex < groups.length - 1) {
      setCurrentGroupIndex(prev => prev + 1)
    }
  }
  
  const handlePrevious = () => {
    if (currentGroupIndex > 0) {
      setCurrentGroupIndex(prev => prev - 1)
    }
  }
  
  const handleToggleSelect = (groupId: string) => {
    setSelectedGroups(prev => {
      const next = new Set(prev)
      if (next.has(groupId)) {
        next.delete(groupId)
      } else {
        next.add(groupId)
      }
      return next
    })
  }
  
  const handleBatchDelete = () => {
    if (selectedGroups.size > 0) {
      onDelete?.(Array.from(selectedGroups))
      setSelectedGroups(new Set())
    }
  }
  
  if (groups.length === 0 && !isLoading) {
    return (
      <div className="p-8">
        <div className="max-w-md mx-auto text-center">
          <h2 className="text-2xl font-bold mb-4">智能去重</h2>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            通过感知哈希 (pHash) 算法检测相似图片，帮助您清理重复内容
          </p>
          
          {/* Threshold Slider */}
          <div className="mb-6">
            <label className="block text-sm font-medium mb-2">
              相似度阈值: {threshold}%
            </label>
            <input
              type="range"
              min="70"
              max="99"
              value={threshold}
              onChange={(e) => setThreshold(Number(e.target.value))}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-gray-500 mt-1">
              <span>70% (宽松)</span>
              <span>99% (严格)</span>
            </div>
          </div>
          
          <button
            onClick={() => onScan?.(threshold)}
            className="btn-primary flex items-center gap-2 mx-auto"
          >
            <Search className="w-5 h-5" />
            开始扫描
          </button>
        </div>
      </div>
    )
  }
  
  return (
    <div className="p-4">
      {/* Header Stats */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold">去重结果</h2>
        <div className="flex items-center gap-4">
          <span className="text-sm text-gray-600 dark:text-gray-400">
            发现 <span className="font-semibold text-red-600">{groups.length}</span> 组重复图片
          </span>
          {selectedGroups.size > 0 && (
            <button
              onClick={handleBatchDelete}
              className="btn-primary bg-red-600 hover:bg-red-700 flex items-center gap-2"
            >
              <Trash2 className="w-4 h-4" />
              删除选中 ({selectedGroups.size})
            </button>
          )}
        </div>
      </div>
      
      <AnimatePresence mode="wait">
        {currentGroup && (
          <motion.div
            key={currentGroup.id}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            className="bg-white dark:bg-dark-100 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
          >
            {/* Group Header */}
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium">
                  第 {currentGroupIndex + 1} / {groups.length} 组
                </span>
                <span className="px-2 py-0.5 bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400 rounded text-xs">
                  相似度: {currentGroup.similarity}%
                </span>
              </div>
              
              <button
                onClick={() => handleToggleSelect(currentGroup.id)}
                className={cn(
                  'px-3 py-1 rounded text-sm border transition-colors',
                  selectedGroups.has(currentGroup.id)
                    ? 'bg-primary-50 border-primary-500 text-primary-600'
                    : 'border-gray-300 hover:border-gray-400'
                )}
              >
                {selectedGroups.has(currentGroup.id) ? '已选中' : '选中此组'}
              </button>
            </div>
            
            {/* Comparison View */}
            <div className="grid grid-cols-2 gap-4 mb-4">
              {currentGroup.images.map((image) => {
                const isKeep = groupDecisions[currentGroup.id] === image.id
                
                return (
                  <div
                    key={image.id}
                    onClick={() => handleKeepSelection(currentGroup.id, image.id)}
                    className={cn(
                      'relative rounded-lg overflow-hidden cursor-pointer transition-all',
                      'border-2',
                      isKeep
                        ? 'border-green-500 ring-2 ring-green-500/30'
                        : 'border-gray-200 dark:border-gray-700 hover:border-primary-400'
                    )}
                  >
                    <img
                      src={image.thumbnail_path}
                      alt={image.file_name}
                      className="w-full aspect-square object-cover"
                    />
                    
                    {/* Keep Badge */}
                    {isKeep && (
                      <div className="absolute top-2 right-2 p-1 bg-green-500 rounded-full">
                        <Check className="w-4 h-4 text-white" />
                      </div>
                    )}
                    
                    {/* Image Info */}
                    <div className="p-2 bg-white dark:bg-dark-100">
                      <p className="text-xs truncate">{image.file_name}</p>
                      <div className="flex gap-2 text-xs text-gray-500 mt-1">
                        {image.width && image.height && (
                          <span>{image.width}x{image.height}</span>
                        )}
                        {image.file_size && (
                          <span>{(image.file_size / 1024).toFixed(1)} KB</span>
                        )}
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
            
            {/* Navigation */}
            <div className="flex items-center justify-between">
              <button
                onClick={handlePrevious}
                disabled={currentGroupIndex === 0}
                className="btn-secondary disabled:opacity-50"
              >
                上一组
              </button>
              
              <div className="flex items-center gap-2">
                <button
                  onClick={() => {
                    if (groupDecisions[currentGroup.id]) {
                      handleNext()
                    }
                  }}
                  disabled={!groupDecisions[currentGroup.id]}
                  className="btn-primary disabled:opacity-50 flex items-center gap-2"
                >
                  保留并跳过
                  <ArrowRight className="w-4 h-4" />
                </button>
              </div>
              
              <button
                onClick={handleNext}
                disabled={currentGroupIndex === groups.length - 1}
                className="btn-secondary disabled:opacity-50"
              >
                下一组
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
      
      {/* Loading State */}
      {isLoading && (
        <div className="flex items-center justify-center h-64">
          <div className="w-8 h-8 border-2 border-gray-300 border-t-primary-500 rounded-full animate-spin" />
          <span className="ml-3">扫描中...</span>
        </div>
      )}
    </div>
  )
}
