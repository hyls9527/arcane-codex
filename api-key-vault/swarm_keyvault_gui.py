import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext, filedialog
import json
import os
import time
import threading
from pathlib import Path

sys_path = str(Path(__file__).parent)
if sys_path not in __import__("sys").path:
    __import__("sys").path.insert(0, sys_path)

from swarm_keyctl import KeyStore, VaultCrypto, VAULT_FILE, HAS_CRYPTO

AUTO_LOCK_TIMEOUT = 300


class PasswordDialog:
    def __init__(self, parent, title="密码", prompt="输入密码"):
        self.result = ""
        self.dialog = tk.Toplevel(parent)
        self.dialog.title(title)
        self.dialog.geometry("340x150")
        self.dialog.transient(parent)
        self.dialog.grab_set()

        ttk.Label(self.dialog, text=prompt).pack(pady=(15, 5))
        self.entry = ttk.Entry(self.dialog, show="●", width=40)
        self.entry.pack(pady=5, padx=15)
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
        self.dialog.geometry("380x200")
        self.dialog.transient(parent)
        self.dialog.grab_set()

        frame = ttk.Frame(self.dialog, padding=15)
        frame.pack(fill=tk.BOTH, expand=True)

        ttk.Label(frame, text="设置保险箱密码:").pack(anchor=tk.W, pady=(0, 2))
        self.entry1 = ttk.Entry(frame, show="●", width=45)
        self.entry1.pack(fill=tk.X, pady=(0, 8))

        ttk.Label(frame, text="确认密码:").pack(anchor=tk.W, pady=(0, 2))
        self.entry2 = ttk.Entry(frame, show="●", width=45)
        self.entry2.pack(fill=tk.X, pady=(0, 8))

        self.strength_lbl = ttk.Label(frame, text="", font=("Microsoft YaHei", 8))
        self.strength_lbl.pack(anchor=tk.W)

        self.entry1.bind("<KeyRelease>", lambda e: self._update_strength())

        btn_frame = ttk.Frame(frame)
        btn_frame.pack(fill=tk.X, pady=10)
        ttk.Button(btn_frame, text="取消", command=self.dialog.destroy).pack(side=tk.RIGHT, padx=5)
        ttk.Button(btn_frame, text="创建", command=self._ok).pack(side=tk.RIGHT, padx=5)

        self.entry1.focus()
        self.dialog.bind("<Return>", lambda e: self._ok())
        self.dialog.bind("<Escape>", lambda e: self.dialog.destroy())
        self.dialog.wait_window()

    def _update_strength(self):
        pwd = self.entry1.get()
        score = 0
        if len(pwd) >= 8: score += 1
        if len(pwd) >= 12: score += 1
        if any(c.isupper() for c in pwd): score += 1
        if any(c.isdigit() for c in pwd): score += 1
        if any(not c.isalnum() for c in pwd): score += 1
        labels = {0: ("极弱", "#EF4444"), 1: ("弱", "#EF4444"), 2: ("一般", "#F59E0B"),
                  3: ("良好", "#10B981"), 4: ("强", "#059669"), 5: ("极强", "#059669")}
        label, color = labels.get(score, ("", "gray"))
        self.strength_lbl.config(text=f"强度: {label}", foreground=color)

    def _ok(self):
        p1, p2 = self.entry1.get(), self.entry2.get()
        if not p1:
            messagebox.showwarning("警告", "密码不能为空", parent=self.dialog)
            return
        if p1 != p2:
            messagebox.showerror("错误", "两次密码不一致", parent=self.dialog)
            return
        if len(p1) < 8:
            messagebox.showwarning("警告", "密码至少8位", parent=self.dialog)
            return
        self.password = p1
        self.dialog.destroy()


class EditKeyDialog:
    def __init__(self, parent, title="添加密钥", name="", value="", tags="", url="", notes=""):
        self.result = None
        self.dialog = tk.Toplevel(parent)
        self.dialog.title(title)
        self.dialog.geometry("440x320")
        self.dialog.transient(parent)
        self.dialog.grab_set()

        frame = ttk.Frame(self.dialog, padding=15)
        frame.pack(fill=tk.BOTH, expand=True)

        ttk.Label(frame, text="名称:").pack(anchor=tk.W, pady=(0, 2))
        self.name_entry = ttk.Entry(frame, width=50)
        self.name_entry.pack(fill=tk.X, pady=(0, 6))
        self.name_entry.insert(0, name)

        ttk.Label(frame, text="值:").pack(anchor=tk.W, pady=(0, 2))
        self.value_entry = ttk.Entry(frame, width=50, show="●")
        self.value_entry.pack(fill=tk.X, pady=(0, 6))
        self.value_entry.insert(0, value)

        show_var = tk.BooleanVar()
        ttk.Checkbutton(frame, text="显示值", variable=show_var,
                         command=lambda: self.value_entry.config(show="" if show_var.get() else "●")).pack(anchor=tk.W, pady=(0, 6))

        ttk.Label(frame, text="标签 (逗号分隔):").pack(anchor=tk.W, pady=(0, 2))
        self.tags_entry = ttk.Entry(frame, width=50)
        self.tags_entry.pack(fill=tk.X, pady=(0, 6))
        self.tags_entry.insert(0, tags)

        ttk.Label(frame, text="URL:").pack(anchor=tk.W, pady=(0, 2))
        self.url_entry = ttk.Entry(frame, width=50)
        self.url_entry.pack(fill=tk.X, pady=(0, 6))
        self.url_entry.insert(0, url)

        ttk.Label(frame, text="备注:").pack(anchor=tk.W, pady=(0, 2))
        self.notes_entry = ttk.Entry(frame, width=50)
        self.notes_entry.pack(fill=tk.X, pady=(0, 6))
        self.notes_entry.insert(0, notes)

        btn_frame = ttk.Frame(frame)
        btn_frame.pack(fill=tk.X, pady=6)
        ttk.Button(btn_frame, text="取消", command=self.dialog.destroy).pack(side=tk.RIGHT, padx=5)
        ttk.Button(btn_frame, text="保存", command=self._ok).pack(side=tk.RIGHT, padx=5)

        self.dialog.bind("<Return>", lambda e: self._ok())
        self.dialog.bind("<Escape>", lambda e: self.dialog.destroy())
        self.dialog.wait_window()

    def _ok(self):
        name = self.name_entry.get().strip()
        value = self.value_entry.get().strip()
        if not name or not value:
            messagebox.showwarning("警告", "名称和值不能为空", parent=self.dialog)
            return
        self.result = {
            "name": name, "value": value,
            "tags": self.tags_entry.get().strip(),
            "url": self.url_entry.get().strip(),
            "notes": self.notes_entry.get().strip(),
        }
        self.dialog.destroy()


class SwarmKeyVaultGUI:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("Swarm Key Vault - 密钥保险箱")
        self.root.geometry("850x580")
        self.root.minsize(650, 420)

        if not HAS_CRYPTO:
            messagebox.showerror("错误", "缺少依赖。\n请安装: pip install cryptography")
            self.root.destroy()
            return

        self.store = KeyStore()
        self.last_activity = time.time()
        self.search_text = ""
        self._build_ui()
        self._setup_menu()
        self._setup_bindings()
        self._check_vault()
        self._start_timer()

    def _build_ui(self):
        status_frame = ttk.Frame(self.root, padding=(10, 8, 10, 4))
        status_frame.pack(fill=tk.X)

        self.status_lbl = ttk.Label(status_frame, text="已锁定", font=("Microsoft YaHei", 9, "bold"), foreground="gray")
        self.status_lbl.pack(side=tk.LEFT)

        self.path_lbl = ttk.Label(status_frame, text="", font=("Microsoft YaHei", 8), foreground="#6B7280")
        self.path_lbl.pack(side=tk.LEFT, padx=(10, 0))

        self.timer_lbl = ttk.Label(status_frame, text="", font=("Microsoft YaHei", 8), foreground="#9CA3AF")
        self.timer_lbl.pack(side=tk.RIGHT)

        search_frame = ttk.Frame(self.root, padding=(10, 0, 10, 4))
        search_frame.pack(fill=tk.X)

        ttk.Label(search_frame, text="搜索:").pack(side=tk.LEFT, padx=(0, 5))
        self.search_var = tk.StringVar()
        self.search_entry = ttk.Entry(search_frame, textvariable=self.search_var, width=30)
        self.search_entry.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 5))
        ttk.Button(search_frame, text="清空", width=4, command=lambda: self.search_var.set("")).pack(side=tk.LEFT)
        self.search_var.trace_add("write", lambda *a: self._do_search())

        tree_frame = ttk.Frame(self.root)
        tree_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=(0, 8))

        columns = ("name", "status", "tags", "created", "rotated")
        self.tree = ttk.Treeview(tree_frame, columns=columns, show="headings")
        self.tree.heading("name", text="名称", anchor=tk.W)
        self.tree.heading("status", text="状态", anchor=tk.W)
        self.tree.heading("tags", text="标签", anchor=tk.W)
        self.tree.heading("created", text="创建时间", anchor=tk.W)
        self.tree.heading("rotated", text="上次轮换", anchor=tk.W)
        self.tree.column("name", width=200, minwidth=120, stretch=True)
        self.tree.column("status", width=60, minwidth=50, stretch=False)
        self.tree.column("tags", width=120, minwidth=80, stretch=True)
        self.tree.column("created", width=110, minwidth=90, stretch=False)
        self.tree.column("rotated", width=110, minwidth=90, stretch=False)
        self.tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

        style = ttk.Style()
        style.configure("Treeview", rowheight=26)

        self.tree.bind("<Double-1>", self._show_value)

        scrollbar = ttk.Scrollbar(tree_frame, orient=tk.VERTICAL, command=self.tree.yview)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        self.tree.configure(yscrollcommand=scrollbar.set)

        btn_frame = ttk.Frame(self.root, padding=(10, 0, 10, 10))
        btn_frame.pack(fill=tk.X, side=tk.BOTTOM)

        ttk.Button(btn_frame, text="复制", command=self._copy_value, width=8).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_frame, text="编辑", command=self._edit_key, width=8).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_frame, text="删除", command=self._delete_key, width=8).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_frame, text="轮换", command=self._rotate_key, width=8).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_frame, text="吊销", command=self._revoke_key, width=8).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_frame, text="代理", command=self._start_proxy, width=8).pack(side=tk.RIGHT, padx=2)
        ttk.Button(btn_frame, text="导出", command=self._export_env, width=8).pack(side=tk.RIGHT, padx=2)
        ttk.Button(btn_frame, text="添加", command=self._add_key, width=8).pack(side=tk.RIGHT, padx=2)
        ttk.Button(btn_frame, text="锁定", command=self._lock, width=8).pack(side=tk.RIGHT, padx=2)
        ttk.Button(btn_frame, text="解锁", command=self._unlock, width=8).pack(side=tk.RIGHT, padx=2)

    def _setup_menu(self):
        menubar = tk.Menu(self.root)
        self.root.config(menu=menubar)

        file_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="文件", menu=file_menu)
        file_menu.add_command(label="创建保险箱...", command=self._create_vault, accelerator="Ctrl+N")
        file_menu.add_separator()
        file_menu.add_command(label="从 .env 导入...", command=self._import_env)
        file_menu.add_command(label="导出 .env", command=self._export_env, accelerator="Ctrl+E")
        file_menu.add_separator()
        file_menu.add_command(label="退出", command=self.root.quit)

        edit_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="编辑", menu=edit_menu)
        edit_menu.add_command(label="添加密钥", command=self._add_key, accelerator="Ctrl+K")
        edit_menu.add_command(label="编辑选中", command=self._edit_key, accelerator="Ctrl+D")
        edit_menu.add_command(label="删除选中", command=self._delete_key, accelerator="Delete")

        view_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="查看", menu=view_menu)
        view_menu.add_command(label="操作日志", command=self._show_log, accelerator="Ctrl+L")
        view_menu.add_command(label="锁定", command=self._lock)

    def _setup_bindings(self):
        self.root.bind("<Control-n>", lambda e: self._create_vault())
        self.root.bind("<Control-e>", lambda e: self._export_env())
        self.root.bind("<Control-k>", lambda e: self._add_key())
        self.root.bind("<Control-d>", lambda e: self._edit_key())
        self.root.bind("<Control-l>", lambda e: self._show_log())
        self.root.bind("<Control-f>", lambda e: self.search_entry.focus())
        self.root.bind("<F5>", lambda e: self._unlock())

    def _start_timer(self):
        def tick():
            if self.store.is_loaded():
                elapsed = int(time.time() - self.last_activity)
                remaining = max(0, AUTO_LOCK_TIMEOUT - elapsed)
                if remaining > 0:
                    self.timer_lbl.config(text=f"自动锁定 {remaining}s")
                else:
                    self._lock()
                    self.timer_lbl.config(text="")
            self.root.after(1000, tick)
        tick()

    def _reset_timer(self):
        self.last_activity = time.time()

    def _check_vault(self):
        if VAULT_FILE.exists():
            self.status_lbl.config(text="已锁定 - 点击解锁", foreground="#F59E0B")
            self.path_lbl.config(text=str(VAULT_FILE))
        else:
            self.status_lbl.config(text="未创建 - 新建保险箱", foreground="gray")
            self.path_lbl.config(text="")

    def _do_search(self):
        self.search_text = self.search_var.get().lower()
        self._refresh_tree()

    def _refresh_tree(self):
        for item in self.tree.get_children():
            self.tree.delete(item)
        if not self.store.is_loaded():
            return
        keys = self.store.list_keys(search=self.search_text if self.search_text else None)
        for name, meta in keys.items():
            status = "❌吊销" if meta.get("revoked") else "✅有效"
            tags = ",".join(meta.get("tags", [])[:3])
            created = meta.get("created", "")[:10]
            rotated = meta.get("rotated", "")[:10] if meta.get("rotated") else "从未"
            self.tree.insert("", tk.END, values=(name, status, tags, created, rotated))

    def _unlock(self):
        if not VAULT_FILE.exists():
            messagebox.showwarning("无保险箱", "请新建或打开保险箱", parent=self.root)
            return
        dialog = PasswordDialog(self.root, "解锁", "输入保险箱密码")
        if not dialog.result:
            return
        try:
            self.store.load(dialog.result)
            self.last_activity = time.time()
            self._refresh_tree()
            count = len(self.store.list_keys())
            self.status_lbl.config(text=f"已解锁 - {count} 个密钥", foreground="#10B981")
        except Exception as e:
            messagebox.showerror("解锁失败", str(e), parent=self.root)

    def _lock(self):
        self.store.lock()
        self._refresh_tree()
        self.status_lbl.config(text="已锁定", foreground="gray")
        self.timer_lbl.config(text="")

    def _create_vault(self):
        dialog = CreateVaultDialog(self.root)
        if dialog.password:
            try:
                self.store.load(dialog.password)
                self.store.save()
                self._refresh_tree()
                self.status_lbl.config(text="已解锁 - 新保险箱", foreground="#10B981")
                self.path_lbl.config(text=str(VAULT_FILE))
            except Exception as e:
                messagebox.showerror("错误", str(e), parent=self.root)

    def _add_key(self):
        if not self.store.is_loaded():
            messagebox.showwarning("已锁定", "请先解锁", parent=self.root)
            return
        dialog = EditKeyDialog(self.root, "添加密钥")
        if dialog.result:
            r = dialog.result
            self.store.set(r["name"], r["value"], tags=r["tags"].split(",") if r["tags"] else [],
                           url=r["url"], notes=r["notes"])
            self.store.save()
            self._refresh_tree()
            self._reset_timer()

    def _edit_key(self):
        if not self.store.is_loaded():
            return
        sel = self.tree.selection()
        if not sel:
            messagebox.showinfo("提示", "请选择要编辑的密钥", parent=self.root)
            return
        name = self.tree.item(sel[0])["values"][0]
        meta = self.store.get(name)
        if not meta:
            return
        dialog = EditKeyDialog(self.root, "编辑密钥", name=name, value=meta["value"],
                                tags=",".join(meta.get("tags", [])), url=meta.get("url", ""),
                                notes=meta.get("notes", ""))
        if dialog.result:
            r = dialog.result
            self.store.delete(name)
            self.store.set(r["name"], r["value"], tags=r["tags"].split(",") if r["tags"] else [],
                           url=r["url"], notes=r["notes"])
            self.store.save()
            self._refresh_tree()
            self._reset_timer()

    def _show_value(self, event):
        if not self.store.is_loaded():
            return
        sel = self.tree.selection()
        if not sel:
            return
        name = self.tree.item(sel[0])["values"][0]
        meta = self.store.get(name)
        if not meta:
            return
        dialog = tk.Toplevel(self.root)
        dialog.title(name)
        dialog.geometry("550x250")
        dialog.transient(self.root)

        text = scrolledtext.ScrolledText(dialog, wrap=tk.WORD, font=("Consolas", 10))
        text.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        text.insert("1.0", meta["value"])
        text.config(state=tk.DISABLED)

        btn_frame = ttk.Frame(dialog)
        btn_frame.pack(pady=5)
        ttk.Button(btn_frame, text="复制", command=lambda: self._copy_text(meta["value"])).pack(side=tk.LEFT, padx=5)
        ttk.Button(btn_frame, text="关闭", command=dialog.destroy).pack(side=tk.LEFT, padx=5)

    def _copy_value(self):
        if not self.store.is_loaded():
            return
        sel = self.tree.selection()
        if not sel:
            messagebox.showinfo("提示", "请先选中一个密钥", parent=self.root)
            return
        name = self.tree.item(sel[0])["values"][0]
        meta = self.store.get(name)
        if meta:
            self._copy_text(meta["value"])

    def _copy_text(self, text):
        self.root.clipboard_clear()
        self.root.clipboard_append(text)
        self.root.update()
        self._reset_timer()

    def _delete_key(self):
        if not self.store.is_loaded():
            return
        sel = self.tree.selection()
        if not sel:
            return
        name = self.tree.item(sel[0])["values"][0]
        if messagebox.askyesno("确认", f"确定删除 '{name}'?", parent=self.root):
            self.store.delete(name)
            self.store.save()
            self._refresh_tree()
            self._reset_timer()

    def _rotate_key(self):
        if not self.store.is_loaded():
            return
        sel = self.tree.selection()
        if not sel:
            messagebox.showinfo("提示", "请选择要轮换的密钥", parent=self.root)
            return
        name = self.tree.item(sel[0])["values"][0]
        dialog = PasswordDialog(self.root, f"轮换 {name}", "输入新值")
        if dialog.result:
            try:
                self.store.rotate(name, dialog.result)
                self.store.save()
                self._refresh_tree()
                self._reset_timer()
                messagebox.showinfo("成功", f"已轮换: {name}", parent=self.root)
            except Exception as e:
                messagebox.showerror("错误", str(e), parent=self.root)

    def _revoke_key(self):
        if not self.store.is_loaded():
            return
        sel = self.tree.selection()
        if not sel:
            return
        name = self.tree.item(sel[0])["values"][0]
        if messagebox.askyesno("确认", f"确定吊销 '{name}'?", parent=self.root):
            self.store.revoke(name)
            self.store.save()
            self._refresh_tree()
            self._reset_timer()

    def _export_env(self):
        if not self.store.is_loaded():
            messagebox.showwarning("已锁定", "请先解锁", parent=self.root)
            return
        path = filedialog.asksaveasfilename(title="导出 .env", defaultextension=".env",
                                             filetypes=[("Env 文件", "*.env"), ("所有文件", "*.*")])
        if path:
            try:
                self.store.export_env(path)
                messagebox.showinfo("成功", f"已导出到:\n{path}", parent=self.root)
            except Exception as e:
                messagebox.showerror("错误", str(e), parent=self.root)

    def _import_env(self):
        if not self.store.is_loaded():
            messagebox.showwarning("已锁定", "请先解锁", parent=self.root)
            return
        path = filedialog.askopenfilename(title="导入 .env",
                                            filetypes=[("Env 文件", "*.env"), ("所有文件", "*.*")])
        if path:
            try:
                count = self.store.import_env(path)
                self.store.save()
                self._refresh_tree()
                messagebox.showinfo("成功", f"已导入 {count} 个密钥", parent=self.root)
            except Exception as e:
                messagebox.showerror("错误", str(e), parent=self.root)

    def _start_proxy(self):
        if not self.store.is_loaded():
            messagebox.showwarning("已锁定", "请先解锁", parent=self.root)
            return
        try:
            from swarm_keyctl import cmd_proxy
            import argparse
            threading.Thread(target=lambda: cmd_proxy(argparse.Namespace(port=18239)), daemon=True).start()
            messagebox.showinfo("代理已启动",
                                "签名代理运行在:\nhttp://127.0.0.1:18239\n\n用法:\ncurl http://127.0.0.1:18239/?key=KEY_NAME",
                                parent=self.root)
        except Exception as e:
            messagebox.showerror("错误", str(e), parent=self.root)

    def _show_log(self):
        if not self.store.is_loaded():
            messagebox.showwarning("已锁定", "请先解锁", parent=self.root)
            return
        entries = self.store.get_log(limit=50)
        dialog = tk.Toplevel(self.root)
        dialog.title("操作日志")
        dialog.geometry("600x400")
        dialog.transient(self.root)

        text = scrolledtext.ScrolledText(dialog, wrap=tk.WORD, font=("Consolas", 9))
        text.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        for e in entries:
            text.insert(tk.END, f"  {e['time']}  {e['action']:<15} {e['detail']}\n")
        text.config(state=tk.DISABLED)

    def run(self):
        self.root.mainloop()


if __name__ == "__main__":
    app = SwarmKeyVaultGUI()
    app.run()
