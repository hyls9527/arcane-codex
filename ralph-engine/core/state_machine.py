"""
State Machine Manager - 状态机管理器

核心职责：管理任务生命周期的状态转换，确保状态一致性。
"""

import json
import os
from typing import Dict, List, Optional
from datetime import datetime
from enum import Enum
from dataclasses import dataclass, field, asdict


class TaskState(Enum):
    """任务状态"""
    IDLE = "idle"
    PLANNING = "planning"
    DISPATCHED = "dispatched"
    EXECUTING = "executing"
    VERIFYING = "verifying"
    COMPLETED = "completed"
    FAILED = "failed"


class StateTransition(Enum):
    """允许的状态转换"""
    IDLE_TO_PLANNING = ("idle", "planning")
    PLANNING_TO_DISPATCHED = ("planning", "dispatched")
    DISPATCHED_TO_EXECUTING = ("dispatched", "executing")
    EXECUTING_TO_VERIFYING = ("executing", "verifying")
    VERIFYING_TO_COMPLETED = ("verifying", "completed")
    VERIFYING_TO_FAILED = ("verifying", "failed")
    FAILED_TO_DISPATCHED = ("failed", "dispatched")  # 重试


@dataclass
class StateRecord:
    """状态记录"""
    task_id: str
    from_state: str
    to_state: str
    transition_at: str = field(default_factory=lambda: datetime.now().isoformat())
    reason: str = ""
    verified: bool = False


class StateMachineManager:
    """
    状态机管理器

    核心规则：
    1. 状态转换必须有明确的原因
    2. 关键转换需要验证
    3. 所有转换必须记录
    4. 失败状态可以重试
    """

    ALLOWED_TRANSITIONS = {
        TaskState.IDLE: [TaskState.PLANNING],
        TaskState.PLANNING: [TaskState.DISPATCHED],
        TaskState.DISPATCHED: [TaskState.EXECUTING],
        TaskState.EXECUTING: [TaskState.VERIFYING],
        TaskState.VERIFYING: [TaskState.COMPLETED, TaskState.FAILED],
        TaskState.FAILED: [TaskState.DISPATCHED],  # 允许重试
        TaskState.COMPLETED: []  # 终态
    }

    def __init__(self, state_file: str = ""):
        self.state_file = state_file or os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "..", ".trae-instances", "state_machine.json"
        )
        self.task_states: Dict[str, TaskState] = {}
        self.transition_log: List[StateRecord] = []
        self._load_state()

    def transition(self, task_id: str, to_state: TaskState, reason: str = "", verified: bool = False) -> bool:
        """
        执行状态转换

        参数:
            task_id: 任务 ID
            to_state: 目标状态
            reason: 转换原因
            verified: 是否已验证

        返回:
            是否转换成功
        """
        current_state = self.task_states.get(task_id, TaskState.IDLE)

        # 检查是否允许转换
        allowed = self.ALLOWED_TRANSITIONS.get(current_state, [])
        if to_state not in allowed:
            return False

        # 执行转换
        self.task_states[task_id] = to_state

        # 记录转换
        record = StateRecord(
            task_id=task_id,
            from_state=current_state.value,
            to_state=to_state.value,
            reason=reason,
            verified=verified
        )
        self.transition_log.append(record)

        self._save_state()
        return True

    def get_state(self, task_id: str) -> TaskState:
        """获取任务当前状态"""
        return self.task_states.get(task_id, TaskState.IDLE)

    def get_all_states(self) -> Dict[str, str]:
        """获取所有任务状态"""
        return {k: v.value for k, v in self.task_states.items()}

    def get_transition_history(self, task_id: str = "") -> List[Dict]:
        """获取转换历史"""
        if task_id:
            return [asdict(r) for r in self.transition_log if r.task_id == task_id]
        return [asdict(r) for r in self.transition_log]

    def get_tasks_by_state(self, state: TaskState) -> List[str]:
        """获取指定状态的所有任务"""
        return [task_id for task_id, task_state in self.task_states.items() if task_state == state]

    def get_summary(self) -> Dict:
        """获取状态摘要"""
        summary = {}
        for state in TaskState:
            tasks = self.get_tasks_by_state(state)
            summary[state.value] = {
                "count": len(tasks),
                "task_ids": tasks
            }
        return summary

    def _load_state(self):
        """加载状态"""
        if os.path.exists(self.state_file):
            try:
                with open(self.state_file, 'r', encoding='utf-8') as f:
                    state = json.load(f)
                    self.task_states = {k: TaskState(v) for k, v in state.get("task_states", {}).items()}
                    self.transition_log = [StateRecord(**r) for r in state.get("transition_log", [])]
            except:
                pass

    def _save_state(self):
        """保存状态"""
        os.makedirs(os.path.dirname(self.state_file), exist_ok=True)
        state = {
            "task_states": {k: v.value for k, v in self.task_states.items()},
            "transition_log": [asdict(r) for r in self.transition_log]
        }
        with open(self.state_file, 'w', encoding='utf-8') as f:
            json.dump(state, f, ensure_ascii=False, indent=2)
