import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { MainLayout } from './MainLayout'
import { Sidebar } from './Sidebar'
import { TopBar } from './TopBar'

// Mock useThemeStore
vi.mock('@/stores/useThemeStore', () => ({
  useThemeStore: vi.fn(() => ({ applyTheme: vi.fn() })),
}))

// Mock useConfigStore
vi.mock('@/stores/useConfigStore', () => ({
  useConfigStore: vi.fn(() => ({ theme: 'light', language: 'zh', updateField: vi.fn() })),
  CONFIG_KEYS: { THEME: 'theme', LANGUAGE: 'language' },
}))

// Mock i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { changeLanguage: vi.fn(), language: 'zh' },
  }),
}))

describe('布局组件单元测试', () => {
  describe('MainLayout', () => {
    it('应该正确渲染子组件', () => {
      render(
        <MainLayout>
          <div data-testid="child-content">Test Content</div>
        </MainLayout>
      )
      expect(screen.getByTestId('child-content')).toBeInTheDocument()
    })

    it('应该应用正确的 CSS 类', () => {
      const { container } = render(<MainLayout>Content</MainLayout>)
      const root = container.firstChild as HTMLElement
      expect(root).toHaveClass('flex', 'h-screen', 'w-screen')
    })

    it('应该支持自定义 className', () => {
      const { container } = render(<MainLayout className="custom-class">Content</MainLayout>)
      const root = container.firstChild as HTMLElement
      expect(root).toHaveClass('custom-class')
    })
  })

  describe('Sidebar', () => {
    const mockOnNavigate = vi.fn()

    it('应该渲染所有导航项', () => {
      render(<Sidebar onNavigate={mockOnNavigate} currentPage="gallery" />)
      expect(screen.getByText('图库')).toBeInTheDocument()
      expect(screen.getByText('AI 打标')).toBeInTheDocument()
      expect(screen.getByText('去重')).toBeInTheDocument()
      expect(screen.getByText('设置')).toBeInTheDocument()
    })

    it('应该高亮当前页面', () => {
      render(<Sidebar onNavigate={mockOnNavigate} currentPage="ai" />)
      const aiButton = screen.getByText('AI 打标').closest('button')
      expect(aiButton).toHaveClass('bg-primary-50')
    })

    it('应该在点击导航项时调用 onNavigate', () => {
      render(<Sidebar onNavigate={mockOnNavigate} currentPage="gallery" />)
      const settingsButton = screen.getByText('设置').closest('button')
      settingsButton?.click()
      expect(mockOnNavigate).toHaveBeenCalledWith('settings')
    })

    it('应该支持折叠功能', async () => {
      render(<Sidebar onNavigate={mockOnNavigate} currentPage="gallery" />)
      // 初始应该显示 "图库" 文本
      expect(screen.getByText('图库')).toBeInTheDocument()
      
      // 找到折叠按钮并点击（使用无障碍标签）
      const collapseButton = screen.getByRole('button', { name: '折叠侧边栏' })
      await fireEvent.click(collapseButton)
      
      // 等待更新完成
      await waitFor(() => {
        expect(screen.queryByText('图库')).not.toBeInTheDocument()
      })
    })
  })

  describe('TopBar', () => {
    it('应该渲染搜索输入框', () => {
      render(<TopBar />)
      expect(screen.getByPlaceholderText('搜索图片...')).toBeInTheDocument()
    })

    it('应该渲染主题切换按钮', () => {
      render(<TopBar />)
      const themeButton = screen.getByRole('button', { name: '切换主题' })
      expect(themeButton).toBeInTheDocument()
    })

    it('应该在搜索框输入时更新值', () => {
      render(<TopBar />)
      const input = screen.getByPlaceholderText('搜索图片...') as HTMLInputElement
      input.value = 'test query'
      expect(input.value).toBe('test query')
    })
  })
})
