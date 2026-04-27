"""
Intent Translator - 意图翻译器

将高层意图分解为原子任务，供调度器分配。
"""

import json
import uuid
from typing import List, Dict, Optional
from datetime import datetime
from dataclasses import dataclass, field, asdict


@dataclass
class AtomicTask:
    """原子任务 - 不可再分的最小执行单元"""
    id: str = field(default_factory=lambda: str(uuid.uuid4())[:8])
    description: str = ""
    type: str = ""  # coding, testing, reviewing, deploying
    target_instance: Optional[str] = None
    dependencies: List[str] = field(default_factory=list)
    expected_output: str = ""
    verification_criteria: List[str] = field(default_factory=list)
    priority: str = "normal"  # high, normal, low
    status: str = "pending"  # pending, dispatched, executing, completed, failed
    error: Optional[str] = None

    def to_dict(self) -> Dict:
        return asdict(self)


@dataclass
class Intent:
    """用户意图"""
    id: str = field(default_factory=lambda: str(uuid.uuid4())[:8])
    description: str = ""
    tasks: List[AtomicTask] = field(default_factory=list)
    status: str = "pending"  # pending, executing, completed, failed
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())

    def to_dict(self) -> Dict:
        return {
            "id": self.id,
            "description": self.description,
            "tasks": [t.to_dict() for t in self.tasks],
            "status": self.status,
            "created_at": self.created_at
        }


class IntentTranslator:
    """
    意图翻译器

    核心职责：将自然语言意图分解为原子任务
    """

    # 任务类型模板
    TASK_TEMPLATES = {
        "frontend": {
            "type": "coding",
            "verification_criteria": [
                "文件存在于正确路径",
                "代码语法正确",
                "组件可正常渲染"
            ]
        },
        "backend": {
            "type": "coding",
            "verification_criteria": [
                "API 端点可访问",
                "数据库操作正常",
                "单元测试通过"
            ]
        },
        "testing": {
            "type": "testing",
            "verification_criteria": [
                "测试覆盖率达到要求",
                "所有测试用例通过",
                "边界条件已验证"
            ]
        }
    }

    def translate(self, intent_description: str) -> Intent:
        """
        将意图描述翻译为原子任务列表

        参数:
            intent_description: 自然语言意图描述

        返回:
            Intent 对象，包含分解后的原子任务
        """
        intent = Intent(description=intent_description)

        # 解析意图关键词
        keywords = self._extract_keywords(intent_description)

        # 根据关键词生成任务
        if "前端" in keywords or "frontend" in keywords:
            intent.tasks.append(self._create_frontend_task(intent_description))

        if "后端" in keywords or "backend" in keywords:
            intent.tasks.append(self._create_backend_task(intent_description))

        if "测试" in keywords or "testing" in keywords:
            intent.tasks.append(self._create_testing_task(intent_description))

        # 如果没有匹配到具体类型，创建通用任务
        if not intent.tasks:
            intent.tasks.append(self._create_general_task(intent_description))

        return intent

    def _extract_keywords(self, description: str) -> List[str]:
        """提取关键词"""
        keywords = []
        task_types = ["前端", "后端", "测试", "部署", "数据库", "UI", "API", "frontend", "backend", "testing"]
        for keyword in task_types:
            if keyword.lower() in description.lower():
                keywords.append(keyword.lower())
        return keywords

    def _create_frontend_task(self, intent_desc: str) -> AtomicTask:
        """创建前端任务"""
        template = self.TASK_TEMPLATES["frontend"]
        return AtomicTask(
            description=f"实现前端功能: {intent_desc}",
            type=template["type"],
            verification_criteria=template["verification_criteria"],
            priority="high"
        )

    def _create_backend_task(self, intent_desc: str) -> AtomicTask:
        """创建后端任务"""
        template = self.TASK_TEMPLATES["backend"]
        return AtomicTask(
            description=f"实现后端功能: {intent_desc}",
            type=template["type"],
            verification_criteria=template["verification_criteria"],
            priority="high"
        )

    def _create_testing_task(self, intent_desc: str) -> AtomicTask:
        """创建测试任务"""
        template = self.TASK_TEMPLATES["testing"]
        return AtomicTask(
            description=f"编写测试: {intent_desc}",
            type=template["type"],
            verification_criteria=template["verification_criteria"],
            priority="normal"
        )

    def _create_general_task(self, intent_desc: str) -> AtomicTask:
        """创建通用任务"""
        return AtomicTask(
            description=intent_desc,
            type="coding",
            verification_criteria=["任务按预期完成"],
            priority="normal"
        )
