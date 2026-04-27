import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext, filedialog
import json
import os
import sys
from pathlib import Path
import time

try:
    from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
    from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
    from cryptography.hazmat.primitives import hashes
    from cryptography.hazmat.backends import default_backend
    HAS_CRYPTO = True
except ImportError:
    HAS_CRYPTO = False

try:
    import hashlib
    HAS_HASHLIB = True
except ImportError:
    HAS_HASHLIB = False

APP_DIR = Path(__file__).parent
VAULT_FILE = APP_DIR / ".env.keys.json.enc"
BACKEND = default_backend()

AUTO_LOCK_TIMEOUT = 300


class SimpleVault:
    @staticmethod
    def encrypt(plaintext, password):
        salt = os.urandom(16)
        iv = os.urandom(16)
        kdf = PBKDF2HMAC(
            algorithm=hashes.SHA256(), length=32, salt=salt,
            iterations=100000, backend=BACKEND
        )
        key = kdf.derive(password.encode())
        cipher = Cipher(algorithms.AES(key), modes.CBC(iv), backend=BACKEND)
        encryptor = cipher.encryptor()
        padded = SimpleVault._pad(plaintext.encode())
        encrypted = encryptor.update(padded) + encryptor.finalize()
        return b"APIVAULT" + salt + iv + encrypted

    @staticmethod
    def decrypt(data, password):
        if data[:8] != b"APIVAULT":
            raise ValueError("文件格式无效")
        salt = data[8:24]
        iv = data[24:40]
        encrypted = data[40:]
        kdf = PBKDF2HMAC(
            algorithm=hashes.SHA256(), length=32, salt=salt,
            iterations=100000, backend=BACKEND
        )
        key = kdf.derive(password.encode())
        cipher = Cipher(algorithms.AES(key), modes.CBC(iv), backend=BACKEND)
        decryptor = cipher.decryptor()
        padded = decryptor.update(encrypted) + decryptor.finalize()
        return SimpleVault._unpad(padded).decode("utf-8")

    @staticmethod
    def _pad(data):
        bs = 16
        pad_len = bs - (len(data) % bs)
        return data + bytes([pad_len] * pad_len)

    @staticmethod
    def _unpad(data):
        pad_len = data[-1]
        if pad_len < 1 or pad_len > 16:
            raise ValueError("填充数据无效")
        return data[:-pad_len]


class FallbackVault:
    @staticmethod
    def encrypt(plaintext, password):
        salt = os.urandom(16)
        iv = os.urandom(16)
        key = hashlib.pbkdf2_hmac("sha256", password.encode(), salt, 100000, dklen=32)
        from Crypto.Cipher import AES
        cipher = AES.new(key, AES.MODE_CBC, iv)
        padded = FallbackVault._pad(plaintext.encode())
        encrypted = cipher.encrypt(padded)
        return b"APIVAULT" + salt + iv + encrypted

    @staticmethod
    def decrypt(data, password):
        if data[:8] != b"APIVAULT":
            raise ValueError("文件格式无效")
        salt = data[8:24]
        iv = data[24:40]
        encrypted = data[40:]
        key = hashlib.pbkdf2_hmac("sha256", password.encode(), salt, 100000, dklen=32)
        from Crypto.Cipher import AES
        cipher = AES.new(key, AES.MODE_CBC, iv)
        padded = cipher.decrypt(encrypted)
        return FallbackVault._unpad(padded).decode("utf-8")

    @staticmethod
    def _pad(data):
        bs = 16
        pad_len = bs - (len(data) % bs)
        return data + bytes([pad_len] * pad_len)

    @staticmethod
    def _unpad(data):
        pad_len = data[-1]
        if pad_len < 1 or pad_len > 16:
            raise ValueError("填充数据无效")
        return data[:-pad_len]


def get_vault_class():
    if HAS_CRYPTO:
        return SimpleVault
    try:
        from Crypto.Cipher import AES
        return FallbackVault
    except ImportError:
        return None


def check_password_strength(pwd):
    score = 0
    if len(pwd) >= 8:
        score += 1
    if len(pwd) >= 12:
        score += 1
    if any(c.isupper() for c in pwd):
        score += 1
    if any(c.isdigit() for c in pwd):
        score += 1
    if any(not c.isalnum() for c in pwd):
        score += 1
    if score <= 1:
        return "弱", "#EF4444"
    elif score <= 2:
        return "一般", "#F59E0B"
    elif score <= 3:
        return "良好", "#10B981"
    else:
        return "强", "#059669"


class EditKeyDialog:
    def __init__(self, parent, title, name="", value="", is_edit=False):
        self.result = None
        self.dialog = tk.Toplevel(parent)
        self.dialog.title(title)
        self.dialog.geometry("420x260")
        self.dialog.transient(parent)
        self.dialog.grab_set()

        frame = ttk.Frame(self.dialog, padding=15)
        frame.pack(fill=tk.BOTH, expand=True)

        ttk.Label(frame, text="名称:").pack(anchor=tk.W, pady=(0, 2))
        self.name_entry = ttk.Entry(frame, width=50)
        self.name_entry.pack(fill=tk.X, pady=(0, 8))
        self.name_entry.insert(0, name)

        ttk.Label(frame, text="值:").pack(anchor=tk.W, pady=(0, 2))
        self.value_entry = ttk.Entry(frame, width=50, show="*")
        self.value_entry.pack(fill=tk.X, pady=(0, 8))
        self.value_entry.insert(0, value)

        show_var = tk.BooleanVar()
        show_cb = ttk.Checkbutton(frame, text="显示值", variable=show_var,
                                   command=lambda: self.value_entry.config(show="" if show_var.get() else "*"))
        show_cb.pack(anchor=tk.W, pady=(0, 10))

        self.strength_lbl = ttk.Label(frame, text="", font=("Microsoft YaHei", 8))
        self.strength_lbl.pack(anchor=tk.W)

        def on_value_change(*args):
            label, color = check_password_strength(self.value_entry.get())
            self.strength_lbl.config(text="密码强度: " + label, foreground=color)

        self.value_entry.bind("<KeyRelease>", on_value_change)

        btn_frame = ttk.Frame(frame)
        btn_frame.pack(fill=tk.X, pady=10)

        ttk.Button(btn_frame, text="取消", command=self.dialog.destroy).pack(side=tk.RIGHT, padx=5)
        ttk.Button(btn_frame, text="保存", command=self._ok).pack(side=tk.RIGHT, padx=5)

        if not name:
            self.name_entry.focus()
        else:
            self.value_entry.focus()

        self.dialog.bind("<Return>", lambda e: self._ok())
        self.dialog.bind("<Escape>", lambda e: self.dialog.destroy())

        self.dialog.wait_window()

    def _ok(self):
        name = self.name_entry.get().strip()
        value = self.value_entry.get().strip()
        if not name or not value:
            messagebox.showwarning("警告", "名称和值不能为空", parent=self.dialog)
            return
        self.result = (name, value)
        self.dialog.destroy()


class PasswordDialog:
    def __init__(self, parent, prompt):
        self.result = ""
        self.dialog = tk.Toplevel(parent)
        self.dialog.title("密码")
        self.dialog.geometry("320x140")
        self.dialog.transient(parent)
        self.dialog.grab_set()

        ttk.Label(self.dialog, text=prompt).pack(pady=(15, 5))
        self.entry = ttk.Entry(self.dialog, show="*", width=35)
        self.entry.pack(pady=5)
        self.entry.focus()

        btn_frame = ttk.Frame(self.dialog)
        btn_frame.pack(pady=10)

        ttk.Button(btn_frame, text="取消", command=self.dialog.destroy).pack(side=tk.RIGHT, padx=5)
        ttk.Button(btn_frame, text="确定", command=self._ok).pack(side=tk.RIGHT, padx=5)

        self.dialog.bind("<Return>", lambda e: self._ok())
        self.dialog.bind("<Escape>", lambda e: self.dialog.destroy())

        self.dialog.wait_window()

    def _ok(self):
        self.result = self.entry.get()
        self.dialog.destroy()


class CreateVaultDialog:
    def __init__(self, parent):
        self.password = ""
        self.dialog = tk.Toplevel(parent)
        self.dialog.title("创建保险箱")
        self.dialog.geometry("380x240")
        self.dialog.transient(parent)
        self.dialog.grab_set()

        frame = ttk.Frame(self.dialog, padding=15)
        frame.pack(fill=tk.BOTH, expand=True)

        ttk.Label(frame, text="设置保险箱密码:").pack(anchor=tk.W, pady=(0, 2))
        self.entry1 = ttk.Entry(frame, show="*", width=45)
        self.entry1.pack(fill=tk.X, pady=(0, 8))

        ttk.Label(frame, text="确认密码:").pack(anchor=tk.W, pady=(0, 2))
        self.entry2 = ttk.Entry(frame, show="*", width=45)
        self.entry2.pack(fill=tk.X, pady=(0, 8))

        self.strength_lbl = ttk.Label(frame, text="", font=("Microsoft YaHei", 8))
        self.strength_lbl.pack(anchor=tk.W)

        def on_change(*args):
            label, color = check_password_strength(self.entry1.get())
            self.strength_lbl.config(text="强度: " + label, foreground=color)

        self.entry1.bind("<KeyRelease>", on_change)

        btn_frame = ttk.Frame(frame)
        btn_frame.pack(fill=tk.X, pady=10)

        ttk.Button(btn_frame, text="取消", command=self.dialog.destroy).pack(side=tk.RIGHT, padx=5)
        ttk.Button(btn_frame, text="创建", command=self._ok).pack(side=tk.RIGHT, padx=5)

        self.entry1.focus()
        self.dialog.bind("<Return>", lambda e: self._ok())
        self.dialog.bind("<Escape>", lambda e: self.dialog.destroy())

        self.dialog.wait_window()

    def _ok(self):
        p1 = self.entry1.get()
        p2 = self.entry2.get()
        if not p1:
            messagebox.showwarning("警告", "密码不能为空", parent=self.dialog)
            return
        if p1 != p2:
            messagebox.showerror("错误", "两次密码不一致", parent=self.dialog)
            return
        self.password = p1
        self.dialog.destroy()


class SearchBar(ttk.Frame):
    def __init__(self, parent, on_search):
        super().__init__(parent)
        self.on_search = on_search
        self.search_var = tk.StringVar()

        ttk.Label(self, text="搜索:").pack(side=tk.LEFT, padx=(0, 5))
        self.entry = ttk.Entry(self, textvariable=self.search_var, width=25)
        self.entry.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 5))

        clear_btn = ttk.Button(self, text="清空", width=4, command=self._clear)
        clear_btn.pack(side=tk.LEFT)

        self.search_var.trace_add("write", lambda *args: self.on_search(self.search_var.get()))
        self.entry.bind("<Escape>", lambda e: self._clear())

    def _clear(self):
        self.search_var.set("")
        self.entry.focus()


class APIVaultGUI:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("API 密钥保险箱")
        self.root.geometry("800x550")
        self.root.minsize(600, 400)
        self.vault_class = get_vault_class()
        self.keys = {}
        self.unlocked = False
        self.password = ""
        self.vault_path = VAULT_FILE
        self.last_activity = time.time()
        self.search_text = ""

        self._check_crypto()
        self._build_ui()
        self._setup_menu()
        self._setup_bindings()
        self._check_vault()

    def _check_crypto(self):
        if self.vault_class is None:
            messagebox.showerror("错误",
                "未找到加密库。\n请安装: pip install cryptography\n或: pip install pycryptodome")
            self.root.destroy()
            sys.exit(1)

    def _build_ui(self):
        self.root.geometry("800x580")

        status_frame = ttk.Frame(self.root, padding=(10, 10, 10, 5))
        status_frame.pack(fill=tk.X)

        self.status_lbl = ttk.Label(status_frame, text="已锁定", font=("Microsoft YaHei", 9, "bold"), foreground="gray")
        self.status_lbl.pack(side=tk.LEFT)

        self.path_lbl = ttk.Label(status_frame, text="", font=("Microsoft YaHei", 8), foreground="#6B7280")
        self.path_lbl.pack(side=tk.LEFT, padx=(10, 0))

        self.timer_lbl = ttk.Label(status_frame, text="", font=("Microsoft YaHei", 8), foreground="#9CA3AF")
        self.timer_lbl.pack(side=tk.RIGHT)

        self.search_bar = SearchBar(self.root, self._do_search)
        self.search_bar.pack(fill=tk.X, padx=10, pady=(0, 5))

        tree_frame = ttk.Frame(self.root)
        tree_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=(0, 8))

        columns = ("name", "value")
        self.tree = ttk.Treeview(tree_frame, columns=columns, show="headings")
        self.tree.heading("name", text="名称", anchor=tk.W)
        self.tree.heading("value", text="值", anchor=tk.W)
        self.tree.column("name", width=220, minwidth=120, stretch=True)
        self.tree.column("value", width=500, minwidth=200, stretch=True)
        self.tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

        style = ttk.Style()
        style.configure("Treeview", rowheight=28)

        self.tree.bind("<Double-1>", self._show_value)
        self.tree.bind("<Delete>", lambda e: self._delete_key())

        scrollbar = ttk.Scrollbar(tree_frame, orient=tk.VERTICAL, command=self.tree.yview)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        self.tree.configure(yscrollcommand=scrollbar.set)

        btn_frame = ttk.Frame(self.root, padding=(10, 0, 10, 10))
        btn_frame.pack(fill=tk.X, side=tk.BOTTOM)

        ttk.Button(btn_frame, text="复制", command=self._copy_value, width=10).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_frame, text="删除", command=self._delete_key, width=10).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_frame, text="导出 .env", command=self._export_env, width=12).pack(side=tk.RIGHT, padx=2)
        ttk.Button(btn_frame, text="保存保险箱", command=self._save_vault, width=12).pack(side=tk.RIGHT, padx=2)
        ttk.Button(btn_frame, text="添加密钥", command=self._add_key, width=10).pack(side=tk.RIGHT, padx=2)
        ttk.Button(btn_frame, text="锁定", command=self._lock, width=10).pack(side=tk.RIGHT, padx=2)
        self.unlock_btn = ttk.Button(btn_frame, text="解锁", command=self._unlock, width=10)
        self.unlock_btn.pack(side=tk.RIGHT, padx=2)
        self.lock_btn = btn_frame.winfo_children()[-2]
        self.add_btn = btn_frame.winfo_children()[-4]
        self.save_btn = btn_frame.winfo_children()[-5]

        self._update_button_states()
        self._start_timer()

    def _setup_menu(self):
        menubar = tk.Menu(self.root)
        self.root.config(menu=menubar)

        file_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="文件", menu=file_menu)
        file_menu.add_command(label="打开保险箱...", command=self._open_vault, accelerator="Ctrl+O")
        file_menu.add_command(label="新建保险箱...", command=self._create_vault, accelerator="Ctrl+N")
        file_menu.add_separator()
        file_menu.add_command(label="从 .env 导入...", command=self._import_env)
        file_menu.add_command(label="导出 .env", command=self._export_env, accelerator="Ctrl+E")
        file_menu.add_separator()
        file_menu.add_command(label="退出", command=self.root.quit, accelerator="Alt+F4")

        edit_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="编辑", menu=edit_menu)
        edit_menu.add_command(label="添加密钥", command=self._add_key, accelerator="Ctrl+K")
        edit_menu.add_command(label="编辑选中", command=self._edit_key, accelerator="Ctrl+D")
        edit_menu.add_command(label="删除选中", command=self._delete_key, accelerator="Delete")

        view_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="查看", menu=view_menu)
        view_menu.add_command(label="锁定", command=self._lock, accelerator="Ctrl+L")

    def _setup_bindings(self):
        self.root.bind("<Control-o>", lambda e: self._open_vault())
        self.root.bind("<Control-n>", lambda e: self._create_vault())
        self.root.bind("<Control-e>", lambda e: self._export_env())
        self.root.bind("<Control-k>", lambda e: self._add_key())
        self.root.bind("<Control-d>", lambda e: self._edit_key())
        self.root.bind("<Control-l>", lambda e: self._lock())
        self.root.bind("<Control-f>", lambda e: self.search_bar.entry.focus())
        self.root.bind("<F5>", lambda e: self._unlock())

    def _start_timer(self):
        def tick():
            if self.unlocked:
                elapsed = int(time.time() - self.last_activity)
                remaining = max(0, AUTO_LOCK_TIMEOUT - elapsed)
                if remaining > 0:
                    self.timer_lbl.config(text="自动锁定倒计时 {}s".format(remaining))
                else:
                    self._lock()
                    self.timer_lbl.config(text="")
            self.root.after(1000, tick)
        tick()

    def _reset_timer(self):
        self.last_activity = time.time()

    def _check_vault(self):
        if self.vault_path.exists():
            self.status_lbl.config(text="已锁定 - 点击解锁", foreground="#F59E0B")
            self.path_lbl.config(text=str(self.vault_path))
        else:
            self.status_lbl.config(text="未找到保险箱 - 新建或打开已有", foreground="gray")
            self.path_lbl.config(text="")

    def _update_button_states(self):
        state = tk.NORMAL if self.unlocked else tk.DISABLED
        self.add_btn.config(state=state)
        self.save_btn.config(state=state)
        self.lock_btn.config(state=state)
        self.unlock_btn.config(state=tk.DISABLED if self.unlocked else tk.NORMAL)

    def _do_search(self, text):
        self.search_text = text.lower()
        self._refresh_tree()

    def _open_vault(self):
        path = filedialog.askopenfilename(
            title="打开保险箱",
            filetypes=[("保险箱文件", "*.enc"), ("所有文件", "*.*")],
            initialdir=str(APP_DIR)
        )
        if path:
            self.vault_path = Path(path)
            self._check_vault()

    def _create_vault(self):
        dialog = CreateVaultDialog(self.root)
        if dialog.password:
            self.password = dialog.password
            self.keys = {}
            self.unlocked = True
            self.vault_path = VAULT_FILE
            self._save_vault(silent=True)
            self._refresh_tree()
            self.status_lbl.config(text="已解锁 - 新保险箱已创建", foreground="#10B981")
            self.path_lbl.config(text=str(self.vault_path))
            self._update_button_states()

    def _unlock(self):
        if not self.vault_path.exists():
            messagebox.showwarning("无保险箱", "未找到保险箱文件。\n请新建或打开一个已有的保险箱。",
                                   parent=self.root)
            return
        dialog = PasswordDialog(self.root, "输入保险箱密码")
        if not dialog.result:
            return
        try:
            data = self.vault_path.read_bytes()
            json_str = self.vault_class.decrypt(data, dialog.result)
            obj = json.loads(json_str)
            self.keys = obj.get("keys", {})
            self.unlocked = True
            self.password = dialog.result
            self.last_activity = time.time()
            self._refresh_tree()
            self.status_lbl.config(text="已解锁 - {} 个密钥".format(len(self.keys)), foreground="#10B981")
            self.path_lbl.config(text=str(self.vault_path))
            self._update_button_states()
        except Exception as e:
            messagebox.showerror("解锁失败", str(e), parent=self.root)

    def _lock(self):
        self.keys = {}
        self.unlocked = False
        self.password = ""
        self.search_text = ""
        self.search_bar.search_var.set("")
        self._refresh_tree()
        self.status_lbl.config(text="已锁定", foreground="gray")
        self.timer_lbl.config(text="")
        self._update_button_states()

    def _refresh_tree(self):
        for item in self.tree.get_children():
            self.tree.delete(item)
        for name in sorted(self.keys.keys()):
            if self.search_text and self.search_text not in name.lower():
                continue
            value = self.keys[name]
            display = value[:30] + "..." if len(value) > 30 else value
            self.tree.insert("", tk.END, values=(name, display))

    def _add_key(self):
        if not self.unlocked:
            return
        dialog = EditKeyDialog(self.root, "添加 API 密钥")
        if dialog.result:
            name, value = dialog.result
            self.keys[name] = value
            self._refresh_tree()
            self._reset_timer()

    def _edit_key(self):
        if not self.unlocked:
            return
        sel = self.tree.selection()
        if not sel:
            messagebox.showinfo("提示", "请选择要编辑的密钥", parent=self.root)
            return
        item = self.tree.item(sel[0])
        name = item["values"][0]
        value = self.keys.get(name, "")
        dialog = EditKeyDialog(self.root, "编辑密钥", name=name, value=value, is_edit=True)
        if dialog.result:
            new_name, new_value = dialog.result
            if new_name != name and new_name in self.keys:
                messagebox.showwarning("警告", "密钥已存在", parent=self.root)
                return
            if new_name != name:
                del self.keys[name]
            self.keys[new_name] = new_value
            self._refresh_tree()
            self._reset_timer()

    def _show_value(self, event):
        if not self.unlocked:
            return
        sel = self.tree.selection()
        if not sel:
            return
        item = self.tree.item(sel[0])
        name = item["values"][0]
        value = self.keys.get(name, "")
        dialog = tk.Toplevel(self.root)
        dialog.title(name)
        dialog.geometry("550x250")
        dialog.transient(self.root)

        text = scrolledtext.ScrolledText(dialog, wrap=tk.WORD, font=("Consolas", 10))
        text.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        text.insert("1.0", value)
        text.config(state=tk.DISABLED)

        btn_frame = ttk.Frame(dialog)
        btn_frame.pack(pady=5)
        ttk.Button(btn_frame, text="复制", command=lambda: self._copy_text(value)).pack(side=tk.LEFT, padx=5)
        ttk.Button(btn_frame, text="关闭", command=dialog.destroy).pack(side=tk.LEFT, padx=5)

    def _copy_value(self):
        if not self.unlocked:
            return
        sel = self.tree.selection()
        if not sel:
            messagebox.showinfo("提示", "请先选中一个密钥", parent=self.root)
            return
        item = self.tree.item(sel[0])
        name = item["values"][0]
        value = self.keys.get(name, "")
        self._copy_text(value)

    def _copy_text(self, text):
        self.root.clipboard_clear()
        self.root.clipboard_append(text)
        self.root.update()

    def _delete_key(self):
        if not self.unlocked:
            return
        sel = self.tree.selection()
        if not sel:
            return
        item = self.tree.item(sel[0])
        name = item["values"][0]
        if messagebox.askyesno("确认", "确定删除 '{}'?".format(name), parent=self.root):
            del self.keys[name]
            self._refresh_tree()
            self._reset_timer()

    def _save_vault(self, silent=False):
        if not self.unlocked:
            return
        try:
            obj = {"keys": self.keys}
            json_str = json.dumps(obj, indent=2, ensure_ascii=False)
            encrypted = self.vault_class.encrypt(json_str, self.password)
            self.vault_path.write_bytes(encrypted)
            if not silent:
                messagebox.showinfo("保存成功", "保险箱已保存到:\n" + str(self.vault_path), parent=self.root)
            self.status_lbl.config(text="已解锁 - {} 个密钥 (已保存)".format(len(self.keys)), foreground="#10B981")
        except Exception as e:
            messagebox.showerror("保存失败", str(e), parent=self.root)

    def _export_env(self):
        if not self.unlocked:
            messagebox.showwarning("已锁定", "请先解锁保险箱", parent=self.root)
            return
        path = filedialog.asksaveasfilename(
            title="导出 .env",
            defaultextension=".env",
            filetypes=[("Env 文件", "*.env"), ("所有文件", "*.*")],
            initialdir=str(self.vault_path.parent)
        )
        if path:
            lines = []
            for name, value in sorted(self.keys.items()):
                lines.append(name + "=" + value)
            with open(path, "w", encoding="utf-8") as f:
                f.write("\n".join(lines) + "\n")
            messagebox.showinfo("导出成功", "已导出到:\n" + path, parent=self.root)

    def _import_env(self):
        path = filedialog.askopenfilename(
            title="导入 .env",
            filetypes=[("Env 文件", "*.env"), ("所有文件", "*.*")],
            initialdir=str(APP_DIR)
        )
        if not path:
            return
        if not self.unlocked:
            messagebox.showwarning("已锁定", "请先解锁保险箱", parent=self.root)
            return
        imported = 0
        with open(path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                if "=" in line:
                    name, value = line.split("=", 1)
                    self.keys[name.strip()] = value.strip()
                    imported += 1
        self._refresh_tree()
        messagebox.showinfo("导入成功", "已导入 {} 个密钥".format(imported), parent=self.root)
        self._reset_timer()

    def run(self):
        self.root.mainloop()


if __name__ == "__main__":
    app = APIVaultGUI()
    app.run()
