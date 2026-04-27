import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { CheckCircle, Loader2 } from 'lucide-react'

interface ImportProgressPayload {
  current: number
  total: number
  currentFile?: string
  status: 'processing' | 'completed'
}

interface ImportProgressProps {
  onComplete?: () => void
}

export function ImportProgressBar({ onComplete }: ImportProgressProps) {
  const [progress, setProgress] = useState<ImportProgressPayload | null>(null)
  const [visible, setVisible] = useState(false)

  useEffect(() => {
    const unlisten = listen<ImportProgressPayload>('import-progress', (event) => {
      setProgress(event.payload)
      setVisible(true)

      if (event.payload.status === 'completed') {
        setTimeout(() => {
          setVisible(false)
          setProgress(null)
          onComplete?.()
        }, 1500)
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [onComplete])

  if (!visible || !progress) return null

  const percent = progress.total > 0
    ? Math.round((progress.current / progress.total) * 100)
    : 0
  const isCompleted = progress.status === 'completed'

  return (
    <div className="fixed bottom-4 right-4 z-50 w-80 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg p-4 transition-all duration-300">
      <div className="flex items-center gap-2 mb-2">
        {isCompleted ? (
          <CheckCircle className="w-4 h-4 text-green-500 shrink-0" />
        ) : (
          <Loader2 className="w-4 h-4 text-primary-500 animate-spin shrink-0" />
        )}
        <span className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
          {isCompleted ? '导入完成' : `导入中 ${progress.current}/${progress.total}`}
        </span>
      </div>

      {!isCompleted && progress.currentFile && (
        <p className="text-xs text-gray-500 dark:text-gray-400 mb-2 truncate">
          {progress.currentFile}
        </p>
      )}

      <div className="w-full h-2 bg-gray-100 dark:bg-gray-700 rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-300 ${
            isCompleted
              ? 'bg-green-500'
              : 'bg-primary-500'
          }`}
          style={{ width: `${percent}%` }}
        />
      </div>

      <p className="text-xs text-gray-400 mt-1 text-right">
        {percent}%
      </p>
    </div>
  )
}
