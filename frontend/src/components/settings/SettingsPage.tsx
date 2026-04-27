import { useEffect, useState } from 'react'
import { Settings as SettingsIcon, Brain, Palette, Database, Info } from 'lucide-react'
import { cn } from '@/utils/cn'
import { AIConfig } from './AIConfig'
import { DisplayConfig } from './DisplayConfig'
import { StorageConfig } from './StorageConfig'
import { AboutPage } from './AboutPage'
import { useConfigStore } from '@/stores/useConfigStore'
import { useThemeStore } from '@/stores/useThemeStore'

type SettingsTab = 'ai' | 'display' | 'storage' | 'about'

const tabs = [
  { id: 'ai' as const, label: 'AI 配置', icon: Brain },
  { id: 'display' as const, label: '显示设置', icon: Palette },
  { id: 'storage' as const, label: '存储配置', icon: Database },
  { id: 'about' as const, label: '关于', icon: Info },
]

export function SettingsPage() {
  const [activeTab, setActiveTab] = useState<SettingsTab>('ai')
  const [isSaving, setIsSaving] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)
  
  const { hasPendingChanges, loadConfigs, saveConfigs } = useConfigStore()
  const { setTheme } = useThemeStore()

  // Load persisted configs when component mounts
  useEffect(() => {
    loadConfigs().then((configs) => {
      // Sync theme from config store to theme store
      if (configs?.theme) {
        setTheme(configs.theme as 'light' | 'dark' | 'system')
      }
    })
  }, [loadConfigs, setTheme])

  const handleSave = async () => {
    setIsSaving(true)
    setSaveError(null)
    try {
      await saveConfigs()
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : '保存失败')
    } finally {
      setIsSaving(false)
    }
  }

  const ActiveComponent = {
    ai: AIConfig,
    display: DisplayConfig,
    storage: StorageConfig,
    about: AboutPage,
  }[activeTab]

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <div className="flex items-center gap-3 mb-6">
        <SettingsIcon className="w-6 h-6" />
        <h1 className="text-2xl font-bold">设置</h1>
      </div>

      <div className="flex gap-6">
        {/* Sidebar Tabs */}
        <nav className="w-48 flex-shrink-0">
          <ul className="space-y-1">
            {tabs.map(({ id, label, icon: Icon }) => (
              <li key={id}>
                <button
                  onClick={() => setActiveTab(id)}
                  className={cn(
                    'flex items-center gap-3 w-full px-3 py-2 rounded-lg text-left transition-colors',
                    'hover:bg-gray-100 dark:hover:bg-gray-800',
                    activeTab === id && 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400'
                  )}
                >
                  <Icon className="w-4 h-4 flex-shrink-0" />
                  <span className="text-sm">{label}</span>
                </button>
              </li>
            ))}
          </ul>
        </nav>

        {/* Content Area */}
        <div className="flex-1">
          <div className="bg-white dark:bg-dark-100 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
            <ActiveComponent onChange={() => {}} />
          </div>

          {/* Save Button */}
          {hasPendingChanges() && activeTab !== 'about' && (
            <div className="mt-4 flex items-center gap-4">
              {saveError && (
                <p className="text-sm text-red-600">{saveError}</p>
              )}
              <button
                onClick={handleSave}
                disabled={isSaving}
                className="btn-primary ml-auto disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isSaving ? '保存中...' : '保存更改'}
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
