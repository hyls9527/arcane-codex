# API 密钥保险箱

本地 API Key 加密管理工具。AES-256-CBC 加密，密码强度检测，自动锁定。

## 安装

```bash
pip install cryptography
```

或

```bash
pip install pycryptodome
```

## 使用

双击 `启动API保险箱.bat` 或运行：

```bash
pythonw api-vault-gui.py
```

## 功能

- AES-256-CBC 加密存储
- 密码强度实时检测
- 搜索过滤
- 导入/导出 .env 文件
- 5 分钟无操作自动锁定
- 快捷键支持

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+O | 打开保险箱 |
| Ctrl+N | 新建保险箱 |
| Ctrl+K | 添加密钥 |
| Ctrl+D | 编辑选中 |
| Ctrl+L | 锁定 |
| Ctrl+E | 导出 .env |
| Ctrl+F | 搜索 |
| F5 | 解锁 |
| Delete | 删除选中 |

## 安全说明

- 保险箱密码请妥善保管，遗忘无法恢复
- `.env.keys.json.enc` 为加密文件，不含明文密钥
- 请勿将保险箱文件上传到公开仓库
