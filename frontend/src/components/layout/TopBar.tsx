import { Search, Sun, Moon, Monitor } from 'lucide-react'
import { useState } from 'react'
import { useThemeStore } from '@/stores/useThemeStore'

export function TopBar() {
  const [searchQuery, setSearchQuery] = useState('')
  const { theme, setTheme } = useThemeStore()
  
  const cycleTheme = () => {
    const themes = ['light', 'dark', 'system'] as const
    const currentIndex = themes.indexOf(theme)
    setTheme(themes[(currentIndex + 1) % themes.length])
  }
  
  const getThemeIcon = () => {
    switch (theme) {
      case 'light': return <Sun className="w-5 h-5" />
      case 'dark': return <Moon className="w-5 h-5" />
      case 'system': return <Monitor className="w-5 h-5" />
    }
  }
  
  return (
    <header className="flex items-center gap-4 p-3 bg-white dark:bg-dark-100 border-b border-gray-200 dark:border-gray-700">
      <div className="flex-1 relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          type="text"
          placeholder="搜索图片..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full pl-10 pr-4 py-2 rounded-lg bg-gray-100 dark:bg-dark-200 
                     border-none focus:ring-2 focus:ring-primary-500 outline-none"
        />
      </div>
      
      <button
        onClick={cycleTheme}
        className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
        aria-label="切换主题"
      >
        {getThemeIcon()}
      </button>
    </header>
  )
}
