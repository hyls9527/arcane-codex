import { useState } from 'react'
import { 
  Image as ImageIcon, 
  Settings, 
  Sparkles, 
  Copy,
  ChevronLeft,
  ChevronRight
} from 'lucide-react'
import { cn } from '@/utils/cn'

type Page = 'gallery' | 'settings' | 'ai' | 'dedup'

interface SidebarProps {
  onNavigate: (page: Page) => void
  currentPage: Page
}

const navItems = [
  { id: 'gallery' as const, label: '图库', icon: ImageIcon },
  { id: 'ai' as const, label: 'AI 打标', icon: Sparkles },
  { id: 'dedup' as const, label: '去重', icon: Copy },
  { id: 'settings' as const, label: '设置', icon: Settings },
]

export function Sidebar({ onNavigate, currentPage }: SidebarProps) {
  const [collapsed, setCollapsed] = useState(false)
  
  return (
    <aside className={cn(
      'flex flex-col bg-white dark:bg-dark-100 border-r border-gray-200 dark:border-gray-700 transition-all duration-300',
      collapsed ? 'w-16' : 'w-64'
    )}>
      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        {!collapsed && (
          <h1 className="text-lg font-semibold text-gray-900 dark:text-white">
            Arcane Codex
          </h1>
        )}
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
        >
          {collapsed ? (
            <ChevronRight className="w-5 h-5" />
          ) : (
            <ChevronLeft className="w-5 h-5" />
          )}
        </button>
      </div>
      
      <nav className="flex-1 p-2">
        {navItems.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => onNavigate(id)}
            className={cn(
              'flex items-center w-full gap-3 px-3 py-2 rounded-lg transition-colors',
              'hover:bg-gray-100 dark:hover:bg-gray-700',
              currentPage === id && 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400',
              collapsed && 'justify-center'
            )}
          >
            <Icon className="w-5 h-5 flex-shrink-0" />
            {!collapsed && <span>{label}</span>}
          </button>
        ))}
      </nav>
    </aside>
  )
}
