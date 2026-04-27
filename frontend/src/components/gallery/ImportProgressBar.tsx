import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { X, CheckCircle, AlertCircle, Loader2 } from 'lucide-react'
import { listen } from '@tauri-apps/api/event'
import { cn } from '@/utils/cn'

interface ImportProgress {
  current_file: string
  current: number
  total: number
  status: 'processing' | 'success' | 'duplicate' | 'error'
}

interface ImportProgressBarProps {
  onComplete?: () => void
}

export function ImportProgressBar({ onComplete }: ImportProgressBarProps) {
  const { t } = useTranslation()
  const [isVisible, setIsVisible] = useState(false)
  const [progress, setProgress] = useState<ImportProgress>({
    current_file: '',
    current: 0,
    total: 0,
    status: 'processing',
  })

  useEffect(() => {
    let unlisten: (() => void) | undefined

    const setupListener = async () => {
      unlisten = await listen<ImportProgress>('import-progress', (event) => {
        setProgress(event.payload)
        setIsVisible(true)

        // Auto-hide when import completes
        if (event.payload.current >= event.payload.total) {
          onComplete?.()
          setTimeout(() => setIsVisible(false), 3000)
        }
      })
    }

    setupListener()

    return () => {
      if (unlisten) unlisten()
    }
  }, [])

  if (!isVisible) return null

  const percentage = progress.total > 0 ? Math.round((progress.current / progress.total) * 100) : 0

  const getStatusIcon = () => {
    switch (progress.status) {
      case 'success':
        return <CheckCircle className="w-4 h-4 text-green-500" />
      case 'error':
        return <AlertCircle className="w-4 h-4 text-red-500" />
      case 'duplicate':
        return <AlertCircle className="w-4 h-4 text-yellow-500" />
      default:
        return <Loader2 className="w-4 h-4 text-blue-500 animate-spin" />
    }
  }

  return (
    <div className="fixed bottom-4 right-4 z-50 w-96 bg-white dark:bg-dark-100 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700">
      <div className="p-4">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            {getStatusIcon()}
            <span className="text-sm font-medium">
              {t('import.importing')} {progress.current}/{progress.total}
            </span>
          </div>
          <button
            onClick={() => setIsVisible(false)}
            className="p-1 hover:bg-gray-100 dark:hover:bg-dark-200 rounded transition-colors"
            aria-label={t('import.close')}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Progress Bar */}
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 mb-2">
          <div
            className={cn(
              'h-2 rounded-full transition-all duration-300',
              progress.status === 'error' ? 'bg-red-500' : 'bg-primary-500'
            )}
            style={{ width: `${percentage}%` }}
          />
        </div>

        {/* Current File */}
        {progress.current_file && (
          <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
            {progress.current_file}
          </p>
        )}

        {/* Percentage */}
        <p className="text-xs text-gray-400 mt-1">{percentage}%</p>
      </div>
    </div>
  )
}
