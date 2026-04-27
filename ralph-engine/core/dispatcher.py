"""
Task Dispatcher - 任务调度器

核心职责：根据实例能力和负载，将原子任务分配给合适的实例。
"""

import json
import os
from typing import List, Dict, Optional
from datetime import datetime
from dataclasses import dataclass, field, asdict

try:
    from .intent import AtomicTask
except ImportError:
    from intent import AtomicTask


@dataclass
class InstanceCapability:
    """实例能力"""
    instance_id: str
    capabilities: List[str] = field(default_factory=list)  # ["frontend", "backend", "testing"]
    current_load: int = 0  # 当前任务数
    max_concurrent: int = 3  # 最大并发任务数
    status: str = "idle"  # idle, busy, error
    last_active: str = field(default_factory=lambda: datetime.now().isoformat())


@dataclass
class TaskAssignment:
    """任务分配记录"""
    task_id: str
    instance_id: str
    assigned_at: str = field(default_factory=lambda: datetime.now().isoformat())
    status: str = "assigned"  # assigned, executing, completed, failed


class TaskDispatcher:
    """
    任务调度器

    核心策略：
    1. 能力匹配 - 优先分配给有对应能力的实例
    2. 负载均衡 - 优先分配给负载低的实例
    3. 依赖排序 - 确保依赖任务先执行
    """

    def __init__(self, state_file: str = ""):
        self.state_file = state_file or os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "..", ".trae-instances", "dispatcher_state.json"
        )
        self.instances: Dict[str, InstanceCapability] = {}
        self.assignments: List[TaskAssignment] = []
        self._load_state()

    def register_instance(self, instance_id: str, capabilities: List[str], max_concurrent: int = 3):
        """注册实例"""
        self.instances[instance_id] = InstanceCapability(
            instance_id=instance_id,
            capabilities=capabilities,
            max_concurrent=max_concurrent
        )
        self._save_state()

    def dispatch(self, tasks: List[AtomicTask]) -> List[TaskAssignment]:
        """
        调度任务到实例

        参数:
            tasks: 待调度的原子任务列表

        返回:
            任务分配记录列表
        """
        assignments = []

        # 按依赖关系排序
        sorted_tasks = self._sort_by_dependencies(tasks)

        for task in sorted_tasks:
            # 选择最佳实例
            best_instance = self._select_instance(task)

            if best_instance:
                assignment = TaskAssignment(
                    task_id=task.id,
                    instance_id=best_instance.instance_id
                )
                self.assignments.append(assignment)
                assignments.append(assignment)

                # 更新实例负载
                best_instance.current_load += 1
                best_instance.status = "busy" if best_instance.current_load >= best_instance.max_concurrent else "idle"
                best_instance.last_active = datetime.now().isoformat()

                # 更新任务状态
                task.status = "dispatched"
                task.target_instance = best_instance.instance_id

        self._save_state()
        return assignments

    def _select_instance(self, task: AtomicTask) -> Optional[InstanceCapability]:
        """选择最佳实例"""
        candidates = []

        for inst in self.instances.values():
            # 跳过已达最大负载的实例
            if inst.current_load >= inst.max_concurrent:
                continue

            # 跳过错误状态的实例
            if inst.status == "error":
                continue

            # 计算匹配分数
            score = self._calculate_match_score(inst, task)
            if score > 0:
                candidates.append((score, inst))

        if not candidates:
            return None

        # 选择分数最高的，如果分数相同选择负载最低的
        candidates.sort(key=lambda x: (-x[0], x[1].current_load))
        return candidates[0][1]

    def _calculate_match_score(self, instance: InstanceCapability, task: AtomicTask) -> int:
        """计算实例与任务的匹配分数"""
        score = 0

        # 能力匹配 (最高优先级)
        task_type = task.type.lower()
        for cap in instance.capabilities:
            if cap.lower() in task_type or task_type in cap.lower():
                score += 100

        # 负载越低分数越高
        load_ratio = instance.current_load / instance.max_concurrent
        score += int((1 - load_ratio) * 50)

        # 最近活跃的实例有轻微加分
        try:
            last_active = datetime.fromisoformat(instance.last_active)
            minutes_since = (datetime.now() - last_active).total_seconds() / 60
            if minutes_since < 5:
                score += 10
        except:
            pass

        return score

    def _sort_by_dependencies(self, tasks: List[AtomicTask]) -> List[AtomicTask]:
        """按依赖关系排序任务"""
        # 简单的拓扑排序
        task_map = {t.id: t for t in tasks}
        sorted_tasks = []
        visited = set()

        def visit(task_id: str):
            if task_id in visited:
                return
            visited.add(task_id)

            task = task_map.get(task_id)
            if not task:
                return

            # 先访问依赖
            for dep_id in task.dependencies:
                visit(dep_id)

            sorted_tasks.append(task)

        for task in tasks:
            visit(task.id)

        return sorted_tasks

    def complete_assignment(self, task_id: str, success: bool = True, error: str = None):
        """标记任务完成"""
        for assignment in self.assignments:
            if assignment.task_id == task_id:
                assignment.status = "completed" if success else "failed"

                # 更新实例负载
                inst = self.instances.get(assignment.instance_id)
                if inst:
                    inst.current_load = max(0, inst.current_load - 1)
                    inst.status = "idle" if inst.current_load < inst.max_concurrent else "busy"
                    inst.last_active = datetime.now().isoformat()

        self._save_state()

    def get_instance_status(self) -> Dict:
        """获取所有实例状态"""
        return {
            inst_id: {
                "status": inst.status,
                "current_load": inst.current_load,
                "max_concurrent": inst.max_concurrent,
                "capabilities": inst.capabilities,
                "last_active": inst.last_active
            }
            for inst_id, inst in self.instances.items()
        }

    def _load_state(self):
        """从文件加载状态"""
        if os.path.exists(self.state_file):
            try:
                with open(self.state_file, 'r', encoding='utf-8') as f:
                    state = json.load(f)
                    # 恢复实例
                    for inst_id, inst_data in state.get("instances", {}).items():
                        self.instances[inst_id] = InstanceCapability(**inst_data)
                    # 恢复分配记录
                    for assign_data in state.get("assignments", []):
                        self.assignments.append(TaskAssignment(**assign_data))
            except:
                pass

    def _save_state(self):
        """保存状态到文件"""
        os.makedirs(os.path.dirname(self.state_file), exist_ok=True)
        state = {
            "instances": {k: asdict(v) for k, v in self.instances.items()},
            "assignments": [asdict(a) for a in self.assignments]
        }
        with open(self.state_file, 'w', encoding='utf-8') as f:
            json.dump(state, f, ensure_ascii=False, indent=2)
