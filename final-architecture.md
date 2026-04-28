# 📖 Arcane Codex v2.0 - 技术架构与实施方案 (Final)

> **项目代号**: Arcane Codex Rebuild  
> **版本**: 1.0.0-rc  
> **日期**: 2026-04-27  
> **状态**: 🟡 待用户确认 (Pending Approval)  
> **工作区**: `e:\knowledge base\`  
> **原项目**: `D:\Personal\Desktop\arcane-codex-src\` (⚠️ 只读参考，严禁修改)

---

## 🎯 一、项目定位与核心目标

### 1.1 一句话介绍
一款基于 **Tauri 2.x (纯 Rust) + React 18 + SQLite + LM Studio** 的新一代本地图片知识库桌面应用，通过多模态 AI 实现智能理解、隐私安全、离线可用、极具创新视觉体验的个人知识管理系统。

### 1.2 核心设计哲学
| 原则 | 说明 |
|------|------|
| **Local-First (本地优先)** | 所有数据存储、AI 推理均在本地完成，不依赖任何云服务 |
| **Privacy-First (隐私优先)** | 不上传任何用户数据，不采集任何遥测信息 |
| **Zero-Config (零配置)** | 开箱即用，SQLite 嵌入式数据库无需安装额外服务 |
| **Modular (模块化)** | 清晰的分层架构，支持未来扩展插件/第三方 AI 服务 |
| **Progressive (渐进式)** | MVP 先保证基础功能完善，后续版本逐步增强 |

### 1.3 目标用户画像
| 用户类型 | 核心痛点 | 使用场景 |
|---------|---------|---------|
| **个人收藏者** | 图片散落各处、难以检索 | 整理旅行照片、设计素材、截图收藏 |
| **内容创作者** | 素材管理混乱、标签缺失 | 管理设计灵感、参考图库、mood board |
| **隐私敏感用户** | 拒绝云端上传、担心数据泄露 | 本地化存储、完全离线操作 |
| **AI 研究者** | 需要批量处理图像数据 | 测试多模态模型、构建训练数据集 |

---

## 🏗 二、技术架构设计

### 2.1 整体架构图 (Single-Binary Architecture)

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Tauri 2.x Desktop App                           │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Frontend Layer (WebView)                        │   │
│  │  ┌──────────┐ ┌──────────────┐ ┌──────────┐                │   │
│  │  │ React 18 │ │ Zustand +    │ │ React    │                │   │
│  │  │ + TSX    │ │ React Query  │ │ Virtual  │                │   │
│  │  │          │ │ (State)      │ │ Grid     │                │   │
│  │  └────┬─────┘ └──────┬───────┘ └────┬─────┘                │   │
│  │       └──────────────┼───────────────┘                      │   │
│  │              ▼                                              │   │
│  │  ┌──────────────────────────────────────────────────────┐  │   │
│  │  │           Tauri IPC Bridge (Commands + Events)        │  │   │
│  │  └──────────────────────┬───────────────────────────────┘  │   │
│  └─────────────────────────┼──────────────────────────────────┘   │
│                            ▼                                      │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │              Backend Layer (Rust Core)                       │   │
│  │                                                             │   │
│  │  ┌────────────┐  ┌────────────┐  ┌─────────────────────┐  │   │
│  │  │ File       │  │ Image      │  │ AI Task Queue       │  │   │
│  │  │ Manager    │  │ Processor  │  │ (async_channel)     │  │   │
│  │  │ (tokio/fs) │  │ (image)    │  │ + Semaphore (3)     │  │   │
│  │  └─────┬──────┘  └─────┬──────┘  └──────────┬──────────┘  │   │
│  │        │               │                     │              │   │
│  │        ▼               ▼                     ▼              │   │
│  │  ┌─────────────────────────────────────────────────────┐   │   │
│  │  │              Data Layer (SQLite)                     │   │   │
│  │  │  ┌──────────────┐  ┌─────────────────────────────┐  │   │   │
│  │  │  │ rusqlite     │  │ sqlite-vss (Vector Search)  │  │   │   │
│  │  │  │ (Images/Meta)│  │ (Embedding Similarity)      │  │   │   │
│  │  │  └──────────────┘  └─────────────────────────────┘  │   │   │
│  │  └─────────────────────────────────────────────────────┘   │   │
│  └────────────────────────────────────────────────────────────┘   │
│                            │                                      │
│                            ▼ (HTTP reqwest)                       │
│              ┌───────────────────────────────┐                   │
│              │    LM Studio Server (Local)    │                   │
│              │    localhost:1234/v1           │                   │
│              │    Qwen2.5-VL-7B-Instruct      │                   │
│              └───────────────────────────────┘                   │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 架构优势对比

| 对比项 | 原方案 (Rust + Python Sidecar) | **最终方案 (纯 Rust)** |
|--------|-------------------------------|-----------------------|
| **安装包大小** | ~200MB (含 Python 运行时) | **~10.5MB** |
| **进程数** | 2 个 (Rust + Python) | **1 个 (纯 Tauri)** |
| **IPC 开销** | 跨进程通信 (~10-50ms) | **零** (进程内调用) |
| **启动时间** | ~2-3 秒 | **<1 秒** |
| **内存占用** | ~500MB (Python 运行时) | **~50MB** |
| **打包复杂度** | PyInstaller + Tauri 双重打包 | **纯 Tauri 内置** |
| **崩溃风险** | 双进程需管理生命周期 | **单一进程，稳定性高** |

### 2.3 技术栈完整清单

#### 前端 (Frontend)
```json
{
  "core": {
    "react": "^18.3",
    "react-dom": "^18.3",
    "typescript": "^5.4",
    "@tauri-apps/api": "^2.0",
    "@tauri-apps/plugin-shell": "^2.0"
  },
  "state_data": {
    "zustand": "^4.5",
    "@tanstack/react-query": "^5.32"
  },
  "ui_styling": {
    "tailwindcss": "^3.4",
    "framer-motion": "^11.1",
    "clsx": "^2.1",
    "tailwind-merge": "^2.3"
  },
  "components": {
    "@radix-ui/react-dialog": "^1.0",
    "@radix-ui/react-dropdown-menu": "^2.0",
    "@radix-ui/react-toast": "^1.1",
    "lucide-react": "^0.378",
    "react-dropzone": "^14.2",
    "react-hot-toast": "^2.4"
  },
  "performance": {
    "@tanstack/react-virtual": "^3.5"
  },
  "i18n": {
    "i18next": "^23.11",
    "react-i18next": "^14.1"
  }
}
```

#### 后端 (Rust - Tauri Core)
```toml
# Cargo.toml
[package]
name = "arcane-codex"
version = "1.0.0"

[dependencies]
# Tauri 核心
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 异步运行时
tokio = { version = "1", features = ["full"] }
async-channel = "2.2"
futures = "0.3"

# 图像处理
image = "0.25"
imageproc = "0.24"
img_hash = "0.4"        # 感知哈希 (pHash/aHash/dHash)

# 数据库
rusqlite = { version = "0.31", features = ["bundled", "chrono", "limits"] }
# FTS5 由 rusqlite bundled 特性自动包含
# 中文分词: 使用 libsimple 扩展 (支持中文/拼音分词)
# Phase 2 再评估是否加载 sqlite-vss.dll 实现语义相似度搜索

# 编码/工具
base64 = "0.22"
mime_guess = "2.0"

# 中文分词 (搜索索引)
jieba-rs = { version = "0.7", features = ["tfidf", "default"] }

# HTTP 客户端
reqwest = { version = "0.12", features = ["json", "stream", "timeout"] }

# 错误处理
thiserror = "1"
anyhow = "1"

# 日志与追踪
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 文件系统
tokio-util = "0.7"
path-absolutize = "3.1"

# 加密/哈希
sha2 = "0.10"
hex = "0.4"

# 时间
chrono = { version = "0.4", features = ["serde"] }

[features]
custom-protocol = ["tauri/custom-protocol"]
```

---

## 💾 三、数据库设计

### 3.1 完整 Schema (含向量层)

```sql
-- ========================================
-- 1. 图片主表
-- ========================================
CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- 文件信息
    file_path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    file_hash TEXT,              -- SHA256 (用于精确去重)
    mime_type TEXT,
    
    -- 图像属性
    width INTEGER,
    height INTEGER,
    thumbnail_path TEXT,
    phash TEXT,                  -- 感知哈希 (用于相似去重)
    
    -- EXIF 元数据
    exif_data JSON,              -- 拍摄时间/设备/GPS 等
    
    -- AI 分析结果
    ai_status TEXT DEFAULT 'pending',  -- pending/processing/completed/failed
    ai_tags JSON,                -- ["日落", "海滩", "度假"]
    ai_description TEXT,         -- 自然语言描述
    ai_category TEXT,            -- 分类 (风景/人物/物品/文档)
    ai_confidence REAL,          -- 置信度 0.0-1.0
    ai_model TEXT,               -- 模型版本标识
    ai_processed_at DATETIME,
    ai_error_message TEXT,
    ai_retry_count INTEGER DEFAULT 0,
    
    -- 元数据
    source TEXT DEFAULT 'manual',  -- manual/import/watcher/agent
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- 索引
    CHECK (ai_status IN ('pending', 'processing', 'completed', 'failed'))
);

-- 索引优化
CREATE INDEX idx_images_status ON images(ai_status);
CREATE INDEX idx_images_created ON images(created_at DESC);
CREATE INDEX idx_images_phash ON images(phash);
CREATE INDEX idx_images_file_hash ON images(file_hash);

-- ========================================
-- 2. 标签索引表 (加速检索)
-- ========================================
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    count INTEGER DEFAULT 0
);

CREATE INDEX idx_tags_name ON tags(name);

-- ========================================
-- 3. 图片-标签关联表 (多对多)
-- ========================================
CREATE TABLE IF NOT EXISTS image_tags (
    image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (image_id, tag_id)
);

-- ========================================
-- 4. 全文索引表 (语义搜索 - MVP 方案)
-- ========================================
-- 方案 A: 使用 SQLite 内置 FTS5 + unicode61 分词 (简单但中文分词不精确)
-- 方案 B: 使用 libsimple 扩展 (支持中文/拼音分词，需编译为 .dll 加载)
-- 方案 C: 使用自定义 jieba-rs 分词器 + 倒排索引表 (纯 Rust，推荐)
-- 
-- MVP 采用方案 C: 基于 jieba 分词的倒排索引
CREATE TABLE IF NOT EXISTS search_index (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    term TEXT NOT NULL,           -- 分词后的词条
    image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    field TEXT NOT NULL,          -- 'description' | 'tags' | 'category'
    position INTEGER DEFAULT 0,   -- 词条在文本中的位置 (用于高亮)
    UNIQUE(term, image_id, field, position)
);

CREATE INDEX idx_search_index_term ON search_index(term);
CREATE INDEX idx_search_index_image ON search_index(image_id);

-- 触发器: 图片 AI 分析完成后自动更新搜索索引
-- (实际由 Rust 端 jieba 分词后批量插入)

-- ========================================
-- 4b. 向量嵌入表 (Phase 2 语义搜索预留)
-- ========================================
CREATE TABLE IF NOT EXISTS image_embeddings (
    image_id INTEGER PRIMARY KEY REFERENCES images(id) ON DELETE CASCADE,
    embedding BLOB,               -- 768 维 float32 向量 (Phase 2 启用)
    model_version TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ========================================
-- 6. 任务队列表 (AI 批处理)
-- ========================================
CREATE TABLE IF NOT EXISTS task_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    task_type TEXT NOT NULL,          -- auto_tag/dedup/export
    status TEXT DEFAULT 'pending',    -- pending/running/completed/failed
    priority INTEGER DEFAULT 0,       -- 优先级 (0=普通, 1=高)
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    error_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    started_at DATETIME,
    completed_at DATETIME,
    
    CHECK (status IN ('pending', 'running', 'completed', 'failed'))
);

CREATE INDEX idx_task_queue_status ON task_queue(status, priority DESC);

-- ========================================
-- 7. 系统配置表
-- ========================================
CREATE TABLE IF NOT EXISTS app_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 默认配置
INSERT OR IGNORE INTO app_config (key, value) VALUES
    ('ai_concurrency', '3'),
    ('ai_max_retries', '3'),
    ('thumbnail_quality', '80'),
    ('thumbnail_max_dimension', '2048'),
    ('dedup_similarity_threshold', '90'),
    ('watcher_enabled', 'false'),
    ('locale', 'zh-CN'),
    ('theme', 'system');
```

### 3.2 数据流设计

```
┌─────────────────────────────────────────────────────────────────┐
│                    图片导入流程                                    │
└─────────────────────────────────────────────────────────────────┘

[用户拖拽图片]
    ↓
[React UI] → Tauri Command: import_images(file_paths)
    ↓
[Rust Backend]
    ├── 1. 验证文件有效性 (大小/格式/SHA256 去重)
    ├── 2. 复制文件到应用数据目录 (或仅记录路径)
    ├── 3. 生成缩略图 (tokio::spawn_blocking, 300x200)
    ├── 4. 计算 pHash (感知哈希, 用于相似去重)
    ├── 5. 提取 EXIF 元数据 (拍摄时间/设备)
    ├── 6. 插入 images 表 (ai_status = 'pending')
    ├── 7. 创建 task_queue 记录
    └── 8. emit("import_complete", image_ids)
    ↓
[React UI 刷新] → 显示新导入图片 (缩略图加载中状态)

┌─────────────────────────────────────────────────────────────────┐
│                    AI 自动打标流程                                 │
└─────────────────────────────────────────────────────────────────┘

[task_queue 扫描器]
    ↓
1. 查询 status='pending' 任务 (LIMIT 100)
    ↓
2. 批量推入 async_channel (背压队列, 容量 1000)
    ↓
[3 个并发 Worker]
    ├── Worker 1: 获取 semaphore permit
    │   ├── 读取图片路径 + 编码为 base64
    │   ├── 更新 status='processing'
    │   ├── POST http://localhost:1234/v1/chat/completions
    │   │   └── 多模态 Prompt: "分析图像内容,返回标签/描述/分类"
    │   ├── 解析 AI 响应 (JSON 格式)
    │   ├── 更新 images 表 (tags/description/category)
    │   ├── 插入/更新 tags 表 (多对多关联)
    │   ├── jieba 分词 → 写入 search_index 倒排索引表
    │   └── emit("ai_completed", image_id)
    │
    └── 错误处理:
        ├── HTTP 超时/失败 → retry_count++
        ├── retry < 3 → 延迟 5s 重新入队
        └── retry >= 3 → status='failed', emit("ai_failed")

┌─────────────────────────────────────────────────────────────────┐
│                    语义检索流程 (jieba 倒排索引 MVP)               │
└─────────────────────────────────────────────────────────────────┘

[用户输入查询: "找一张日落的海滩照片"]
    ↓
[React UI] → Tauri Command: semantic_search(query, limit, filters)
    ↓
[Rust Backend]
    ├── 1. jieba-rs 对用户查询分词 → ["日落", "海滩", "照片"]
    ├── 2. 搜索 search_index: SELECT DISTINCT image_id, COUNT(*) as score 
    │         FROM search_index WHERE term IN ('日落', '海滩') 
    │         GROUP BY image_id ORDER BY score DESC
    ├── 3. JOIN images 表获取完整元数据 + 缩略图路径
    └── 4. 返回图片列表 + 相关性分数 (匹配词条数)
    ↓
[React UI] → react-virtual 虚拟滚动渲染结果网格

┌─────────────────────────────────────────────────────────────────┐
│              Phase 2: 向量语义搜索 (预留接口)                      │
└─────────────────────────────────────────────────────────────────┘

[用户输入查询: "找一张日落的海滩照片"]
    ↓
1. 调用 LM Studio: POST /v1/embeddings (文本转 768 维向量)
2. Rust 计算: 对每个 image_embedding 计算余弦相似度
3. 按 similarity DESC 排序, 返回 top N image_ids
4. JOIN images 表获取完整元数据
5. 返回图片列表 + 相似度分数
```

---

## 🎨 四、UI/UX 设计规范

### 4.1 视觉风格: "现代神秘主义" (Modern Mysticism)

#### 设计原则
1. **极简而不简单**: 大面积留白 + 精致微交互 (60fps 动画)
2. **科技与古典融合**: 玻璃拟态 (Glassmorphism) + 金色装饰线条
3. **信息层级清晰**: 三级视觉层次（背景 → 内容 → 强调）
4. **响应式过渡**: 尊重 `prefers-reduced-motion`, 300ms 平滑动画

#### 色彩系统 (Design Tokens)
```css
:root {
  /* 主色调 */
  --color-primary: #6C63FF;        /* 神秘紫 (科技感) */
  --color-primary-hover: #5A52E0;
  --color-primary-active: #4841C2;
  
  /* 辅助色 */
  --color-secondary: #FFD700;      /* 金色 (典籍质感) */
  
  /* 背景色 - Light Mode */
  --color-bg-primary: #FAFAFA;
  --color-bg-secondary: #FFFFFF;
  --color-bg-tertiary: #F3F4F6;
  
  /* 背景色 - Dark Mode */
  --color-bg-primary-dark: #0A0A0F;
  --color-bg-secondary-dark: #1A1A2E;
  --color-bg-tertiary-dark: #16213E;
  
  /* 文字色 */
  --color-text-primary: #1A1A2E;
  --color-text-secondary: #6B7280;
  --color-text-inverse: #FAFAFA;
  
  /* 状态色 */
  --color-success: #10B981;        /* 翡翠绿 */
  --color-warning: #F59E0B;        /* 琥珀黄 */
  --color-error: #EF4444;          /* 珊瑚红 */
  --color-info: #3B82F6;           /* 天空蓝 */
  
  /* 边框/分割线 */
  --color-border: #E5E7EB;
  --color-border-dark: #2D3748;
  
  /* 阴影 */
  --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
  --shadow-glow: 0 0 20px rgba(108, 99, 255, 0.3);
}
```

#### 字体系统
```css
:root {
  --font-sans: 'Inter', 'Noto Sans SC', system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', monospace;
  
  --font-size-xs: 0.75rem;    /* 12px */
  --font-size-sm: 0.875rem;   /* 14px */
  --font-size-base: 1rem;     /* 16px */
  --font-size-lg: 1.125rem;   /* 18px */
  --font-size-xl: 1.25rem;    /* 20px */
  --font-size-2xl: 1.5rem;    /* 24px */
  --font-size-3xl: 1.875rem;  /* 30px */
}
```

#### 间距系统 (8px Grid)
```css
:root {
  --spacing-1: 0.25rem;   /* 4px */
  --spacing-2: 0.5rem;    /* 8px */
  --spacing-3: 0.75rem;   /* 12px */
  --spacing-4: 1rem;      /* 16px */
  --spacing-5: 1.25rem;   /* 20px */
  --spacing-6: 1.5rem;    /* 24px */
  --spacing-8: 2rem;      /* 32px */
  --spacing-10: 2.5rem;   /* 40px */
  --spacing-12: 3rem;     /* 48px */
  --spacing-16: 4rem;     /* 64px */
}
```

### 4.2 核心页面布局

#### 主页 (Dashboard)
```
┌─────────────────────────────────────────────────────────────────┐
│ 📖 Arcane Codex          [🔍 搜索...]  [⚙️] [🌙/☀️]             │
├─────────────┬───────────────────────────────────────────────────┤
│             │                                                    │
│  📂 图库    │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐    │
│  🤖 AI 打标 │  │ 🖼️   │ │ 🖼️   │ │ 🖼️   │ │ 🖼️   │ │ 🖼️   │    │
│  🔍 检索    │  │      │ │      │ │      │ │      │ │      │    │
│  📊 统计    │  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘    │
│  ⚙️ 设置    │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐    │
│             │  │ 🖼️   │ │ 🖼️   │ │ 🖼️   │ │ 🖼️   │ │ 🖼️   │    │
│             │  │      │ │      │ │      │ │      │ │      │    │
│  ─────────  │  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘    │
│             │                                                    │
│  📊 状态    │  [虚拟滚动, 仅渲染可视区域]                          │
│  🗂️ 5,231 张│                                                    │
│  🤖 4,892 已│  [加载更多...]                                       │
│  ⏳ 339 待  │                                                    │
│             │                                                    │
└─────────────┴────────────────────────────────────────────────────┘
```

#### AI 处理面板 (Progress Visualization)
```
┌─────────────────────────────────────────────────────────────────┐
│  🤖 AI 自动打标进度                                               │
│                                                                 │
│  ┌───────────────────────────────────────────────────────┐     │
│  │  ████████████████████░░░░░░░░░░░░░░░  63%             │     │
│  │                                                         │     │
│  │  3,298 / 5,231 张已处理                                 │     │
│  │  预计剩余时间: ~12 分钟                                  │     │
│  │                                                         │     │
│  │  ✓ 成功: 3,156   ⚠ 失败: 142   ⟳ 重试中: 15            │     │
│  └───────────────────────────────────────────────────────┘     │
│                                                                 │
│  [⏸ 暂停]  [▶ 继续]  [❌ 取消]                                 │
│                                                                 │
│  最近处理结果:                                                   │
│  ┌─────┬──────────────────────────────────────────────┐        │
│  │ 🖼️  │ sunset_beach.jpg                             │        │
│  │     │ Tags: 日落, 海滩, 橙色天空, 海浪, 度假          │        │
│  │     │ Status: ✓ Completed (1.2s)                    │        │
│  └─────┴──────────────────────────────────────────────┘        │
│  ┌─────┬──────────────────────────────────────────────┐        │
│  │ 🖼️  │ city_night.jpg                               │        │
│  │     │ Status: ⚠ Failed (Timeout after 30s)         │        │
│  │     │ [🔄 重试]                                      │        │
│  └─────┴──────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

### 4.3 明暗主题切换

```tsx
// ThemeProvider.tsx
const ThemeContext = createContext<ThemeContextType>({
  theme: 'system',
  toggleTheme: () => {},
});

// 跟随系统 (默认)
useEffect(() => {
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  setTheme(mediaQuery.matches ? 'dark' : 'light');
  
  const handler = (e: MediaQueryListEvent) => {
    setTheme(e.matches ? 'dark' : 'light');
  };
  mediaQuery.addEventListener('change', handler);
  return () => mediaQuery.removeEventListener('change', handler);
}, []);

// CSS 类名切换
<html className={theme === 'dark' ? 'dark' : ''}>
```

---

## 🤖 五、AI 集成方案

### 5.1 LM Studio API 对接

#### 模型配置
```yaml
服务地址: http://localhost:1234/v1
模型名称: qwen2.5-vl-7b-instruct
模型路径: D:\AI\Models\
兼容协议: OpenAI Chat Completions API
```

#### 图像分析 Prompt 模板
```json
{
  "system_prompt": "You are an expert image analyst. Analyze the image and return structured JSON.",
  "user_prompt": "请分析这张图片,并以以下 JSON 格式返回:\n{\n  \"tags\": [\"标签1\", \"标签2\", \"标签3\"],\n  \"description\": \"一句话描述图片内容\",\n  \"category\": \"风景|人物|物品|动物|建筑|文档|其他\",\n  \"confidence\": 0.95\n}\n要求:\n- tags: 5-10个关键词,中文优先,避免重复和过于宽泛的词(如\"图片\")\n- description: 简洁准确,1-2句话,不超过50字\n- category: 从上述分类中选择一个",
  "parameters": {
    "max_tokens": 512,
    "temperature": 0.3,
    "response_format": { "type": "json_object" }
  }
}
```

#### Rust 调用代码
```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    response_format: ResponseFormat,
}

#[derive(Deserialize)]
struct AIResult {
    tags: Vec<String>,
    description: String,
    category: String,
    confidence: f32,
}

pub async fn analyze_image(
    client: &Client,
    image_path: &str,
) -> Result<AIResult> {
    // 1. 读取图片并编码为 base64
    let image_data = tokio::fs::read(image_path).await?;
    let base64_image = encode_base64(&image_data);
    let mime_type = detect_mime(image_path)?;
    
    // 2. 构建请求
    let request = ChatRequest {
        model: "qwen2.5-vl-7b-instruct".into(),
        messages: vec![
            Message {
                role: "system".into(),
                content: SYSTEM_PROMPT.into(),
            },
            Message {
                role: "user".into(),
                content: format!(
                    "![image](data:{};base64,{})\n\n{}",
                    mime_type, base64_image, USER_PROMPT
                ),
            },
        ],
        max_tokens: 512,
        temperature: 0.3,
        response_format: ResponseFormat { r#type: "json_object".into() },
    };
    
    // 3. 发送请求 (带超时)
    let response = client
        .post("http://localhost:1234/v1/chat/completions")
        .timeout(Duration::from_secs(60))
        .json(&request)
        .send()
        .await?;
    
    // 4. 解析响应
    let result: ChatResponse = response.json().await?;
    let ai_result: AIResult = serde_json::from_str(&result.choices[0].message.content)?;
    
    Ok(ai_result)
}
```

### 5.2 向量嵌入生成 (语义搜索)

```rust
pub async fn generate_embedding(
    client: &Client,
    text: &str,
) -> Result<Vec<f32>> {
    let request = EmbeddingRequest {
        model: "qwen2.5-vl-7b-instruct".into(),
        input: text.into(),
    };
    
    let response = client
        .post("http://localhost:1234/v1/embeddings")
        .timeout(Duration::from_secs(30))
        .json(&request)
        .send()
        .await?;
    
    let result: EmbeddingResponse = response.json().await?;
    Ok(result.data[0].embedding.clone())
}
```

---

## 📦 六、打包与发布

### 6.1 Tauri 内置打包配置

```json
// tauri.conf.json
{
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build"
  },
  "productName": "Arcane Codex",
  "version": "1.0.0",
  "identifier": "com.arcanecodex.app",
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": "",
      "nsis": {
        "installerIcon": "icons/installer.ico",
        "headerImage": "icons/header.bmp",
        "sidebarImage": "icons/sidebar.bmp",
        "displayLanguageSelector": true,
        "languages": ["English", "SimpChinese"],
        "installMode": "both",
        "compression": "lzma"
      }
    }
  },
  "plugins": {
    "updater": {
      "active": true,
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEFCQ0RFRjAxMjM0NTY3ODkw...",
      "endpoints": [
        "https://releases.arcanecodex.com/{{target}}/{{arch}}/{{current_version}}"
      ]
    }
  }
}
```

### 6.2 打包产物预估

| 产物 | 大小 | 说明 |
|------|------|------|
| **安装包 (.exe)** | **~10.5MB** | NSIS 压缩后 |
| **安装后占用** | **~50MB** | 解压后应用文件 |
| **内存占用 (运行中)** | **~50-100MB** | 视图片库规模 |

### 6.3 构建流程

```bash
# 1. 安装依赖
pnpm install
cargo fetch

# 2. 开发模式
cargo tauri dev

# 3. 生产构建
cargo tauri build

# 输出目录:
# src-tauri/target/release/bundle/nsis/
# └── Arcane.Codex_1.0.0_x64-setup.exe
```

---

## 🧪 七、测试与质量保障

### 7.1 测试策略

| 测试类型 | 工具 | 覆盖范围 | 执行时机 |
|---------|------|---------|---------|
| **单元测试** | Vitest (前端) + cargo test (Rust) | 核心逻辑/工具函数 | 每次 commit |
| **集成测试** | Playwright | 用户旅程 (导入→打标→检索) | CI/CD |
| **性能测试** | Lighthouse + Custom Scripts | 虚拟滚动/内存泄漏 | 每周 |
| **手动测试** | 真实用户场景 | 边界情况/异常恢复 | 发布前 |

### 7.2 关键测试用例 (MVP)

```markdown
## T-001: 图片导入功能
- [ ] 拖拽单张图片到主界面 → 成功导入并显示
- [ ] 拖拽包含 100 张图片的文件夹 → 批量导入,进度条正确
- [ ] 导入重复图片 (SHA256 相同) → 提示重复并跳过
- [ ] 导入损坏图片 → 记录错误,不崩溃

## T-002: AI 自动打标
- [ ] 导入图片后自动触发 AI 分析 → 状态从 pending → processing → completed
- [ ] LM Studio 未启动 → 显示错误提示,任务保持 pending
- [ ] 并发控制测试: 设置并发=3, 观察同时只有 3 个请求
- [ ] 失败重试: 模拟超时,验证 3 次重试后标记 failed
- [ ] 暂停/恢复: 点击暂停后队列停止,点击恢复后继续

## T-003: 智能去重
- [ ] 导入两张极相似图片 → pHash 检测,提示重复
- [ ] 并排对比视图 → 选择保留/删除
- [ ] 批量删除重复项 → 数据库记录正确清理

## T-004: 语义检索 (Phase 2)
- [ ] 输入"日落的海滩" → 返回相关图片 (即使标签是"夕阳+海边")
- [ ] 筛选器组合: 时间范围 + 分类 + 标签
- [ ] 无结果时显示友好提示

## T-005: 性能测试
- [ ] 5000 张图片列表滚动 → 60fps, 无卡顿
- [ ] 内存占用稳定 < 200MB (无泄漏)
- [ ] 应用启动时间 < 2 秒
- [ ] 缩略图加载不阻塞 UI
```

---

## 🗺 八、开发路线图 (Roadmap)

### Phase 1: MVP 基础版 (Week 1-4)

| 里程碑 | 任务 | 优先级 | 预估工时 |
|--------|------|--------|---------|
| **M1: 项目初始化** | Tauri 2.x + React 18 脚手架 | P0 | 4h |
| | 配置 TypeScript + Tailwind CSS | P0 | 2h |
| | 初始化 SQLite 数据库 + Migration 系统 | P0 | 4h |
| | 编写 `.trae/rules/project_rules.md` | P0 | 1h |
| **M2: 核心功能** | 实现文件导入 + 缩略图生成 (async) | P0 | 8h |
| | pHash 计算 + SHA256 去重 | P0 | 4h |
| | 图片网格展示 (react-virtual) | P0 | 6h |
| | 大图预览 + 缩放 | P0 | 4h |
| **M3: AI 集成** | LM Studio 客户端封装 (reqwest) | P0 | 6h |
| | AI 任务队列 (async_channel + Semaphore) | P0 | 8h |
| | 多模态 Prompt 模板 + 结果解析 | P0 | 4h |
| | 进度可视化面板 (暂停/恢复/重试) | P0 | 6h |
| **M4: 去重功能** | pHash 相似度计算 | P0 | 4h |
| | 重复项标记 + 并排对比 UI | P1 | 6h |
| | 批量删除/归档 | P1 | 2h |

### Phase 2: 增强版 (Week 5-6)

| 里程碑 | 任务 | 优先级 | 预估工时 |
|--------|------|--------|---------|
### M5: 语义检索 | jieba-rs 中文分词 + 倒排索引搜索 | P1 | 8h |
| | 语义搜索 API + UI (关键词高亮) | P1 | 6h |
| | 筛选器组合 (时间/标签/分类) | P1 | 4h |
| | [预留] 向量嵌入生成 + 余弦相似度计算 | P2 | 6h |
| **M6: 数据导出** | JSON/CSV 导出功能 | P1 | 4h |
| | 批量选择 + 导出进度 | P2 | 2h |
| **M7: 国际化** | i18next 配置 (中英双语) | P1 | 6h |
| | 所有 UI 文本翻译 | P1 | 4h |

### Phase 3: 打磨与发布 (Week 7-8)

| 里程碑 | 任务 | 优先级 | 预估工时 |
|--------|------|--------|---------|
| **M8: 测试** | 编写单元测试 (前端 + Rust) | P0 | 12h |
| | E2E 测试 (Playwright) | P0 | 8h |
| | 性能测试 + 内存泄漏检测 | P1 | 4h |
| **M9: 优化** | 虚拟滚动优化 (5万张图片场景) | P0 | 6h |
| | SQLite 索引优化 + 查询调优 | P1 | 4h |
| | 错误处理完善 (Toast/Dialog) | P1 | 4h |
| **M10: 打包** | Tauri 生产构建 + NSIS 安装包 | P0 | 4h |
| | 自动更新机制预留 | P2 | 2h |
| | 发布文档 + README | P1 | 2h |

---

## 📂 九、项目目录结构

```
e:\knowledge base\
├── .trae/
│   └── rules/
│       └── project_rules.md           # 团队开发规范
│
├── src-tauri/                         # Rust 后端
│   ├── src/
│   │   ├── main.rs                    # 入口
│   │   ├── commands/                  # Tauri Commands
│   │   │   ├── mod.rs
│   │   │   ├── images.rs              # 图片导入/管理
│   │   │   ├── ai.rs                  # AI 分析
│   │   │   ├── search.rs              # 检索
│   │   │   └── export.rs              # 导出
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   ├── db.rs                  # 数据库连接 + Migration
│   │   │   ├── image.rs               # 图像处理 (缩略图/pHash)
│   │   │   ├── ai_queue.rs            # 任务队列
│   │   │   └── lm_studio.rs           # LM Studio 客户端
│   │   ├── models/                    # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── image.rs
│   │   │   └── task.rs
│   │   └── utils/
│   │       ├── mod.rs
│   │       ├── hash.rs                # SHA256/pHash
│   │       └── error.rs               # 错误类型
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── icons/                         # 应用图标
│
├── frontend/                          # React 前端
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── components/                # UI 组件
│   │   │   ├── layout/
│   │   │   │   ├── Sidebar.tsx
│   │   │   │   ├── TopBar.tsx
│   │   │   │   └── MainLayout.tsx
│   │   │   ├── gallery/
│   │   │   │   ├── ImageGrid.tsx
│   │   │   │   ├── ImageCard.tsx
│   │   │   │   └── ImageViewer.tsx
│   │   │   ├── ai/
│   │   │   │   ├── AIProgressPanel.tsx
│   │   │   │   └── AITaskQueue.tsx
│   │   │   └── ui/                    # 基础组件 (Button/Modal/Toast)
│   │   ├── stores/                    # Zustand 状态管理
│   │   │   ├── imageStore.ts
│   │   │   └── aiStore.ts
│   │   ├── hooks/                     # 自定义 Hooks
│   │   │   ├── useImages.ts
│   │   │   └── useAI.ts
│   │   ├── i18n/                      # 国际化
│   │   │   ├── index.ts
│   │   │   ├── en.json
│   │   │   └── zh.json
│   │   └── styles/
│   │       ├── globals.css            # Tailwind + 自定义 Token
│   │       └── theme.ts
│   ├── index.html
│   ├── package.json
│   ├── tsconfig.json
│   ├── tailwind.config.js
│   └── vite.config.ts
│
├── test_assets/                       # 测试资源
│   ├── landscape.jpg
│   ├── portrait.jpg
│   └── object.jpg
│
├── docs/                              # 文档
│   ├── architecture.md
│   └── api-spec.md
│
├── pre-requirements.md                # 需求预分析文档
├── final-architecture.md              # 本文件
└── README.md
```

---

## ⚠️ 十、约束与假设

### 10.1 必须遵守 (Hard Constraints)
| 约束 | 说明 |
|------|------|
| **工作区隔离** | 所有新代码必须位于 `e:\knowledge base\`, 原项目仅参考 |
| **本地优先** | 不依赖任何云服务 (除 LM Studio 本地服务) |
| **Windows 10+** | 仅 targeting Windows 10 1809+ (Build 17763) |
| **离线可用** | 核心浏览/管理功能无需网络 |
| **隐私保护** | 不采集/上传任何用户数据 |
| **安装包大小** | ≤ 15MB (纯 Rust 架构可实现) |

### 10.2 技术假设 (Assumptions)
| 假设 | 说明 |
|------|------|
| **LM Studio** | 已安装且运行在 `localhost:1234` |
| **Qwen2.5-VL-7B** | 模型已加载到 LM Studio |
| **硬件要求** | 至少 16GB RAM (推荐 32GB 用于大模型推理) |
| **磁盘空间** | 应用 + 数据预计 < 500MB (不含图片本身) |
| **Python** | 不需要 (纯 Rust 架构) |

---

## 🎯 十一、风险评估与缓解策略

| 风险 | 影响 | 概率 | 缓解策略 |
|------|------|------|---------|
| **LM Studio 未启动** | AI 功能不可用 | 高 | 自动检测 + Toast 提示 + 任务队列保持 pending |
| **7B 模型响应慢** | 打标耗时过长 | 中 | 并发控制 (3) + 超时 60s + 用户可调整并发数 |
| **SQLite 锁冲突** | 写入失败 | 低 | WAL 模式 + 重试 3 次 + 队列缓冲 |
| **大文件内存溢出** | 应用崩溃 | 低 | 文件大小检查 (50MB 上限) + 流式处理 |
| **缩略图生成阻塞** | UI 卡顿 | 中 | `tokio::spawn_blocking` + 虚拟滚动 |
| **pHash 误判重复** | 错误删除 | 低 | 相似度阈值可调 (默认 90%) + 用户确认 |
| **向量维度不匹配** | 语义搜索失败 | 低 | 启动时检测模型版本 + 自动重建索引 |

---

## ✅ 十二、下一步行动

### 立即执行 (Pending Your Approval)

1. **📋 用户确认本文档** → 回复"确认"或提出修改意见
2. **🚀 启动 Ralph Planner Skill** → 进入正式规划阶段
3. **📝 生成 Ralph 标准文档**:
   - `01-requirements.md` (PRD 产品需求文档)
   - `02-architecture.md` (详细架构设计)
   - `03-routine.md` (开发常规)
   - `04-ralph-tasks.md` (原子化任务分解)
   - `05-test-plan.md` (测试计划)
   - `RALPH_STATE.md` (状态跟踪)

### 本周内完成 (Week 1)

4. **🏗 初始化项目脚手架**
   - Tauri 2.x + React 18 + TypeScript
   - 配置 Tailwind CSS + Zustand
   - 初始化 SQLite 数据库
5. **🎨 搭建 Design System**
   - 定义色彩/字体/间距 Token
   - 实现明暗主题切换
   - 开发核心组件原型
6. **📥 下载测试资源**
   - 从 Unsplash 下载 3 张免版权测试图
   - 放置到 `./test_assets/` 目录

---

## 📎 附录

### A. LM Studio 官方文档
- [LM Studio API 文档](https://lmstudio.ai/docs/api-reference)
- [OpenAI 兼容接口规范](https://platform.openai.com/docs/api-reference/chat)
- [Qwen2.5-VL 模型卡](https://huggingface.co/Qwen/Qwen2.5-VL-7B-Instruct)

### B. Tauri 官方文档
- [Tauri 2.x 指南](https://v2.tauri.app/)
- [Rust + Tauri 最佳实践](https://v2.tauri.app/develop/)
- [打包配置](https://v2.tauri.app/distribute/)

### C. Rust 生态参考
- [image crate 文档](https://docs.rs/image/latest/image/)
- [rusqlite 文档](https://docs.rs/rusqlite/latest/rusqlite/)
- [reqwest 文档](https://docs.rs/reqwest/latest/reqwest/)
- [tokio 异步指南](https://tokio.rs/tokio/tutorial)

### D. sqlite-vss 向量搜索
- [sqlite-vss GitHub](https://github.com/asg017/sqlite-vss)
- [向量相似度搜索教程](https://alexgarcia.xyz/blog/sqlite-vector-search/)

---

**文档结束** | **版本**: 1.0.0-rc | **最后更新**: 2026-04-27 | **状态**: 🟡 待确认
