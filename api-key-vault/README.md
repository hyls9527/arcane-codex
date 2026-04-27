# Swarm Key Vault - API 密钥保险箱

本地 API Key 加密管理工具。AES-256-GCM 认证加密，PBKDF2 600k 迭代，暴力破解防护，签名代理。

## 安装

```bash
pip install cryptography
```

## 使用

### GUI 模式

双击 `启动密钥保险箱.bat` 或：

```bash
pythonw swarm_keyvault_gui.py
```

### CLI 模式

```bash
# 创建保险箱
python swarm_keyctl.py init

# 解锁
python swarm_keyctl.py session start

# 添加密钥
python swarm_keyctl.py set OPENAI_API_KEY --tags ai,llm --url https://api.openai.com

# 查看密钥
python swarm_keyctl.py get OPENAI_API_KEY
python swarm_keyctl.py get OPENAI_API_KEY --show

# 列出所有
python swarm_keyctl.py list
python swarm_keyctl.py list --search openai

# 轮换密钥
python swarm_keyctl.py rotate OPENAI_API_KEY

# 吊销密钥
python swarm_keyctl.py revoke OPENAI_API_KEY

# 删除密钥
python swarm_keyctl.py delete OPENAI_API_KEY

# 导入/导出
python swarm_keyctl.py import .env
python swarm_keyctl.py export .env

# 启动签名代理
python swarm_keyctl.py proxy --port 18239

# 查看操作日志
python swarm_keyctl.py log

# 锁定
python swarm_keyctl.py session stop
```

### 签名代理

启动代理后，应用通过 HTTP 获取密钥，无需复制粘贴：

```bash
curl http://127.0.0.1:18239/?key=OPENAI_API_KEY
```

## 安全特性

| 特性 | 说明 |
|------|------|
| AES-256-GCM | 认证加密，防篡改+防Padding Oracle |
| PBKDF2 600k 迭代 | OWASP 2025 推荐强度 |
| 暴力破解防护 | 指数退避（1s→2s→4s→...→300s） |
| 原子写入 | 先写临时文件再重命名，防崩溃丢数据 |
| 签名代理 | 密钥不经过剪贴板，直接HTTP注入 |
| 操作审计 | 所有操作记录日志 |
| 自动锁定 | 5分钟无操作自动锁定 |

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+N | 新建保险箱 |
| Ctrl+K | 添加密钥 |
| Ctrl+D | 编辑选中 |
| Ctrl+E | 导出 .env |
| Ctrl+L | 操作日志 |
| Ctrl+F | 搜索 |
| F5 | 解锁 |
| Delete | 删除选中 |

## 安全等级

| 等级 | 模式 | 安全性 | 便利性 |
|------|------|--------|--------|
| 0 | 内存only | 最高 | 每次手动输入 |
| 1 | 持久化加密 | 中 | 自动加载 |
| 2 | 签名代理 | 中高 | 服务化注入 |

## 安全说明

- 保险箱密码请妥善保管，遗忘无法恢复
- `vault.enc` 为加密文件，不含明文密钥
- 请勿将保险箱文件上传到公开仓库
