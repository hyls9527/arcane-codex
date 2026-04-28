# Arcane Codex

> 本地图片知识库桌面应用 - 智能整理、语义搜索、AI 分析

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://tauri.app)
[![React](https://img.shields.io/badge/React-18-61DAFB.svg)](https://react.dev)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

## ✨ 核心功能

### 🖼️ 图片管理
- **批量导入** - 支持 JPG/PNG/WebP/GIF/BMP 格式
- **智能去重** - BK-Tree pHash 算法，O(n log n) 性能
- **缩略图生成** - 懒加载 + 虚拟滚动，5000+ 图片流畅浏览

### 🤖 AI 分析
- **多推理源支持** - LM Studio / Ollama / Hermes / 智谱 / OpenAI / OpenRouter
- **自动标签** - 5-10 个关键词，中文优先
- **分类归档** - 风景/人物/物品/动物/建筑/文档/其他
- **置信度校准** - ECE 算法，可靠性分级

### 🔍 语义搜索
- **中文分词** - jieba-rs 倒排索引
- **语义理解** - 同义词/近义词匹配
- **多条件筛选** - 标签/分类/时间/文件大小

### 🏷️ 用户反馈
- **标签修正** - 记录修正历史，持续优化
- **错误模式** - 识别高频错误，主动规避

## 🛠️ 技术栈

```
┌─────────────────────────────────────────────────────────────┐
│  前端 (Frontend)                                             │
│  ├── React 18 + TypeScript                                  │
│  ├── Tailwind CSS + Framer Motion                           │
│  ├── Zustand 状态管理                                        │
│  └── i18n 国际化 (zh/en)                                     │
├─────────────────────────────────────────────────────────────┤
│  后端 (Backend)                                              │
│  ├── Tauri 2.x (Rust)                                       │
│  ├── SQLite + rusqlite                                      │
│  ├── reqwest HTTP 客户端                                     │
│  └── tokio 异步运行时                                        │
├─────────────────────────────────────────────────────────────┤
│  AI 集成                                                     │
│  ├── 本地: LM Studio / Ollama / Hermes                      │
│  └── 云端: 智谱 GLM-4 / OpenAI / OpenRouter                 │
└─────────────────────────────────────────────────────────────┘
```

## 📦 安装

### 系统要求
- Windows 10/11
- 内存: 4GB+
- 磁盘: 100MB+ (不含图片)

### 下载
从 [Releases](https://github.com/hyls9527/arcane-codex/releases) 下载最新版本

### 开发环境
```bash
# 克隆仓库
git clone https://github.com/hyls9527/arcane-codex.git
cd arcane-codex

# 安装依赖
npm install
cd src-tauri && cargo build

# 启动开发服务器
npm run tauri dev
```

## 🚀 使用指南

### 1. 配置 AI 服务
打开设置 → AI 配置，选择推理源：
- **本地**: 启动 LM Studio (1234) / Ollama (11434) / Hermes (18789)
- **云端**: 输入 API Key (智谱/OpenAI/OpenRouter)

### 2. 导入图片
- 拖拽图片到应用窗口
- 或点击"导入"按钮选择文件夹

### 3. 智能分析
- 自动为图片生成标签和描述
- 点击"AI 分析"面板查看结果

### 4. 语义搜索
- 输入自然语言描述，如"去年夏天的海边照片"
- 支持标签组合搜索，如"猫 白色 室内"

## 📊 项目状态

| 阶段 | 状态 | 进度 |
|------|------|------|
| 规划 | ✅ 完成 | 3 轮迭代 |
| 开发 | ✅ 完成 | 202/202 任务 |
| 测试 | ✅ 完成 | 117/121 (96.7%) |
| 交付 | 🔄 进行中 | 等待用户验收 |

## 🗺️ 路线图

- [x] v1.0.0-rc - 核心功能完成
- [ ] v1.1.0 - HEIC/HEIF 支持
- [ ] v1.2.0 - 云端同步
- [ ] v2.0.0 - 多用户协作

## 🤝 贡献

欢迎提交 Issue 和 PR！

## 📄 许可证

MIT License © 2024 Arcane Codex

---

> 用 AI 整理你的数字记忆
