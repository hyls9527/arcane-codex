import { Palette } from 'lucide-react'
import { useThemeStore } from '@/stores/useThemeStore'
import { useConfigStore, CONFIG_KEYS } from '@/stores/useConfigStore'

interface DisplayConfigProps {
  onChange?: () => void
}

export function DisplayConfig({ onChange }: DisplayConfigProps) {
  const { theme: persistedTheme, setTheme } = useThemeStore()
  const { thumbnailSize, updateField } = useConfigStore()

  return (
    <div>
      <div className="flex items-center gap-2 mb-6">
        <Palette className="w-5 h-5" />
        <h2 className="text-lg font-semibold">显示设置</h2>
      </div>

      <div className="space-y-6">
        {/* Theme Selector */}
        <div>
          <label className="block text-sm font-medium mb-2">主题</label>
          <div className="grid grid-cols-3 gap-3">
            {[
              { id: 'light' as const, label: '浅色', preview: 'bg-white border-gray-300' },
              { id: 'dark' as const, label: '深色', preview: 'bg-gray-900 border-gray-700' },
              { id: 'system' as const, label: '跟随系统', preview: 'bg-gradient-to-br from-white to-gray-900 border-gray-400' },
            ].map(({ id, label, preview }) => (
              <button
                key={id}
                onClick={() => {
                  setTheme(id)
                  onChange?.()
                }}
                className={`p-3 rounded-lg border-2 transition-all ${
                  persistedTheme === id ? 'border-primary-500 ring-2 ring-primary-500/30' : 'border-gray-200 dark:border-gray-700'
                }`}
              >
                <div className={`h-12 rounded ${preview} border mb-2`} />
                <span className="text-sm">{label}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Thumbnail Size */}
        <div>
          <label className="block text-sm font-medium mb-2">缩略图尺寸</label>
          <select
            value={thumbnailSize}
            onChange={(e) => {
              updateField(CONFIG_KEYS.THUMBNAIL_SIZE, e.target.value)
              onChange?.()
            }}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                       bg-white dark:bg-dark-200 focus:ring-2 focus:ring-primary-500 outline-none"
          >
            <option value="200">小 (200px)</option>
            <option value="300">中 (300px)</option>
            <option value="400">大 (400px)</option>
          </select>
        </div>
      </div>
    </div>
  )
}
