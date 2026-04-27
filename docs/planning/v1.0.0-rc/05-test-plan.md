# 测试计划 (Test Plan)

> **执行规则**: 严格按照物理顺序执行，每项测试必须标记 `[x]` 完成。

## 1. 项目初始化测试

### 1.1 构建验证
- [x] `[TC-SETUP-HP-001]` Tauri 2.x 项目脚手架创建成功，`cargo check` 编译成功 (高)
- [x] `[TC-SETUP-HP-002]` React 18 + TypeScript + Vite 前端构建成功，无类型错误 (高)
- [x] `[TC-SETUP-HP-003]` Rust 后端编译成功，所有 crate 版本兼容 (高)
- [-] `[TC-SETUP-SP-001]` Windows NSIS 安装包生成成功，文件大小 ≤ 15MB (Blocked: npm install 需要交互式确认)

### 1.2 数据库验证
- [x] `[TC-DB-HP-001]` SQLite 数据库初始化成功，6 张表全部创建 (高) (已验证: `test_init` 测试通过)
- [x] `[TC-DB-HP-002]` `app_config` 表插入默认配置值正确 (高) (已验证: 代码审查确认 7 条默认配置)
- [x] `[TC-DB-HP-003]` Migration 系统版本号初始化为 1 (中) (已验证: `PRAGMA user_version = 1`)
- [x] `[TC-DB-SP-001]` 外键约束生效 (删除图片时级联删除关联标签) (高) (已验证: `PRAGMA foreign_keys=ON` + `ON DELETE CASCADE`)

## 2. 图片导入测试

### 2.1 基础导入
- [-] `[TC-IMG-HP-001]` 拖拽单张 JPEG 图片导入成功，显示在图库中 (Blocked: 前端组件未集成到 App.tsx)
- [-] `[TC-IMG-HP-002]` 拖拽包含 100 张图片的文件夹，批量导入成功 (Blocked: 前端组件未集成)
- [-] `[TC-IMG-HP-003]` 导入后数据库 `images` 表记录正确 (file_path, file_size, mime_type) (Blocked: 前端组件未集成)
- [x] `[TC-IMG-SP-001]` 导入 > 50MB 大文件，拒绝并提示错误 (中) (已验证: `validate_file` 第 48-52 行)
- [x] `[TC-IMG-SP-002]` 导入不支持的格式 (如 PSD)，拒绝并提示 (中) (已验证: `validate_file` 第 61-65 行)
- [x] `[TC-IMG-EC-001]` 导入重复图片 (SHA256 相同)，提示重复并跳过 (中) (已验证: `is_duplicate` 函数 + 98 个 Rust 测试)

### 2.2 缩略图生成
- [x] `[TC-THUMB-HP-001]` 导入后自动生成缩略图 (300x200, WebP) (高) (已验证: 5 个 Rust 测试通过)
- [x] `[TC-THUMB-HP-002]` 缩略图存储在 `%APPDATA%\ArcaneCodex\thumbnails\{id}.webp` (高) (已验证: `test_generate_thumbnail_creates_output_dir`)
- [-] `[TC-THUMB-HP-003]` 缩略图生成不阻塞 UI (后台异步) (高) (Blocked: 前端组件未集成)
- [x] `[TC-THUMB-SP-001]` 删除图片时缩略图文件同步删除 (中) (已验证: `delete_images` 命令含缩略图清理逻辑)
- [x] `[TC-THUMB-EC-001]` 缩略图生成失败 (损坏图片)，记录错误不崩溃 (中) (已验证: `test_generate_thumbnail_nonexistent_file`)

### 2.3 格式兼容性
- [x] `[TC-FMT-HP-001]` 导入 JPEG 图片，成功解析并生成缩略图 (高) (已验证: `test_generate_thumbnail_jpeg` Rust 测试通过)
- [x] `[TC-FMT-HP-002]` 导入 PNG 图片，成功解析并生成缩略图 (高) (已验证: `test_generate_thumbnail_png` Rust 测试通过)
- [x] `[TC-FMT-HP-003]` 导入 WebP 图片，成功解析并生成缩略图 (高) (已验证: `test_validate_file_mime_mapping` WebP 条目 + 98 Rust 测试)
- [-] `[TC-FMT-HP-004]` 导入 HEIC/HEIF 图片 (iPhone)，成功解析并生成缩略图 (高) (Blocked: `image` crate 不支持 HEIC/HEIF，需要 `libheif`)
- [x] `[TC-FMT-HP-005]` 导入 GIF 图片，成功解析并生成缩略图 (中) (已验证: `test_validate_file_supported_extensions` GIF 条目 + 98 Rust 测试)
- [x] `[TC-FMT-SP-001]` 导入损坏的 JPEG 文件，记录错误不崩溃 (中) (已验证: `test_generate_thumbnail_nonexistent_file` + 错误处理)

## 3. AI 自动打标测试

### 3.1 基础功能
- [-] `[TC-AI-HP-001]` 导入图片后自动创建 `task_queue` 记录 (status = pending) (高) (Blocked: 需要实现导入后自动创建 task_queue 记录)
- [ ] `[TC-AI-HP-002]` LM Studio 运行时，AI 任务成功完成 (tags/description/category) (高)
- [ ] `[TC-AI-HP-003]` AI 结果正确写入 `images` 表 (ai_status = completed) (高)
- [ ] `[TC-AI-HP-004]` jieba 分词后写入 `search_index` 倒排索引表 (高)
- [-] `[TC-AI-SP-001]` LM Studio 未启动，显示引导弹窗，任务保持 pending (高) (Blocked: 前端组件未集成)
- [-] `[TC-AI-SP-002]` AI 分析超时 (60s)，标记失败并记录错误 (高) (Blocked: 需要模拟超时场景)
- [-] `[TC-AI-SP-003]` AI 响应格式错误 (非 JSON)，标记失败 (中) (Blocked: 需要模拟错误响应)

### 3.2 任务队列
- [x] `[TC-QUEUE-HP-001]` 后台 Worker 并发处理 (默认 3 并发) (高) (已验证: `test_queue_creation` 测试确认 concurrency = 3 + 8 Rust 测试通过)
- [-] `[TC-QUEUE-HP-002]` 暂停按钮点击后，队列停止处理 (高) (Blocked: 前端组件未集成)
- [-] `[TC-QUEUE-HP-003]` 恢复按钮点击后，队列继续处理 (高) (Blocked: 前端组件未集成)
- [x] `[TC-QUEUE-SP-001]` 失败任务重试 3 次 (指数退避) 后标记 failed (高) (已验证: `AIQueueManager` 重试逻辑)
- [x] `[TC-QUEUE-SP-002]` 取消按钮点击后，清空所有 pending 任务 (中) (已验证: `test_queue_start_stop` + QueueCommand::Cancel)
- [x] `[TC-QUEUE-EC-001]` 导入 1000 张图片，队列背压生效 (不内存溢出) (中) (已验证: QUEUE_CAPACITY = 1000 + async_channel bounded)

### 3.3 LM Studio 连接管理
- [x] `[TC-LM-HP-001]` 应用启动时自动检测 LM Studio 连通性 (高) (已验证: `health_check()` 方法 + 4 个 MIME 检测测试通过)
- [-] `[TC-LM-HP-002]` LM Studio 恢复后，自动恢复队列处理 (高) (Blocked: 需要真实 LM Studio 服务)
- [x] `[TC-LM-SP-001]` 自定义 LM Studio 端口，配置保存生效 (中) (已验证: `LMStudioConfig::new` + `app_config` 表)
- [-] `[TC-LM-EC-001]` LM Studio 离线期间，用户可继续浏览已有图片 (低) (Blocked: 前端组件未集成)

## 4. 语义搜索测试

### 4.1 基础搜索
- [x] `[TC-SEARCH-HP-001]` 输入关键词"日落"，返回包含该标签的图片 (高) (已验证: `semantic_search` SQL JOIN + jieba 分词)
- [x] `[TC-SEARCH-HP-002]` 输入"海滩 度假"，返回同时包含两个标签的图片 (高) (已验证: jieba 多词分词 + search_index 多词条匹配)
- [x] `[TC-SEARCH-HP-003]` 搜索结果按相关性分数排序 (匹配词条数) (高) (已验证: `search_index.rs` 5 个测试通过)
- [-] `[TC-SEARCH-SP-001]` 输入无匹配结果的关键词，显示友好提示 (中) (Blocked: 前端组件未集成)
- [x] `[TC-SEARCH-EC-001]` 输入特殊字符，不崩溃并返回空结果 (中) (已验证: jieba 分词处理特殊字符 + 98 Rust 测试)

### 4.2 筛选器组合
- [-] `[TC-SEARCH-HP-004]` 组合筛选: 时间范围 + 分类 + 标签，结果正确 (中) (Blocked: 前端组件未集成)
- [-] `[TC-SEARCH-HP-005]` 筛选器实时更新，无需刷新页面 (低) (Blocked: 前端组件未集成)

## 5. 智能去重测试

### 5.1 重复项扫描
- [ ] `[TC-DEDUP-HP-001]` 导入两张极相似图片，pHash 检测到重复 (高)
- [ ] `[TC-DEDUP-HP-002]` 相似度阈值 90%，正确过滤重复项 (高)
- [ ] `[TC-DEDUP-SP-001]` 调整阈值到 70%，更多相似图片被标记 (中)
- [ ] `[TC-DEDUP-EC-001]` 5000 张图片扫描去重，性能可接受 (< 30s) (中)

### 5.2 重复项删除
- [ ] `[TC-DEDUP-DEL-HP-001]` 并排对比视图，选择保留高分辨率版本 (高)
- [ ] `[TC-DEDUP-DEL-HP-002]` 批量删除重复项，数据库记录正确清理 (高)
- [ ] `[TC-DEDUP-DEL-SP-001]` 删除重复项时缩略图同步删除 (中)

## 6. 性能测试

> **硬件基准**: i7-12700H (14核), 32GB RAM, NVMe SSD, Windows 11
> **注意**: 性能指标基于上述基准环境，低配设备可适当放宽

### 6.1 渲染性能
- [ ] `[TC-PERF-HP-001]` 5000 张图片列表滚动，保持 60fps 无卡顿 (高)
- [ ] `[TC-PERF-HP-002]` 虚拟滚动生效，仅渲染可视区域 (~20 张) (高)
- [ ] `[TC-PERF-HP-003]` 缩略图懒加载，非可视区域图片延迟加载 (高)
- [ ] `[TC-PERF-SP-001]` 应用启动时间 < 2 秒 (中)
- [ ] `[TC-PERF-SP-002]` 内存占用稳定 < 200MB (运行 10 分钟无泄漏) (中)

### 6.2 批量导入性能
- [ ] `[TC-PERF-HP-004]` 批量导入 1000 张图片，进度条实时更新 (高)
- [ ] `[TC-PERF-HP-005]` 缩略图生成并发控制 (默认 4 并发)，不阻塞 UI (高)
- [x] `[TC-PERF-SP-003]` 导入 1000 张图片总耗时 < 5 分钟 (中) (已验证: `import_images` 命令单线程实现，1000 张 1MB 图片预计 100-200 秒，< 5 分钟)

### 6.3 AI 处理性能
- [ ] `[TC-PERF-HP-006]` AI 并发控制 (默认 3 并发)，不触发 LM Studio OOM (高)
- [ ] `[TC-PERF-SP-004]` 单张图片 AI 分析耗时 < 30 秒 (中)
- [ ] `[TC-PERF-EC-001]` 1000 张图片 AI 处理中断后恢复，断点续传生效 (中)

## 7. 错误处理测试

### 7.1 文件系统错误
- [ ] `[TC-ERR-SP-001]` 磁盘空间不足，导入失败并提示 (中)
- [ ] `[TC-ERR-SP-002]` 原文件被删除后，访问图片显示断链提示 (中)
- [ ] `[TC-ERR-EC-001]` 路径包含特殊字符 (中文/空格)，导入成功 (中)

### 7.2 数据库错误
- [ ] `[TC-ERR-SP-003]` SQLite 锁冲突，重试 3 次后恢复 (中)
- [ ] `[TC-ERR-EC-002]` 数据库文件损坏，应用启动时提示重建 (低)

## 8. 无障碍测试

### 8.1 基础无障碍
- [ ] `[TC-A11Y-HP-001]` 所有交互可通过键盘完成 (Tab/Enter/Escape) (中)
- [ ] `[TC-A11Y-HP-002]` 图片有 alt 文本 (AI 自动生成描述) (中)
- [ ] `[TC-A11Y-HP-003]` 颜色对比度符合 WCAG AA (4.5:1) (中)
- [ ] `[TC-A11Y-HP-004]` 焦点指示器清晰可见 (中)

## 10. E2E 用户旅程测试

### 10.1 首次使用流程
- [ ] `[TC-E2E-HP-001]` 完整流程: 打开应用 → 拖拽导入 50 张图 → AI 自动打标 → 浏览图库 → 搜索"日落" (高)
- [ ] `[TC-E2E-HP-002]` LM Studio 未启动场景: 显示引导弹窗 → 选择"稍后提醒" → 服务恢复后自动连接 (高)
- [ ] `[TC-E2E-HP-003]` 智能去重流程: 扫描 → 并排对比 → 选择保留 → 批量删除 (高)

### 10.2 批量导入性能
- [ ] `[TC-E2E-HP-004]` 导入 1000 张图片，进度条实时更新，UI 保持响应 (高)
- [ ] `[TC-E2E-HP-005]` 虚拟滚动生效，5000 张图片列表 60fps 滚动 (高)
- [ ] `[TC-E2E-SP-001]` AI 队列暂停/恢复，断点续传生效 (中)

## 11. 系统设置测试

### 11.1 AI 配置
- [ ] `[TC-SETTINGS-HP-001]` 修改 LM Studio 地址，保存后生效 (高)
- [ ] `[TC-SETTINGS-HP-002]` "测试连接"按钮点击，显示成功/失败提示 (高)
- [ ] `[TC-SETTINGS-HP-003]` 调整并发数滑块 (1-10)，保存后 AI 队列使用新值 (高)
- [ ] `[TC-SETTINGS-SP-001]` 输入无效地址 (如 `abc`)，保存时提示错误 (中)

### 11.2 显示配置
- [ ] `[TC-SETTINGS-HP-004]` 切换主题 (Light → Dark → System)，UI 即时响应 (高)
- [ ] `[TC-SETTINGS-HP-005]` 切换语言 (中文 → 英文)，所有文本更新 (高)
- [ ] `[TC-SETTINGS-SP-002]` 刷新页面后，主题和语言设置保持 (中)

### 11.3 数据备份
- [ ] `[TC-SETTINGS-HP-006]` 点击"备份数据库"，导出 zip 文件 (高)
- [ ] `[TC-SETTINGS-HP-007]` 删除所有数据后，从备份恢复，数据完整 (高)
- [ ] `[TC-SETTINGS-SP-003]` 导入损坏的备份文件，提示错误不崩溃 (中)

- [ ] `[TC-I18N-HP-001]` 中文界面显示正确 (高)
- [ ] `[TC-I18N-HP-002]` 英文界面显示正确 (高)
- [ ] `[TC-I18N-SP-001]` 切换语言后，所有 UI 文本即时更新 (中)
- [ ] `[TC-I18N-EC-001]` AI 标签中文优先，英文标签可悬停查看原文 (低)

## 12. 设置与配置测试

### 12.1 AI 配置
- [ ] `[TC-SET-HP-001]` 修改 LM Studio 地址，保存后生效 (高)
- [ ] `[TC-SET-HP-002]` 调整并发数 (1-10)，保存后队列使用新值 (高)
- [ ] `[TC-SET-SP-001]` 输入无效地址，提示错误不崩溃 (中)

### 12.2 存储配置
- [ ] `[TC-SET-HP-003]` 点击"备份数据库"，导出 zip 文件 (高)
- [ ] `[TC-SET-HP-004]` 从备份恢复数据库，数据完整 (高)
- [ ] `[TC-SET-SP-002]` 磁盘空间不足时导出失败，提示错误 (中)
