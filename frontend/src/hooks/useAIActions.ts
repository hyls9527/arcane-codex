import { useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import {
  getAIStatus,
  startAIProcessing,
  pauseAIProcessing,
  resumeAIProcessing,
  retryFailedAI,
  type AIStatus,
} from '../lib/api'

export function useAIActions() {
  const { t } = useTranslation()
  const [aiStatus, setAiStatus] = useState<AIStatus>({
    status: 'idle',
    total: 0,
    completed: 0,
    failed: 0,
    retrying: 0,
  })
  const [aiLoading, setAiLoading] = useState(false)

  // const addToastRef = useState<(message: string, type: 'error' | 'success' | 'info') => void>(null as any)

  const loadAIStatus = useCallback(async () => {
    try {
      const status = await getAIStatus()
      setAiStatus(status)
    } catch {
    }
  }, [])

  const onStart = useCallback(async (addToast: (msg: string, type: 'error' | 'success' | 'info') => void) => {
    setAiLoading(true)
    try {
      await startAIProcessing()
      await loadAIStatus()
      addToast(t('ai.processStarted'), 'success')
    } catch {
      addToast(t('errors.aiStartFailed'), 'error')
    } finally {
      setAiLoading(false)
    }
  }, [loadAIStatus, t])

  const onPause = useCallback(async (addToast: (msg: string, type: 'error' | 'success' | 'info') => void) => {
    try {
      await pauseAIProcessing()
      await loadAIStatus()
      addToast(t('ai.processPaused'), 'info')
    } catch {
      addToast(t('errors.aiPauseFailed'), 'error')
    }
  }, [loadAIStatus, t])

  const onResume = useCallback(async (addToast: (msg: string, type: 'error' | 'success' | 'info') => void) => {
    try {
      await resumeAIProcessing()
      await loadAIStatus()
      addToast(t('ai.processResumed'), 'info')
    } catch {
      addToast(t('errors.aiResumeFailed'), 'error')
    }
  }, [loadAIStatus, t])

  const onRetry = useCallback(async (addToast: (msg: string, type: 'error' | 'success' | 'info') => void) => {
    try {
      await retryFailedAI()
      await loadAIStatus()
      addToast(t('ai.retryQueued'), 'success')
    } catch {
      addToast(t('errors.aiRetryFailed'), 'error')
    }
  }, [loadAIStatus, t])

  return {
    aiStatus,
    aiLoading,
    loadAIStatus,
    onStart,
    onPause,
    onResume,
    onRetry,
  }
}
