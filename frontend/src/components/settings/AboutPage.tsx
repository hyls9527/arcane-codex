import { Info, Github, BookOpen, Shield } from 'lucide-react'

interface AboutPageProps {
  onChange?: () => void
}

export function AboutPage({ onChange: _onChange }: AboutPageProps) {
  return (
    <div>
      <div className="flex items-center gap-2 mb-6">
        <Info className="w-5 h-5" />
        <h2 className="text-lg font-semibold">关于</h2>
      </div>
      
      <div className="space-y-6">
        {/* App Info */}
        <div className="text-center py-6">
          <div className="w-20 h-20 mx-auto mb-4 bg-gradient-to-br from-primary-500 to-primary-700 rounded-2xl flex items-center justify-center">
            <span className="text-3xl font-bold text-white">AC</span>
          </div>
          <h1 className="text-2xl font-bold mb-1">Arcane Codex</h1>
          <p className="text-gray-600 dark:text-gray-400">本地图像知识库</p>
          <p className="text-sm text-gray-500 mt-2">版本 0.1.0</p>
        </div>
        
        {/* Links */}
        <div className="grid grid-cols-2 gap-4">
          <a
            href="https://github.com/arcanecodex/app"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-dark-200 rounded-lg hover:bg-gray-100 dark:hover:bg-dark-100 transition-colors"
          >
            <Github className="w-5 h-5" />
            <div>
              <div className="font-medium">GitHub</div>
              <div className="text-xs text-gray-500">查看源代码</div>
            </div>
          </a>
          
          <a
            href="#"
            className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-dark-200 rounded-lg hover:bg-gray-100 dark:hover:bg-dark-100 transition-colors"
          >
            <BookOpen className="w-5 h-5" />
            <div>
              <div className="font-medium">文档</div>
              <div className="text-xs text-gray-500">使用指南</div>
            </div>
          </a>
        </div>
        
        {/* License */}
        <div className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-dark-200 rounded-lg">
          <Shield className="w-5 h-5" />
          <div>
            <div className="font-medium">MIT 许可证</div>
            <div className="text-xs text-gray-500">开源软件，可自由使用和修改</div>
          </div>
        </div>
        
        {/* Tech Stack */}
        <div>
          <h3 className="font-medium mb-2">技术栈</h3>
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div className="flex justify-between p-2 bg-gray-50 dark:bg-dark-200 rounded">
              <span className="text-gray-600">前端</span>
              <span className="font-mono">React 18</span>
            </div>
            <div className="flex justify-between p-2 bg-gray-50 dark:bg-dark-200 rounded">
              <span className="text-gray-600">后端</span>
              <span className="font-mono">Tauri 2.x</span>
            </div>
            <div className="flex justify-between p-2 bg-gray-50 dark:bg-dark-200 rounded">
              <span className="text-gray-600">数据库</span>
              <span className="font-mono">SQLite</span>
            </div>
            <div className="flex justify-between p-2 bg-gray-50 dark:bg-dark-200 rounded">
              <span className="text-gray-600">AI 推理</span>
              <span className="font-mono">LM Studio</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
