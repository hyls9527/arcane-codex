import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Brain, CheckCircle, AlertCircle, Loader2 } from 'lucide-react'
import { cn } from '@/utils/cn'
import { useConfigStore, CONFIG_KEYS } from '@/stores/useConfigStore'
import { testLmStudioConnection } from '@/lib/api'

interface AIConfigProps {
  onChange?: () => void
}

/**
 * 验证 URL 格式是否有效
 * 接受 http:// 或 https:// 开头的 URL
 */
function isValidUrl(url: string): boolean {
  if (!url || url.trim().length === 0) return true // 空值允许（使用默认值）
  
  try {
    const parsed = new URL(url)
    return parsed.protocol === 'http:' || parsed.protocol === 'https:'
  } catch {
    return false
  }
}

export function AIConfig({ onChange }: AIConfigProps) {
  const { t } = useTranslation()
  const { lmStudioUrl, aiConcurrency, aiTimeout, updateField } = useConfigStore()
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null)
  const [urlValidationError, setUrlValidationError] = useState<string | null>(null)

  const handleUrlChange = (value: string) => {
    updateField(CONFIG_KEYS.LM_STUDIO_URL, value)
    onChange?.()

    // 实时验证 URL 格式
    if (value.trim().length > 0 && !isValidUrl(value)) {
      setUrlValidationError(t('settings.ai.urlInvalid'))
    } else {
      setUrlValidationError(null)
    }
  }

  const handleTestConnection = async () => {
    // 测试前先验证 URL 格式
    if (!isValidUrl(lmStudioUrl)) {
      setTestResult('error')
      setUrlValidationError(t('settings.ai.urlInvalidTest'))
      return
    }

    setTesting(true)
    setTestResult(null)
    setUrlValidationError(null)

    try {
      const result = await testLmStudioConnection(lmStudioUrl)
      setTestResult(result ? 'success' : 'error')
    } catch {
      setTestResult('error')
    } finally {
      setTesting(false)
    }
  }

  return (
    <div>
      <div className="flex items-center gap-2 mb-6">
        <Brain className="w-5 h-5" />
        <h2 className="text-lg font-semibold">{t('settings.ai.title')}</h2>
      </div>

      <div className="space-y-6">
        {/* LM Studio URL */}
        <div>
          <label className="block text-sm font-medium mb-2">
            {t('settings.ai.lmStudioUrl')}
          </label>
          <div className="flex gap-2">
            <input
              type="url"
              value={lmStudioUrl}
              onChange={(e) => {
                handleUrlChange(e.target.value)
              }}
              placeholder={t('settings.ai.lmStudioUrlPlaceholder')}
              className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                         bg-white dark:bg-dark-200 focus:ring-2 focus:ring-primary-500 outline-none"
            />
            <button
              onClick={handleTestConnection}
              disabled={testing}
              className="btn-secondary flex items-center gap-2 disabled:opacity-50"
            >
              {testing ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : testResult === 'success' ? (
                <CheckCircle className="w-4 h-4 text-green-500" />
              ) : testResult === 'error' ? (
                <AlertCircle className="w-4 h-4 text-red-500" />
              ) : null}
              {t('settings.ai.testConnection')}
            </button>
          </div>

          {/* Test Result */}
          {testResult && (
            <p className={cn(
              'mt-2 text-sm',
              testResult === 'success' ? 'text-green-600' : 'text-red-600'
            )}>
              {testResult === 'success' ? t('settings.ai.connectionSuccess') : t('settings.ai.connectionFailed')}
            </p>
          )}

          {/* URL Validation Error */}
          {urlValidationError && (
            <p className="mt-2 text-sm text-red-600">{urlValidationError}</p>
          )}
        </div>

        {/* Concurrency */}
        <div>
          <label className="block text-sm font-medium mb-2">
            {t('settings.ai.concurrency')}: {aiConcurrency}
          </label>
          <input
            type="range"
            min="1"
            max="10"
            value={aiConcurrency}
            onChange={(e) => {
              updateField(CONFIG_KEYS.AI_CONCURRENCY, e.target.value)
              onChange?.()
            }}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-gray-500 mt-1">
            <span>1 ({t('settings.ai.concurrencySlow')})</span>
            <span>10 ({t('settings.ai.concurrencyFast')})</span>
          </div>
        </div>

        {/* Timeout */}
        <div>
          <label className="block text-sm font-medium mb-2">
            {t('settings.ai.timeout')}
          </label>
          <input
            type="number"
            min="10"
            max="120"
            value={aiTimeout}
            onChange={(e) => {
              updateField(CONFIG_KEYS.AI_TIMEOUT, e.target.value)
              onChange?.()
            }}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                       bg-white dark:bg-dark-200 focus:ring-2 focus:ring-primary-500 outline-none"
          />
        </div>
      </div>
    </div>
  )
}
