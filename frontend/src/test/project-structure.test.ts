import { describe, it, expect, vi } from 'vitest'

// 强制使用 node 环境以支持 fs 和 path 模块
vi.stubGlobal('process', { env: { ...process.env } })

import fs from 'fs'
import path from 'path'

describe('项目结构验证测试', () => {
  const rootDir = path.dirname(__dirname).split(path.sep).slice(0, -2).join(path.sep)
  
  it('应该存在 src-tauri 目录', () => {
    const tauriDir = path.join(rootDir, 'src-tauri')
    expect(fs.existsSync(tauriDir)).toBe(true)
  })
  
  it('应该存在 frontend 目录', () => {
    const frontendDir = path.join(rootDir, 'frontend')
    expect(fs.existsSync(frontendDir)).toBe(true)
  })
  
  it('应该存在 Cargo.toml', () => {
    const cargoToml = path.join(rootDir, 'src-tauri', 'Cargo.toml')
    expect(fs.existsSync(cargoToml)).toBe(true)
    
    const content = fs.readFileSync(cargoToml, 'utf-8')
    expect(content).toContain('tauri')
    expect(content).toContain('rusqlite')
  })
  
  it('应该存在 tauri.conf.json', () => {
    const tauriConf = path.join(rootDir, 'src-tauri', 'tauri.conf.json')
    expect(fs.existsSync(tauriConf)).toBe(true)
    
    const config = JSON.parse(fs.readFileSync(tauriConf, 'utf-8'))
    expect(config.productName).toBe('ArcaneCodex')
    expect(config.identifier).toBe('com.arcanecodex.app')
  })
  
  it('应该存在 package.json', () => {
    const packageJson = path.join(rootDir, 'frontend', 'package.json')
    expect(fs.existsSync(packageJson)).toBe(true)
    
    const pkg = JSON.parse(fs.readFileSync(packageJson, 'utf-8'))
    expect(pkg.dependencies.react).toBeDefined()
    expect(pkg.dependencies.zustand).toBeDefined()
  })
  
  it('应该存在 tailwind.config.js', () => {
    const tailwindConf = path.join(rootDir, 'frontend', 'tailwind.config.js')
    expect(fs.existsSync(tailwindConf)).toBe(true)
  })
  
  it('应该存在 vite.config.ts', () => {
    const viteConf = path.join(rootDir, 'frontend', 'vite.config.ts')
    expect(fs.existsSync(viteConf)).toBe(true)
  })
})
