# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2025-04-29

### Added
- 图片管理：批量导入、缩略图生成、EXIF 元数据提取
- AI 分析：多推理源支持（LM Studio、Ollama、Hermes、智谱、OpenAI、OpenRouter）
- 智能标签：自动提取 5-10 个关键词，支持中文
- 分类归档：风景/人物/物品/动物/建筑/文档/其他
- 语义搜索：基于 jieba 分词的中文搜索
- 智能去重：BK-Tree pHash 算法，O(n log n) 性能
- 用户反馈：标签修正记录、错误模式识别
- 置信度校准：ECE 算法，可靠性分级
- 国际化：中文/英文支持
- 虚拟滚动：支持 5000+ 图片流畅浏览

### Security
- CSP 内容安全策略
- 上下文隔离
- 错误信息脱敏

## [0.1.0] - 2024-XX-XX

### Added
- 项目初始化
- 基础架构搭建
