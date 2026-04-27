# 📖 Arcane Codex Rebuild - 需求参考文档 (Pre-Requirements)

> **项目代号**: Arcane Codex v2.0  
> **版本**: 0.1.0-pre  
> **日期**: 2026-04-27  
> **状态**: 待确认 (Pending Review)  
> **定位**: 本地优先图片知识库 (Local-First Image Knowledge Base)

---

## 🎯 一句话介绍 (Elevator Pitch)

一款基于 **Tauri 2.x + Python + LM Studio AI** 的新一代本地图片管理工具，通过多模态智能自动理解图像内容，实现隐私安全、离线可用、极具创新视觉体验的个人知识库系统。

---

## 👥 目标用户 (Target Users)

| 用户类型 | 核心痛点 | 使用场景 |
|---------|---------|---------|
| **个人收藏者** | 图片散落各处、难以检索 | 整理旅行照片、设计素材、截图收藏 |
| **内容创作者** | 素材管理混乱、标签缺失 | 管理设计灵感、参考图库、 mood board |
| **隐私敏感用户** | 拒绝云端上传、担心数据泄露 | 本地化存储、完全离线操作 |
| **AI 研究者** | 需要批量处理图像数据 | 测试多模态模型、构建训练数据集 |

---

## ✨ 核心功能矩阵 (Key Features)

### Phase 1: MVP 基础版 (Must-Have)

#### 1. 图片导入与管理 (Image Import & Management)
- **拖拽上传**: 支持文件/文件夹拖拽到主界面
- **批量选择**: Ctrl/Shift 多选，批量操作（删除、移动、导出）
- **缩略图生成**: 自动生成高质量缩略图（WebP 格式，质量可配置）
- **进度反馈**: 实时显示导入进度条、成功/失败计数
- **元数据提取**: EXIF 信息读取（拍摄时间、设备、GPS 等）
- **文件夹监控**: 可选监听指定目录，新图片自动入库

#### 2. AI 智能打标 (AI-Powered Auto Tagging)
- **模型集成**: Qwen2.5-VL-7B-Instruct via LM Studio API (`localhost:1234/v1`)
- **触发方式**:
  - ✅ 导入后自动批量处理（可配置并发数）
  - ✅ 手动选中后按需触发
  - ✅ 后台队列处理（不阻塞 UI）
- **输出内容**:
  - **智能标签** (Tags): 5-10 个关键词（如"日落"、"海滩"、"度假"）
  - **自然语言描述** (Description): 1-2 句话概括图像内容
  - **分类建议** (Category): 预设分类体系（风景/人物/物品/文档等）
  - **置信度评分** (Confidence): 0-1 分值，辅助筛选低质量结果
- **错误处理**:
  - 失败重试机制（最多 3 次）
  - 失败队列可视化（红色标记 + 错误日志）
  - 支持手动重新提交失败任务
- **性能优化**:
  - 并发控制（默认 3 并发，可配置）
  - 断点续传（记录已处理列表）
  - 缓存机制（避免重复分析相同图片）

#### 3. 智能去重 (Smart Deduplication)
- **感知哈希算法**: pHash (Perceptual Hash)
- **相似度阈值**: 可配置（默认 90% 相似度视为重复）
- **去重策略**:
  - 自动标记重复项（保留最高分辨率版本）
  - 手动审核模式（并排对比预览）
  - 批量删除/移动重复图片
- **可视化展示**: 相似度矩阵热力图（可选高级功能）

### Phase 2: 增强版 (Nice-to-Have)

#### 4. 语义检索 (Semantic Search) [预留接口]
- **自然语言查询**: 输入"找一张夕阳下的城市天际线"
- **向量嵌入**: 将标签/描述转换为向量表示
- **相似度排序**: 基于余弦相似度返回结果
- **筛选器组合**: 时间范围 + 标签 + 文件类型 + 分数过滤

#### 5. 数据导出与报告 (Data Export)
- **JSON 导出**: 完整元数据 + 标签信息（结构化格式）
- **CSV 导出**: 表格形式，便于 Excel 分析
- **HTML 报告**: 离线可浏览的静态页面（含缩略图网格）

#### 6. 多 AI Agent 协作 [预留接口]
- **标准化输入协议**: 定义图片导入的 API 规范
- **任务队列系统**: 支持外部 Agent 提交批处理任务
- **Webhook 回调**: 任务完成后的通知机制
- **权限控制**: API Key 认证 + 操作审计日志

---

## 🎨 UI/UX 设计哲学 (Design Philosophy)

### 视觉风格定位: "现代神秘主义" (Modern Mysticism)

#### 核心设计原则
1. **极简而不简单**: 大面积留白 + 精致微交互
2. **科技与古典融合**: 玻璃拟态 (Glassmorphism) + 金色装饰线条
3. **信息层级清晰**: 三级视觉层次（背景 → 内容 → 强调）
4. **响应式动画**: 60fps 流畅过渡，尊重 `prefers-reduced-motion`

#### 色彩系统 (Color Tokens)
```
主色调 (Primary):     #6C63FF (神秘紫 - 科技感)
辅助色 (Secondary):   #FFD700 (金色 - 典籍质感)
背景色 (Background):
  - Light Mode:       #FAFAFA (暖白)
  - Dark Mode:        #0A0A0F (深空黑)
文字色 (Text):
  - Primary:          #1A1A2E (深墨黑)
  - Secondary:        #6B7280 (中性灰)
强调色 (Accent):
  - Success:          #10B981 (翡翠绿)
  - Warning:          #F59E0B (琥珀黄)
  - Error:            #EF4444 (珊瑚红)
```

#### 组件库选型
- **前端框架**: React 18+ / Vue 3+ / Svelte 5 (待最终确认)
- **UI 组件库**: 
  - 方案 A: Tailwind CSS + Headless UI (极致灵活)
  - 方案 B: shadcn/ui + Radix UI (现代可访问性)
  - 方案 C: 自研 Design System (完全定制)
- **图标库**: Lucide Icons / Phosphor Icons (线性风格)
- **字体**:
  - 中文: 思源黑体 (Noto Sans SC)
  - 英文: Inter / Geist Sans

#### 明暗主题切换
- **跟随系统**: 默认检测 OS 主题偏好
- **手动切换**: 顶部栏提供一键切换按钮
- **过渡动画**: 300ms 渐变过渡（非闪烁式切换）

---

## 🏗 技术架构蓝图 (Technical Architecture)

### 整体架构图 (High-Level Architecture)

```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri Desktop App                        │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Frontend (WebView)                      │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐            │    │
│  │  │ React/Vue│ │  State   │ │  Router  │            │    │
│  │  │   UI     │ │ Management│ │          │            │    │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘            │    │
│  │       └────────────┼────────────┘                    │    │
│  │              ▼                                    │    │
│  │  ┌──────────────────────────────────┐               │    │
│  │  │      Tauri IPC Bridge           │               │    │
│  │  │  (invoke / listen / emit)       │               │    │
│  │  └──────────────┬───────────────────┘               │    │
│  └─────────────────┼───────────────────────────────────┘    │
│                    ▼                                        │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Backend (Rust Core)                     │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐            │    │
│  │  │ File I/O │ │  DB      │ │  Command │            │    │
│  │  │ Manager  │ │ SQLite   │ │  Sidecar │            │    │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘            │    │
│  └───────┼────────────┼────────────┼────────────────────┘    │
│          ▼            ▼            ▼                          │
│  ┌─────────────────────────────────────────────────────┐    │
│  │         Python Sidecar (AI Engine)                   │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐            │    │
│  │  │  Image   │ │  LM      │ │  Task    │            │    │
│  │  │ Processor│ │ Studio   │ │  Queue   │            │    │
│  │  │ (Pillow) │ │  Client  │ │ Manager  │            │    │
│  │  └──────────┘ └──────────┘ └──────────┘            │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │   LM Studio Server    │
              │   localhost:1234/v1   │
              │  Qwen2.5-VL-7B-GGUF  │
              └────────────────────────┘
```

### 数据流设计 (Data Flow)

```
[用户拖拽图片] 
    ↓
[Tauri Frontend] → [IPC: import_images]
    ↓
[Rust Backend] 
    ├──→ [File I/O]: 复制到应用数据目录
    ├──→ [Image Proc]: 生成缩略图 + 计算 pHash
    └──→ [SQLite]: 写入 images 表 (status = pending)
    ↓
[Python Sidecar] ← [Command: process_queue]
    ├──→ [读取待处理队列]
    ├──→ [调用 LM Studio API]
    │   └──→ POST /v1/chat/completions (image_url base64)
    ├──→ [解析 AI 响应] → tags, description, category
    └──→ [更新 SQLite] (status = completed + 元数据)
    ↓
[Frontend 轮询/WebSocket] → [刷新 UI 显示结果]
```

### 数据库 Schema 设计 (Simplified)

```sql
-- 图片主表
CREATE TABLE images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    file_size INTEGER,
    file_hash TEXT,
    phash TEXT,
    thumbnail_path TEXT,
    width INTEGER,
    height INTEGER,
    mime_type TEXT,
    exif_data JSON,
    
    -- AI 分析结果
    status TEXT DEFAULT 'pending',  -- pending/processing/completed/failed
    tags JSON,
    description TEXT,
    category TEXT,
    confidence REAL,
    ai_model TEXT,
    processed_at DATETIME,
    error_message TEXT,
    
    -- 元数据
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    source TEXT DEFAULT 'manual'
);

-- 标签索引表 (加速检索)
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    count INTEGER DEFAULT 0
);

-- 图片-标签关联表 (多对多)
CREATE TABLE image_tags (
    image_id INTEGER REFERENCES images(id),
    tag_id INTEGER REFERENCES tags(id),
    PRIMARY KEY (image_id, tag_id)
);

-- 任务队列表 (用于 AI 批处理)
CREATE TABLE task_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    image_id INTEGER REFERENCES images(id),
    task_type TEXT NOT NULL,  -- auto_tag/dedup/export
    status TEXT DEFAULT 'pending',
    retry_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    started_at DATETIME,
    completed_at DATETIME
);
```

---

## 🤖 多智能体协作规划 (Multi-Agent Architecture)

### Phase 1: 核心智能体角色定义

| 智能体代号 | 角色 | 职责范围 | 输入 | 输出 |
|-----------|------|---------|------|------|
| **ARCH-001** | 首席架构师 | 技术选型、架构设计、模块划分 | 需求文档 | 架构图 + 技术规范 |
| **UI-DES-002** | UI/UX 设计师 | 组件库搭建、主题系统、交互动画 | 设计规范 | Design System + Storybook |
| **FE-DEV-003** | 前端开发工程师 | 页面开发、状态管理、IPC 通信 | UI 设计稿 | React/Vue 组件代码 |
| **BE-DEV-004** | 后端/AI 工程师 | Python Sidecar、LM Studio 对接、图像处理 | API 文档 | Python 服务代码 |
| **QA-TEST-005** | 测试/打包工程师 | 单元测试、E2E 测试、安装包构建 | 功能代码 | 测试报告 + .exe 安装包 |

### 通信机制 (Communication Protocol)

```
┌─────────────┐
│  ARCH-001   │ ← 项目启动、需求评审
│  (架构师)   │ └──→ 输出: 01-requirements.md + 02-architecture.md
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐
│ UI-DES-002  │◄────►│ FE-DEV-003  │
│ (设计师)    │      │ (前端开发)  │
└──────┬──────┘      └──────┬──────┘
       │                    │
       └────────┬───────────┘
                ▼
       ┌─────────────┐
       │ BE-DEV-004  │ ◄── Python 服务 + AI 集成
       │ (后端开发)  │ └──→ 输出: sidecar/ 目录代码
       └──────┬──────┘
                │
                ▼
       ┌─────────────┐
       │ QA-TEST-005 │ ◄── 测试 + 打包
       │ (测试打包)  │ └──→ 输出: test_report.md + setup.exe
       └─────────────┘
```

### 错误回退策略 (Fallback Strategy)

| 故障场景 | 检测方式 | 回退措施 | 通知机制 |
|---------|---------|---------|---------|
| LM Studio 未启动 | HTTP 连接超时 | 暂停 AI 队列，提示用户启动服务 | Toast 通知 + 状态栏图标 |
| Python 环境缺失 | Sidecar 启动失败 | 引导安装 Python 依赖 | 安装向导弹窗 |
| SQLite 锁冲突 | 数据库写入异常 | 重试 3 次 + 队列缓冲 | 日志记录 + 静默恢复 |
| 磁盘空间不足 | 文件写入前检查 | 暂停导入，清理临时文件 | 警告对话框 |

---

## 📅 开发里程碑路线图 (Development Roadmap)

### Milestone 0: 项目初始化 (Week 1)
- [ ] 初始化 Tauri 2.x 项目脚手架
- [ ] 配置 Rust + Python 双语言开发环境
- [ ] 搭建 monorepo 结构 (frontend/ + backend/ + shared/)
- [ ] 编写 `.trae/rules/project_rules.md` 团队规范
- [ ] 创建 Git 仓库 + 分支策略 (main/dev/feature/*)

### Milestone 1: 基础架构 (Week 2-3)
- [ ] 实现 Tauri IPC 通信层 (Commands + Events)
- [ ] 完成 SQLite 数据库初始化 + Migration 系统
- [ ] 搭建 Python Sidecar 进程管理 (启动/停止/重启)
- [ ] 实现基础文件 I/O 操作 (复制/删除/移动)
- [ ] 编写单元测试框架 (Vitest + Pytest)

### Milestone 2: UI 框架搭建 (Week 3-4)
- [ ] 配置 Design System (色彩/字体/间距 Token)
- [ ] 实现明暗主题切换系统
- [ ] 开发核心布局组件:
  - Sidebar (导航 + 统计面板)
  - ImageGrid (瀑布流/网格视图)
  - ImageViewer (大图预览 + 缩放)
  - TopBar (搜索 + 操作按钮)
- [ ] 集成拖拽上传组件 (react-dropzone / vue-droppable)

### Milestone 3: 核心功能实现 (Week 5-7)
- [ ] **图片导入功能**:
  - 文件/文件夹选择对话框
  - 批量复制 + 缩略图生成 (WebP)
  - EXIF 元数据提取 (Pillow/Pillow-Heif)
  - pHash 计算集成 (imagehash 库)
- [ ] **AI 智能打标**:
  - LM Studio OpenAI 兼容客户端封装
  - Base64 图像编码 + 多模态 Prompt 模板
  - 任务队列系统 (并发控制 + 断点续传)
  - 结果解析 + SQLite 持久化
- [ ] **智能去重**:
  - pHash 相似度计算
  - 重复项标记 + 并排对比 UI
  - 批量删除/归档功能

### Milestone 4: 优化与测试 (Week 8-9)
- [ ] 性能优化:
  - 虚拟滚动 (处理 5万+ 图片列表)
  - 缩略图懒加载 + 缓存策略
  - SQLite 索引优化 (tags/phash 字段)
- [ ] E2E 测试 (Playwright):
  - 用户旅程: 导入 → 打标 → 检索 → 导出
  - 异常场景: 网络断开/服务未启动/磁盘满
- [ ] 错误处理完善:
  - 全局错误边界 (Error Boundary)
  - 用户友好的错误提示 (Toast/Dialog)
  - 崩溃日志收集 (sentry-cli / 自研)

### Milestone 5: 打包与发布 (Week 10)
- [ ] Tauri 打包配置 (`tauri.conf.json`):
  - Windows Installer (NSIS 3 / Inno Setup)
  - 应用签名证书 (Code Signing)
  - 图标 + 元数据 (版本号/描述/版权)
- [ ] Python 打包方案:
  - PyInstaller 打包为单文件 exe
  - 或内嵌 Python 运行时 (Tauri Plugin)
- [ ] 安装程序脚本:
  - 检测 LM Studio 是否已安装
  - 引导配置 AI 模型路径
  - 创建桌面快捷方式 + 开始菜单项
- [ ] 自动更新机制预留:
  - Tauri Updater 插件集成
  - 版本检查 API 端点设计

---

## 🔧 技术依赖清单 (Dependencies)

### 前端 (Frontend)
```json
{
  "core": {
    "tauri": "^2.x",
    "react": "^18.3",
    "typescript": "^5.4",
    "zustand": "^4.5",           // 状态管理
    "@tanstack/react-query": "^5.x", // 数据获取/缓存
    "react-router-dom": "^6.x",   // 路由
    "tailwindcss": "^3.4",        // 样式框架
    "framer-motion": "^11.x"      // 动画库
  },
  "ui": {
    "lucide-react": "^0.400+",    // 图标
    "react-dropzone": "^14.2",    // 拖拽上传
    "react-hot-toast": "^2.4",    // Toast 通知
    "recharts": "^2.12"           // 图表库 (统计功能)
  }
}
```

### 后端 (Rust - Tauri Core)
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }  # SQLite
tokio = { version = "1", features = ["full"] }
thiserror = "1"
log = "0.4"
```

### Python Sidecar (AI Engine)
```python
# requirements.txt
fastapi==0.111.*          # HTTP API (可选)
uvicorn==0.30.*           # ASGI 服务器
pillow==10.4.*            # 图像处理
pillow-heif==0.13.*       # HEIC 格式支持
imagehash==4.3.*          # 感知哈希计算
httpx==0.27.*             # HTTP 客户端 (LM Studio)
pydantic==2.8.*           # 数据验证
python-dotenv==1.0.*      # 环境变量管理
```

### 开发工具 (DevTools)
```json
{
  "testing": {
    "vitest": "^2.0",
    "@testing-library/react": "^15.x",
    "playwright": "^1.45"
  },
  "linting": {
    "eslint": "^9.x",
    "prettier": "^3.3",
    "rust-analyzer": "latest"
  },
  "packaging": {
    "tauri-cli": "^2.x",
    "nsis-tools": "latest",
    "electron-builder": "备用方案"
  }
}
```

---

## ⚠️ 约束与假设 (Constraints & Assumptions)

### 必须遵守 (Hard Constraints)
1. **工作区隔离**: 所有新代码必须位于 `e:\智能体项目优化\` 目录，原项目仅作参考
2. **本地优先**: 不依赖任何云服务（除 LM Studio 本地服务外）
3. **Windows 10+**: 仅 targeting Windows 10 1809+ (Build 17763)
4. **离线可用**: 核心浏览/管理功能无需网络，AI 功能需 LM Studio 在线
5. **隐私保护**: 不采集任何用户数据，不上传任何文件至外部服务器

### 技术假设 (Assumptions)
1. **LM Studio 已安装且运行在 `localhost:1234`**
2. **Qwen2.5-VL-7B-Instruct 模型已加载到 LM Studio**
3. **用户机器至少 16GB RAM（推荐 32GB 用于大模型推理）**
4. **磁盘空间充足（应用 + 数据预计 < 500MB，不含图片本身）**
5. **Python 3.11+ 已安装或通过打包工具内嵌**

---

## ❓ 待确认决策点 (Open Questions)

以下问题需在进入详细设计阶段前明确：

1. **前端框架最终选择**: React 18 vs Vue 3 vs Svelte 5？（影响组件生态和学习曲线）
2. **Python Sidecar 部署方式**:
   - 方案 A: 用户自行安装 Python 环境（轻量但门槛高）
   - 方案 B: PyInstaller 打包为 exe（体积大但零配置）⭐ 推荐
   - 方案 C: Tauri 内嵌 Python 运行时（实验性功能）
3. **LM Studio 启动检测**: 应用启动时是否自动检测/提示启动 LM Studio？
4. **图片存储策略**:
   - 方案 A: 移动原文件到应用数据目录（强管理但破坏原有目录结构）
   - 方案 B: 仅建立索引引用（非侵入式但原文件删除会导致失效）⭐ 推荐
5. **国际化 (i18n) 范围**: MVP 仅中文？还是同时支持英文？

---

## 📎 附录 (Appendix)

### A. LM Studio API 调用示例
```bash
# 测试 LM Studio 是否正常运行
curl http://localhost:1234/v1/models

# 调用 Qwen2.5-VL 进行图像理解
curl -X POST http://localhost:1234/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5-vl-7b-instruct",
    "messages": [
      {
        "role": "user",
        "content": [
          {"type": "text", "text": "描述这张图片的内容，生成5个标签"},
          {"type": "image_url", "image_url": {"url": "data:image/jpeg;base64,..."}}
        ]
      }
    ],
    "max_tokens": 512,
    "temperature": 0.7
  }'
```

### B. 测试图片下载链接 (Unsplash 免版权)
```bash
# 风景类
https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=800

# 人物类
https://images.unsplash.com/photo-1531746020798-e6953c6e8e04?w=800

# 物品类
https://images.unsplash.com/photo-1523275335684-37898b6baf30?w=800
```

### C. 参考资源
- [Tauri 2.x 官方文档](https://v2.tauri.app/)
- [LM Studio 文档](https://lmstudio.ai/docs/)
- [Qwen2.5-VL 模型卡](https://huggingface.co/Qwen/Qwen2.5-VL-7B-Instruct)
- [Inno Setup 打包教程](https://jrsoftware.org/isinfo.php)
- [sqlite-vss 向量搜索](https://github.com/asg017/sqlite-vss)

---

## ✅ 下一步行动 (Next Steps)

1. **用户确认本文档** → 标记状态为 `Approved`
2. **启动 Ralph Planner Skill** → 进入正式规划阶段
3. **生成详细技术规格**:
   - `01-requirements.md` (PRD 产品需求文档)
   - `02-architecture.md` (架构设计文档)
   - `04-ralph-tasks.md` (开发任务分解)
   - `05-test-plan.md` (测试计划)
4. **分配智能体角色** → 开始并行开发

---

**文档结束** | **版本**: 0.1.0-pre | **最后更新**: 2026-04-27
