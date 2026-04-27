import '@testing-library/jest-dom'
import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'

// 确保使用 React 开发模式
process.env.NODE_ENV = 'test'

// 每个测试后清理
afterEach(() => {
  cleanup()
})
