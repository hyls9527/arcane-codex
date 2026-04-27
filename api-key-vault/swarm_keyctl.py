import os
import json
import time
import hashlib
import secrets
import argparse
import sys
import struct
import threading
import signal
from pathlib import Path
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

try:
    from cryptography.hazmat.primitives.ciphers.aead import AESGCM
    from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
    from cryptography.hazmat.primitives import hashes
    from cryptography.hazmat.backends import default_backend
    HAS_CRYPTO = True
except ImportError:
    HAS_CRYPTO = False

APP_DIR = Path(__file__).parent
VAULT_FILE = APP_DIR / "vault.enc"
BACKEND = default_backend()

MAGIC = b"SWKEY02"
VERSION = b"\x02"


class VaultCrypto:
    @staticmethod
    def derive_key(password, salt, iterations=600000):
        kdf = PBKDF2HMAC(
            algorithm=hashes.SHA256(), length=32, salt=salt,
            iterations=iterations, backend=BACKEND
        )
        return kdf.derive(password.encode("utf-8"))

    @staticmethod
    def encrypt(plaintext, password):
        salt = os.urandom(32)
        nonce = os.urandom(12)
        key = VaultCrypto.derive_key(password, salt)
        aesgcm = AESGCM(key)
        aad = MAGIC + VERSION
        ciphertext = aesgcm.encrypt(nonce, plaintext.encode("utf-8"), aad)
        return MAGIC + VERSION + struct.pack(">I", 600000) + salt + nonce + ciphertext

    @staticmethod
    def decrypt(data, password):
        if data[:7] != MAGIC:
            raise ValueError("文件格式无效")
        if data[7:8] != VERSION:
            raise ValueError("不支持的版本")
        iterations = struct.unpack(">I", data[8:12])[0]
        salt = data[12:44]
        nonce = data[44:56]
        ciphertext = data[56:]
        key = VaultCrypto.derive_key(password, salt, iterations)
        aesgcm = AESGCM(key)
        aad = MAGIC + VERSION
        plaintext = aesgcm.decrypt(nonce, ciphertext, aad)
        return plaintext.decode("utf-8")


class BruteForceGuard:
    def __init__(self):
        self._attempts = 0
        self._locked_until = 0
        self._lock = threading.Lock()

    def check(self):
        with self._lock:
            if time.time() < self._locked_until:
                remaining = int(self._locked_until - time.time())
                return False, f"尝试次数过多，请等待 {remaining} 秒"
            return True, ""

    def record_failure(self):
        with self._lock:
            self._attempts += 1
            delay = min(2 ** (self._attempts - 1), 300)
            self._locked_until = time.time() + delay

    def record_success(self):
        with self._lock:
            self._attempts = 0
            self._locked_until = 0


class KeyStore:
    def __init__(self):
        self._keys = {}
        self._loaded = False
        self._password = None
        self._bf_guard = BruteForceGuard()
        self._access_log = []

    def is_loaded(self):
        return self._loaded

    def load(self, password):
        allowed, msg = self._bf_guard.check()
        if not allowed:
            raise PermissionError(msg)
        try:
            if VAULT_FILE.exists():
                data = VAULT_FILE.read_bytes()
                json_str = VaultCrypto.decrypt(data, password)
                obj = json.loads(json_str)
                self._keys = obj.get("keys", {})
            else:
                self._keys = {}
            self._password = password
            self._loaded = True
            self._bf_guard.record_success()
            self._log("unlock", "解锁成功")
        except Exception as e:
            self._bf_guard.record_failure()
            self._log("unlock_failed", str(e))
            raise

    def save(self):
        if not self._loaded:
            raise RuntimeError("保险箱未解锁")
        obj = {"keys": self._keys, "version": 2}
        json_str = json.dumps(obj, ensure_ascii=False)
        encrypted = VaultCrypto.encrypt(json_str, self._password)
        tmp_path = VAULT_FILE.parent / f".vault_tmp_{os.getpid()}.enc"
        tmp_path.write_bytes(encrypted)
        if VAULT_FILE.exists():
            VAULT_FILE.unlink()
        tmp_path.rename(VAULT_FILE)
        self._log("save", "保存成功")

    def lock(self):
        self._keys = {}
        self._password = None
        self._loaded = False
        self._log("lock", "已锁定")

    def get(self, name):
        if not self._loaded:
            raise RuntimeError("保险箱未解锁")
        self._log("get", name)
        return self._keys.get(name)

    def set(self, name, value, tags=None, url=None, notes=None):
        if not self._loaded:
            raise RuntimeError("保险箱未解锁")
        self._keys[name] = {
            "value": value,
            "tags": tags or [],
            "url": url or "",
            "notes": notes or "",
            "created": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "rotated": None,
            "revoked": False,
        }
        self._log("set", name)

    def delete(self, name):
        if not self._loaded:
            raise RuntimeError("保险箱未解锁")
        if name in self._keys:
            del self._keys[name]
            self._log("delete", name)

    def list_keys(self, search=None):
        if not self._loaded:
            raise RuntimeError("保险箱未解锁")
        result = {}
        for name, meta in sorted(self._keys.items()):
            if search and search.lower() not in name.lower():
                if not any(search.lower() in t.lower() for t in meta.get("tags", [])):
                    continue
            result[name] = meta
        return result

    def verify(self, name):
        if not self._loaded:
            raise RuntimeError("保险箱未解锁")
        meta = self._keys.get(name)
        if not meta:
            return "missing"
        if meta.get("revoked"):
            return "revoked"
        value = meta.get("value", "")
        if value.startswith("sk-") and len(value) > 20:
            return "unverified"
        return "unverified"

    def rotate(self, name, new_value):
        if not self._loaded:
            raise RuntimeError("保险箱未解锁")
        if name not in self._keys:
            raise KeyError(f"密钥不存在: {name}")
        old = self._keys[name]
        self._keys[name] = {
            "value": new_value,
            "tags": old.get("tags", []),
            "url": old.get("url", ""),
            "notes": old.get("notes", ""),
            "created": old.get("created", ""),
            "rotated": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "revoked": False,
        }
        self._log("rotate", name)

    def revoke(self, name):
        if not self._loaded:
            raise RuntimeError("保险箱未解锁")
        if name in self._keys:
            self._keys[name]["revoked"] = True
            self._log("revoke", name)

    def export_env(self, path):
        if not self._loaded:
            raise RuntimeError("保险箱未解锁")
        lines = []
        for name, meta in sorted(self._keys.items()):
            if not meta.get("revoked"):
                lines.append(f"{name}={meta['value']}")
        Path(path).write_text("\n".join(lines) + "\n", encoding="utf-8")
        self._log("export", path)

    def import_env(self, path):
        if not self._loaded:
            raise RuntimeError("保险箱未解锁")
        imported = 0
        with open(path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                if "=" in line:
                    n, v = line.split("=", 1)
                    self.set(n.strip(), v.strip())
                    imported += 1
        self._log("import", f"{path} ({imported})")
        return imported

    def get_log(self, limit=20):
        return self._access_log[-limit:]

    def _log(self, action, detail):
        self._access_log.append({
            "time": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "action": action,
            "detail": detail,
        })


store = KeyStore()


def cmd_session(args):
    if args.action == "start":
        import getpass
        pwd = getpass.getpass("保险箱密码: ")
        store.load(pwd)
        if not VAULT_FILE.exists():
            store.save()
        print("✅ 保险箱已解锁")
        keys = store.list_keys()
        if keys:
            print(f"   已加载 {len(keys)} 个密钥")
        else:
            print("   保险箱为空，使用 set 添加密钥")

    elif args.action == "stop":
        store.lock()
        print("🔒 保险箱已锁定")

    elif args.action == "status":
        if store.is_loaded():
            keys = store.list_keys()
            print(f"✅ 已解锁 - {len(keys)} 个密钥")
        else:
            print("🔒 已锁定")


def cmd_set(args):
    _ensure_loaded()
    import getpass
    value = getpass.getpass(f"输入 {args.name} 的值: ")
    tags = args.tags.split(",") if args.tags else []
    store.set(args.name, value, tags=tags, url=args.url or "", notes=args.notes or "")
    store.save()
    print(f"✅ 已设置: {args.name}")


def cmd_get(args):
    _ensure_loaded()
    value = store.get(args.name)
    if value is None:
        print(f"❌ 未找到: {args.name}", file=sys.stderr)
        sys.exit(1)
    if args.show:
        print(value["value"])
    else:
        v = value["value"]
        print(f"{args.name} = {v[:8]}{'*' * (len(v) - 8) if len(v) > 8 else ''}")
        print(f"  创建: {value.get('created', '未知')}")
        print(f"  轮换: {value.get('rotated') or '从未'}")
        print(f"  状态: {'❌ 已吊销' if value.get('revoked') else '✅ 有效'}")
        if value.get("tags"):
            print(f"  标签: {', '.join(value['tags'])}")
        if value.get("url"):
            print(f"  URL:  {value['url']}")


def cmd_list(args):
    _ensure_loaded()
    keys = store.list_keys(search=args.search)
    if not keys:
        print("（空）")
        return
    fmt = "{:<35} {:<8} {:<12} {:<20}"
    print(fmt.format("名称", "状态", "标签", "创建时间"))
    print("-" * 75)
    for name, meta in keys.items():
        status = "❌吊销" if meta.get("revoked") else "✅有效"
        tags = ",".join(meta.get("tags", [])[:2])
        created = meta.get("created", "")[:10]
        print(fmt.format(name[:35], status, tags[:12], created))


def cmd_rotate(args):
    _ensure_loaded()
    import getpass
    new_value = getpass.getpass(f"输入 {args.name} 的新值: ")
    store.rotate(args.name, new_value)
    store.save()
    print(f"✅ 已轮换: {args.name}")


def cmd_revoke(args):
    _ensure_loaded()
    store.revoke(args.name)
    store.save()
    print(f"❌ 已吊销: {args.name}")


def cmd_delete(args):
    _ensure_loaded()
    store.delete(args.name)
    store.save()
    print(f"🗑️ 已删除: {args.name}")


def cmd_verify(args):
    _ensure_loaded()
    result = store.verify(args.name)
    status_map = {
        "unverified": "⚠️ 未验证",
        "revoked": "❌ 已吊销",
        "missing": "❌ 不存在",
    }
    print(f"{args.name}: {status_map.get(result, result)}")


def cmd_export(args):
    _ensure_loaded()
    store.export_env(args.path)
    print(f"✅ 已导出到: {args.path}")


def cmd_import(args):
    _ensure_loaded()
    count = store.import_env(args.path)
    store.save()
    print(f"✅ 已导入 {count} 个密钥")


def cmd_log(args):
    _ensure_loaded()
    entries = store.get_log(limit=args.limit or 20)
    if not entries:
        print("（无记录）")
        return
    for e in entries:
        print(f"  {e['time']}  {e['action']:<15} {e['detail']}")


def cmd_proxy(args):
    _ensure_loaded()
    port = args.port or 18239

    class ProxyHandler(BaseHTTPRequestHandler):
        def do_GET(self):
            self._handle()

        def do_POST(self):
            self._handle()

        def _handle(self):
            parsed = urlparse(self.path)
            params = parse_qs(parsed.query)
            name = params.get("key", [None])[0]
            if not name:
                self.send_error(400, "缺少 key 参数。用法: /?key=KEY_NAME")
                return
            meta = store.get(name)
            if not meta:
                self.send_error(404, f"密钥不存在: {name}")
                return
            if meta.get("revoked"):
                self.send_error(403, f"密钥已吊销: {name}")
                return
            resp = json.dumps({
                "name": name,
                "value": meta["value"],
                "tags": meta.get("tags", []),
                "url": meta.get("url", ""),
            }, ensure_ascii=False)
            self.send_response(200)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.end_headers()
            self.wfile.write(resp.encode("utf-8"))
            store._log("proxy_get", name)

        def log_message(self, fmt, *args):
            pass

    server = HTTPServer(("127.0.0.1", port), ProxyHandler)
    print(f"🔑 签名代理已启动: http://127.0.0.1:{port}")
    print(f"   用法: curl http://127.0.0.1:{port}/?key=OPENAI_API_KEY")
    print(f"   Ctrl+C 停止")

    def shutdown(sig, frame):
        server.shutdown()
        print("\n🔒 代理已停止")

    signal.signal(signal.SIGINT, shutdown)
    server.serve_forever()


def cmd_init(args):
    if VAULT_FILE.exists():
        print("保险箱已存在。如需重建请先删除 vault.enc")
        return
    import getpass
    pwd1 = getpass.getpass("设置保险箱密码: ")
    pwd2 = getpass.getpass("确认密码: ")
    if pwd1 != pwd2:
        print("❌ 两次密码不一致", file=sys.stderr)
        sys.exit(1)
    if len(pwd1) < 8:
        print("❌ 密码至少8位", file=sys.stderr)
        sys.exit(1)
    store.load(pwd1)
    store.save()
    print("✅ 保险箱已创建")


def _ensure_loaded():
    if not store.is_loaded():
        print("❌ 保险箱未解锁。先运行: swarm-keyctl session start", file=sys.stderr)
        sys.exit(1)


def main():
    if not HAS_CRYPTO:
        print("❌ 缺少依赖。请安装: pip install cryptography", file=sys.stderr)
        sys.exit(1)

    parser = argparse.ArgumentParser(
        prog="swarm-keyctl",
        description="Swarm Key Controller - API 密钥保险箱 CLI"
    )
    sub = parser.add_subparsers(dest="command")

    p_init = sub.add_parser("init", help="创建新保险箱")
    p_sess = sub.add_parser("session", help="会话管理")
    p_sess.add_argument("action", choices=["start", "stop", "status"])

    p_set = sub.add_parser("set", help="添加/更新密钥")
    p_set.add_argument("name", help="密钥名称")
    p_set.add_argument("--tags", help="标签，逗号分隔")
    p_set.add_argument("--url", help="服务URL")
    p_set.add_argument("--notes", help="备注")

    p_get = sub.add_parser("get", help="查看密钥")
    p_get.add_argument("name", help="密钥名称")
    p_get.add_argument("--show", action="store_true", help="显示完整值")

    p_list = sub.add_parser("list", help="列出密钥")
    p_list.add_argument("--search", help="搜索关键词")

    p_rotate = sub.add_parser("rotate", help="轮换密钥")
    p_rotate.add_argument("name", help="密钥名称")

    p_revoke = sub.add_parser("revoke", help="吊销密钥")
    p_revoke.add_argument("name", help="密钥名称")

    p_delete = sub.add_parser("delete", help="删除密钥")
    p_delete.add_argument("name", help="密钥名称")

    p_verify = sub.add_parser("verify", help="验证密钥状态")
    p_verify.add_argument("name", help="密钥名称")

    p_export = sub.add_parser("export", help="导出为 .env")
    p_export.add_argument("path", help="导出路径")

    p_import = sub.add_parser("import", help="从 .env 导入")
    p_import.add_argument("path", help=".env 文件路径")

    p_log = sub.add_parser("log", help="查看操作日志")
    p_log.add_argument("--limit", type=int, default=20)

    p_proxy = sub.add_parser("proxy", help="启动签名代理")
    p_proxy.add_argument("--port", type=int, default=18239)

    args = parser.parse_args()

    commands = {
        "init": cmd_init,
        "session": cmd_session,
        "set": cmd_set,
        "get": cmd_get,
        "list": cmd_list,
        "rotate": cmd_rotate,
        "revoke": cmd_revoke,
        "delete": cmd_delete,
        "verify": cmd_verify,
        "export": cmd_export,
        "import": cmd_import,
        "log": cmd_log,
        "proxy": cmd_proxy,
    }

    if args.command in commands:
        try:
            commands[args.command](args)
        except PermissionError as e:
            print(f"❌ {e}", file=sys.stderr)
            sys.exit(1)
        except Exception as e:
            print(f"❌ 错误: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
