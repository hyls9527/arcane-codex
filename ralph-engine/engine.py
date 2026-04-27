"""
Ralph Engine - 主引擎

整合所有模块，提供完整的意图驱动执行流程。
"""

import json
import os
from typing import List, Dict, Optional
from datetime import datetime

try:
    from .core.intent import IntentTranslator, Intent, AtomicTask
    from .core.dispatcher import TaskDispatcher, InstanceCapability
    from .core.verifier import IndependentVerifier, VerificationResult
    from .core.state_machine import StateMachineManager, TaskState
except ImportError:
    from core.intent import IntentTranslator, Intent, AtomicTask
    from core.dispatcher import TaskDispatcher, InstanceCapability
    from core.verifier import IndependentVerifier, VerificationResult
    from core.state_machine import StateMachineManager, TaskState


class RalphEngine:
    """
    Ralph 执行引擎

    核心流程：
    1. 提交意图 → 翻译为原子任务
    2. 调度任务 → 分配给合适实例
    3. 执行任务 → 通过 Trae CN 实例执行
    4. 验证结果 → 独立验证层检查
    5. 反馈循环 → 报告状态，触发下一步
    """

    def __init__(self, project_root: str = ""):
        self.project_root = project_root or os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            ".."
        )

        # 初始化各模块
        self.translator = IntentTranslator()
        self.dispatcher = TaskDispatcher()
        self.verifier = IndependentVerifier(self.project_root)
        self.state_machine = StateMachineManager()

        # 意图和任务存储
        self.intents: Dict[str, Intent] = {}
        self.tasks: Dict[str, AtomicTask] = {}

    def submit_intent(self, description: str) -> Dict:
        """
        提交意图

        参数:
            description: 自然语言意图描述

        返回:
            意图和任务信息
        """
        # 1. 翻译意图
        intent = self.translator.translate(description)
        self.intents[intent.id] = intent

        # 2. 存储任务
        for task in intent.tasks:
            self.tasks[task.id] = task
            # 更新状态机
            self.state_machine.transition(task.id, TaskState.PLANNING, reason="意图翻译完成")

        # 3. 调度任务
        assignments = self.dispatcher.dispatch(intent.tasks)

        # 4. 更新状态
        for task in intent.tasks:
            if task.status == "dispatched":
                self.state_machine.transition(task.id, TaskState.DISPATCHED, reason="任务已调度")

        return {
            "intent_id": intent.id,
            "description": intent.description,
            "task_count": len(intent.tasks),
            "tasks": [t.to_dict() for t in intent.tasks],
            "assignments": [
                {
                    "task_id": a.task_id,
                    "instance_id": a.instance_id
                }
                for a in assignments
            ],
            "status": "dispatched"
        }

    def register_instance(self, instance_id: str, capabilities: List[str], max_concurrent: int = 3):
        """注册实例"""
        self.dispatcher.register_instance(instance_id, capabilities, max_concurrent)

    def get_status(self) -> Dict:
        """获取引擎状态"""
        return {
            "intents": {
                id: intent.to_dict()
                for id, intent in self.intents.items()
            },
            "instances": self.dispatcher.get_instance_status(),
            "state_summary": self.state_machine.get_summary(),
            "verification_summary": self.verifier.get_verification_summary()
        }

    def verify_task(self, task_id: str) -> Dict:
        """验证任务"""
        task = self.tasks.get(task_id)
        if not task:
            return {"error": f"任务 {task_id} 不存在"}

        # 自动完成到 VERIFYING 前的必要状态转换
        current_state = self.state_machine.get_state(task_id)
        if current_state == TaskState.DISPATCHED:
            self.state_machine.transition(task_id, TaskState.EXECUTING, reason="自动执行")
        elif current_state == TaskState.EXECUTING:
            pass  # 已经在 EXECUTING，下一步就是 VERIFYING
        # 如果已经在 VERIFYING 或之后，不需要额外转换

        # 更新状态到 VERIFYING
        self.state_machine.transition(task_id, TaskState.VERIFYING, reason="开始验证")

        # 执行验证
        result = self.verifier.verify(task_id, task.verification_criteria)

        # 根据验证结果更新状态
        if result.passed:
            self.state_machine.transition(task_id, TaskState.COMPLETED, reason="验证通过", verified=True)
            task.status = "completed"
            self.dispatcher.complete_assignment(task_id, success=True)
        else:
            self.state_machine.transition(task_id, TaskState.FAILED, reason="验证失败", verified=True)
            task.status = "failed"
            task.error = result.error
            self.dispatcher.complete_assignment(task_id, success=False, error=result.error)

        return result.to_dict()

    def retry_failed_tasks(self) -> Dict:
        """重试失败的任务"""
        failed_tasks = self.state_machine.get_tasks_by_state(TaskState.FAILED)
        retried = []

        for task_id in failed_tasks:
            # 转换状态回 DISPATCHED
            if self.state_machine.transition(task_id, TaskState.DISPATCHED, reason="重试"):
                task = self.tasks.get(task_id)
                if task:
                    task.status = "dispatched"
                    task.error = None
                    retried.append(task_id)

        return {
            "retried": retried,
            "count": len(retried)
        }
