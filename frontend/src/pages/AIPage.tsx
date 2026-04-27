import { useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { AIProgressPanel } from '../components/ai/AIProgressPanel'
import { useAIStore } from '../stores/useAIStore'
import {
  getAIStatus,
  startAIProcessing,
  pauseAIProcessing,
  resumeAIProcessing,
  retryFailedAI,
} from '../lib/api'

interface AIPageProps {
  addToast: (message: string, type: 'error' | 'success' | 'info') => void
}

export function AIPage({ addToast }: AIPageProps) {
  const { t } = useTranslation()
  const { status, total, completed, failed, retrying, updateStatus } = useAIStore()
  const [loading, setLoading] = useState(false)

  const loadStatus = useCallback(async () => {
    try {
      const s = await getAIStatus()
      updateStatus(s)
    } catch {}
  }, [updateStatus])

  const handleStart = useCallback(async () => {
    setLoading(true)
    try {
      await startAIProcessing()
      await loadStatus()
      addToast(t('ai.processStarted'), 'success')
    } catch {
      addToast(t('errors.aiStartFailed'), 'error')
    } finally {
      setLoading(false)
    }
  }, [loadStatus, addToast, t])

  const handlePause = useCallback(async () => {
    try {
      await pauseAIProcessing()
      await loadStatus()
      addToast(t('ai.processPaused'), 'info')
    } catch {
      addToast(t('errors.aiPauseFailed'), 'error')
    }
  }, [loadStatus, addToast, t])

  const handleResume = useCallback(async () => {
    try {
      await resumeAIProcessing()
      await loadStatus()
      addToast(t('ai.processResumed'), 'info')
    } catch {
      addToast(t('errors.aiResumeFailed'), 'error')
    }
  }, [loadStatus, addToast, t])

  const handleRetry = useCallback(async () => {
    try {
      await retryFailedAI()
      await loadStatus()
      addToast(t('ai.retryQueued'), 'success')
    } catch {
      addToast(t('errors.aiRetryFailed'), 'error')
    }
  }, [loadStatus, addToast, t])

  return (
    <div className="max-w-2xl mx-auto">
      <AIProgressPanel
        status={{ status, total, completed, failed, retrying }}
        isLoading={loading}
        onStart={handleStart}
        onPause={handlePause}
        onResume={handleResume}
        onRetry={handleRetry}
      />
    </div>
  )
}
