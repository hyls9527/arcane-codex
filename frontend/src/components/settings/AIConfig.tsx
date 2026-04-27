import { useState } from 'react'
import { Brain, CheckCircle, AlertCircle, Loader2 } from 'lucide-react'
import { cn } from '@/utils/cn'
import { useConfigStore, CONFIG_KEYS } from '@/stores/useConfigStore'
import { testLmStudioConnection } from '@/lib/api'

interface AIConfigProps {
  onChange?: () => void
}

export function AIConfig({ onChange }: AIConfigProps) {
  const { lmStudioUrl, aiConcurrency, aiTimeout, updateField } = useConfigStore()
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null)

  const handleTestConnection = async () => {
    setTesting(true)
    setTestResult(null)

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
        <h2 className="text-lg font-semibold">AI 配置</h2>
      </div>

      <div className="space-y-6">
        {/* LM Studio URL */}
        <div>
          <label className="block text-sm font-medium mb-2">
            LM Studio 地址
          </label>
          <div className="flex gap-2">
            <input
              type="url"
              value={lmStudioUrl}
              onChange={(e) => {
                updateField(CONFIG_KEYS.LM_STUDIO_URL, e.target.value)
                onChange?.()
              }}
              placeholder="http://localhost:1234"
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
              测试连接
            </button>
          </div>

          {/* Test Result */}
          {testResult && (
            <p className={cn(
              'mt-2 text-sm',
              testResult === 'success' ? 'text-green-600' : 'text-red-600'
            )}>
              {testResult === 'success' ? '连接成功！' : '连接失败，请检查地址和 LM Studio 状态'}
            </p>
          )}
        </div>

        {/* Concurrency */}
        <div>
          <label className="block text-sm font-medium mb-2">
            并发数: {aiConcurrency}
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
            <span>1 (慢但稳定)</span>
            <span>10 (快速但耗资源)</span>
          </div>
        </div>

        {/* Timeout */}
        <div>
          <label className="block text-sm font-medium mb-2">
            超时时间 (秒)
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
