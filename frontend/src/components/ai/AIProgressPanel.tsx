import { useState } from 'react'
import { Play, Pause, RotateCcw, AlertCircle } from 'lucide-react'
import { cn } from '@/utils/cn'
import { motion } from 'framer-motion'

interface AIStatus {
  status: 'idle' | 'processing' | 'paused' | 'completed' | 'failed'
  total: number
  completed: number
  failed: number
  retrying: number
  eta_seconds?: number
}

interface AIProgressPanelProps {
  status?: AIStatus
  isLoading?: boolean
  onStart?: () => void
  onPause?: () => void
  onResume?: () => void
  onCancel?: () => void
  onRetry?: () => void
}

export function AIProgressPanel({
  status = {
    status: 'idle',
    total: 0,
    completed: 0,
    failed: 0,
    retrying: 0,
  },
  isLoading = false,
  onStart,
  onPause,
  onResume,
  onCancel,
  onRetry,
}: AIProgressPanelProps) {
  const [showCancelConfirm, setShowCancelConfirm] = useState(false)
  
  const progress = status.total > 0 
    ? Math.round((status.completed / status.total) * 100) 
    : 0
  
  const formatTime = (seconds?: number) => {
    if (!seconds) return '计算中...'
    const mins = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${mins}分${secs}秒`
  }
  
  const statusText = {
    idle: '空闲',
    processing: '处理中...',
    paused: '已暂停',
    completed: '已完成',
    failed: '失败',
  }
  
  return (
    <div className="p-4 bg-white dark:bg-dark-100 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-lg font-semibold">AI 打标进度</h3>
        <span className={cn(
          'px-2 py-1 rounded text-xs',
          status.status === 'processing' && 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400',
          status.status === 'paused' && 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400',
          status.status === 'completed' && 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
          status.status === 'failed' && 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400',
        )}>
          {statusText[status.status]}
        </span>
      </div>
      
      {/* Progress Bar */}
      <div className="mb-4">
        <div className="h-2 bg-gray-200 dark:bg-dark-200 rounded-full overflow-hidden">
          <motion.div
            className={cn(
              'h-full transition-all duration-500',
              status.status === 'failed' ? 'bg-red-500' : 'bg-primary-500'
            )}
            initial={{ width: 0 }}
            animate={{ width: `${progress}%` }}
          />
        </div>
        <div className="flex justify-between mt-1 text-xs text-gray-500">
          <span>{progress}%</span>
          <span>{status.completed} / {status.total}</span>
        </div>
      </div>
      
      {/* Stats Grid */}
      <div className="grid grid-cols-3 gap-3 mb-4">
        <div className="p-2 bg-green-50 dark:bg-green-900/20 rounded">
          <div className="text-2xl font-bold text-green-600">{status.completed}</div>
          <div className="text-xs text-gray-600">成功</div>
        </div>
        
        <div className="p-2 bg-red-50 dark:bg-red-900/20 rounded">
          <div className="text-2xl font-bold text-red-600">{status.failed}</div>
          <div className="text-xs text-gray-600">失败</div>
        </div>
        
        <div className="p-2 bg-yellow-50 dark:bg-yellow-900/20 rounded">
          <div className="text-2xl font-bold text-yellow-600">{status.retrying}</div>
          <div className="text-xs text-gray-600">重试中</div>
        </div>
      </div>
      
      {/* ETA */}
      {status.status === 'processing' && (
        <div className="mb-4 text-sm text-gray-600 dark:text-gray-400">
          预计剩余时间: {formatTime(status.eta_seconds)}
        </div>
      )}
      
      {/* Controls */}
      <div className="flex gap-2">
        {status.status === 'idle' && (
          <button
            onClick={onStart}
            className="flex-1 btn-primary flex items-center justify-center gap-2"
          >
            <Play className="w-4 h-4" />
            开始处理
          </button>
        )}
        
        {status.status === 'processing' && (
          <button
            onClick={onPause}
            className="flex-1 btn-secondary flex items-center justify-center gap-2"
          >
            <Pause className="w-4 h-4" />
            暂停
          </button>
        )}
        
        {status.status === 'paused' && (
          <button
            onClick={onResume}
            className="flex-1 btn-primary flex items-center justify-center gap-2"
          >
            <RotateCcw className="w-4 h-4" />
            继续
          </button>
        )}
        
        {status.failed > 0 && (
          <button
            onClick={onRetry}
            className="btn-secondary flex items-center gap-2"
          >
            <AlertCircle className="w-4 h-4" />
            重试失败项
          </button>
        )}
        
        {status.status !== 'idle' && (
          <button
            onClick={() => setShowCancelConfirm(true)}
            className="btn-secondary text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
          >
            取消
          </button>
        )}
      </div>
      
      {/* Cancel Confirmation */}
      {showCancelConfirm && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="mt-4 p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded border border-yellow-200 dark:border-yellow-700"
        >
          <p className="text-sm text-yellow-800 dark:text-yellow-200 mb-2">
            确定要取消所有待处理任务吗？
          </p>
          <div className="flex gap-2">
            <button
              onClick={() => {
                onCancel?.()
                setShowCancelConfirm(false)
              }}
              className="btn-primary bg-red-600 hover:bg-red-700"
            >
              确认取消
            </button>
            <button
              onClick={() => setShowCancelConfirm(false)}
              className="btn-secondary"
            >
              返回
            </button>
          </div>
        </motion.div>
      )}
    </div>
  )
}
