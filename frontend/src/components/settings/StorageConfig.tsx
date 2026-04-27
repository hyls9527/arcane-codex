import { useState } from 'react'
import { Database, FolderOpen, Download, Upload, Loader2, CheckCircle, AlertCircle } from 'lucide-react'
import { backupDatabase, restoreDatabase } from '@/lib/api'

interface StorageConfigProps {
  onChange?: () => void
}

export function StorageConfig({ onChange: _onChange }: StorageConfigProps) {
  const appDataPath = '%APPDATA%\\ArcaneCodex'
  
  const [backingUp, setBackingUp] = useState(false)
  const [backupResult, setBackupResult] = useState<'success' | 'error' | null>(null)
  const [restoring, setRestoring] = useState(false)
  const [restoreResult, setRestoreResult] = useState<'success' | 'error' | null>(null)

  const handleBackup = async () => {
    setBackingUp(true)
    setBackupResult(null)
    try {
      const fallbackPath = `${process.env.APPDATA || 'C:/Users/Public'}/ArcaneCodex/backup_${Date.now()}.zip`
      const resultPath = await backupDatabase(fallbackPath)
      setBackupResult('success')
      console.log('Backup saved to:', resultPath)
    } catch {
      setBackupResult('error')
    } finally {
      setBackingUp(false)
    }
  }

  const handleRestore = async () => {
    setRestoring(true)
    setRestoreResult(null)
    try {
      await restoreDatabase('')
      setRestoreResult('success')
    } catch {
      setRestoreResult('error')
    } finally {
      setRestoring(false)
    }
  }
  
  return (
    <div>
      <div className="flex items-center gap-2 mb-6">
        <Database className="w-5 h-5" />
        <h2 className="text-lg font-semibold">存储配置</h2>
      </div>
      
      <div className="space-y-6">
        {/* Data Directory */}
        <div>
          <label className="block text-sm font-medium mb-2">应用数据目录</label>
          <div className="flex gap-2">
            <input
              type="text"
              value={appDataPath}
              readOnly
              className="flex-1 px-3 py-2 bg-gray-100 dark:bg-dark-200 rounded-lg border border-gray-300 dark:border-gray-600"
            />
            <button className="btn-secondary flex items-center gap-2">
              <FolderOpen className="w-4 h-4" />
              打开目录
            </button>
          </div>
        </div>
        
        {/* Backup & Restore */}
        <div className="grid grid-cols-2 gap-4">
          <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
            <Download className="w-6 h-6 text-green-600 mb-2" />
            <h3 className="font-medium mb-1">备份数据库</h3>
            <p className="text-xs text-gray-600 dark:text-gray-400 mb-3">
              将数据库导出为 zip 文件，便于备份和迁移
            </p>
            <button 
              onClick={handleBackup}
              disabled={backingUp}
              className="btn-primary bg-green-600 hover:bg-green-700 w-full flex items-center justify-center gap-2 disabled:opacity-50"
            >
              {backingUp ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  备份中...
                </>
              ) : backupResult === 'success' ? (
                <>
                  <CheckCircle className="w-4 h-4" />
                  备份完成
                </>
              ) : (
                '导出备份'
              )}
            </button>
            
            {backupResult === 'error' && (
              <p className="mt-2 text-xs text-red-600 flex items-center gap-1">
                <AlertCircle className="w-3 h-3" />
                备份失败，请重试
              </p>
            )}
            {backupResult === 'success' && (
              <p className="mt-2 text-xs text-green-700 dark:text-green-400 flex items-center gap-1">
                <CheckCircle className="w-3 h-3" />
                备份成功
              </p>
            )}
          </div>
          
          <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
            <Upload className="w-6 h-6 text-blue-600 mb-2" />
            <h3 className="font-medium mb-1">恢复数据库</h3>
            <p className="text-xs text-gray-600 dark:text-gray-400 mb-3">
              从备份文件恢复数据库，请先备份当前数据
            </p>
            <button 
              onClick={handleRestore}
              disabled={restoring}
              className="btn-primary bg-blue-600 hover:bg-blue-700 w-full flex items-center justify-center gap-2 disabled:opacity-50"
            >
              {restoring ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  恢复中...
                </>
              ) : restoreResult === 'success' ? (
                <>
                  <CheckCircle className="w-4 h-4" />
                  恢复完成
                </>
              ) : (
                '导入备份'
              )}
            </button>
            
            {restoreResult === 'error' && (
              <p className="mt-2 text-xs text-red-600 flex items-center gap-1">
                <AlertCircle className="w-3 h-3" />
                恢复失败，请重试
              </p>
            )}
            {restoreResult === 'success' && (
              <p className="mt-2 text-xs text-blue-700 dark:text-blue-400">
                数据库已成功恢复
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
