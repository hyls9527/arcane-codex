import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { DedupManager } from '../components/dedup/DedupManager'
import { useDedupStore } from '../stores/useDedupStore'
import {
  scanDuplicates,
  deleteDuplicates,
  type BackendDuplicateGroup,
} from '../lib/api'

interface DedupPageProps {
  addToast: (message: string, type: 'error' | 'success' | 'info') => void
  onImagesChanged: () => void
}

export function DedupPage({ addToast, onImagesChanged }: DedupPageProps) {
  const { t } = useTranslation()
  const { groups, loading, setGroups, setLoading, removeGroups } = useDedupStore()

  const handleScan = useCallback(async (threshold: number) => {
    setLoading(true)
    try {
      const result = await scanDuplicates(threshold)
      setGroups(result)
      addToast(t('dedup.scanComplete', { count: result.length }), 'success')
    } catch {
      addToast(t('errors.scanFailed'), 'error')
    } finally {
      setLoading(false)
    }
  }, [setGroups, setLoading, addToast, t])

  const handleDelete = useCallback(async (groupIds: string[]) => {
    try {
      const groupsToDelete = groups.filter(g => groupIds.includes(g.id))
      const backendGroups: BackendDuplicateGroup[] = groupsToDelete.map(g => ({
        images: g.images,
        similarity: g.similarity,
      }))
      if (backendGroups.length > 0) {
        await deleteDuplicates(backendGroups, 'keep_highest_resolution')
        onImagesChanged()
        removeGroups(groupIds)
        addToast(t('dedup.deleteSuccess', { count: groupsToDelete.reduce((sum, g) => sum + g.images.length - 1, 0) }), 'success')
      }
    } catch {
      addToast(t('errors.deleteFailed'), 'error')
    }
  }, [groups, removeGroups, onImagesChanged, addToast, t])

  return (
    <DedupManager
      groups={groups}
      isLoading={loading}
      onScan={handleScan}
      onDelete={handleDelete}
    />
  )
}
