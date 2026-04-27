import { useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import {
  scanDuplicates,
  deleteDuplicates,
  type DuplicateGroup,
  type BackendDuplicateGroup,
} from '../lib/api'

export function useDedupActions() {
  const { t } = useTranslation()
  const [dedupGroups, setDedupGroups] = useState<DuplicateGroup[]>([])
  const [dedupLoading, setDedupLoading] = useState(false)

  const onScan = useCallback(async (threshold: number, addToast: (msg: string, type: 'error' | 'success' | 'info') => void) => {
    setDedupLoading(true)
    try {
      const groups = await scanDuplicates(threshold)
      setDedupGroups(groups)
      addToast(t('dedup.scanComplete', { count: groups.length }), 'success')
    } catch {
      addToast(t('errors.scanFailed'), 'error')
    } finally {
      setDedupLoading(false)
    }
  }, [t])

  const onDelete = useCallback(async (
    groupIds: string[],
    addToast: (msg: string, type: 'error' | 'success' | 'info') => void,
    onImagesChanged?: () => void,
  ) => {
    try {
      const groupsToDelete = dedupGroups.filter(g => groupIds.includes(g.id))
      const backendGroups: BackendDuplicateGroup[] = groupsToDelete.map(g => ({
        images: g.images,
        similarity: g.similarity,
      }))
      if (backendGroups.length > 0) {
        await deleteDuplicates(backendGroups, 'keep_highest_resolution')
        onImagesChanged?.()
        setDedupGroups(prev => prev.filter(g => !groupIds.includes(g.id)))
        addToast(t('dedup.deleteSuccess', { count: groupsToDelete.reduce((sum, g) => sum + g.images.length - 1, 0) }), 'success')
      }
    } catch {
      addToast(t('errors.deleteFailed'), 'error')
    }
  }, [dedupGroups, t])

  return {
    dedupGroups,
    dedupLoading,
    onScan,
    onDelete,
  }
}
