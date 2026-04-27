import { Search, Sun, Moon, Monitor, Languages } from 'lucide-react'
import { useState } from 'react'
import { useThemeStore } from '@/stores/useThemeStore'
import { useTranslation } from 'react-i18next'
import { cn } from '@/utils/cn'

export function TopBar() {
  const [searchQuery, setSearchQuery] = useState('')
  const [showLangMenu, setShowLangMenu] = useState(false)
  const { theme, setTheme } = useThemeStore()
  const { i18n } = useTranslation()
  const currentLang = i18n.language
  
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
  
  const switchLanguage = (lng: string) => {
    i18n.changeLanguage(lng)
    setShowLangMenu(false)
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
      
      {/* Language Switcher */}
      <div className="relative">
        <button
          onClick={() => setShowLangMenu(!showLangMenu)}
          className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          aria-label="切换语言"
        >
          <Languages className="w-5 h-5" />
        </button>
        
        {showLangMenu && (
          <div className="absolute right-0 top-full mt-2 bg-white dark:bg-dark-100 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-50 min-w-[120px]">
            <button
              onClick={() => switchLanguage('zh')}
              className={cn(
                'w-full px-4 py-2 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors',
                currentLang === 'zh' && 'text-primary-600 dark:text-primary-400 font-medium'
              )}
            >
              中文
            </button>
            <button
              onClick={() => switchLanguage('en')}
              className={cn(
                'w-full px-4 py-2 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors',
                currentLang === 'en' && 'text-primary-600 dark:text-primary-400 font-medium'
              )}
            >
              English
            </button>
          </div>
        )}
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
