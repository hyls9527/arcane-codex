import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
      // 测试时强制使用 React 开发版本
      'react$': resolve(__dirname, './node_modules/react/cjs/react.development.js'),
      'react-dom$': resolve(__dirname, './node_modules/react-dom/cjs/react-dom.development.js'),
      'react/jsx-runtime$': resolve(__dirname, './node_modules/react/cjs/react-jsx-runtime.development.js'),
      'react/jsx-dev-runtime$': resolve(__dirname, './node_modules/react/cjs/react-jsx-dev-runtime.development.js'),
    },
  },
  // Tauri 2.x 开发服务器配置
  server: {
    port: 1420,
    strictPort: true,
  },
  // 构建配置
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
  // 测试配置
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    globals: true,
    env: {
      NODE_ENV: 'test',
    },
    // Windows 终端输出优化：禁用线程隔离以提高稳定性
    poolOptions: {
      threads: {
        isolate: false,
      },
    },
  },
})
