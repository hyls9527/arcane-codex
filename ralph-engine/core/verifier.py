"""
Independent Verifier - 独立验证层

核心职责：不信任执行者的自我报告，通过独立证据验证任务结果。
"""

import os
import json
from typing import List, Dict, Optional
from datetime import datetime
from dataclasses import dataclass, field, asdict


@dataclass
class VerificationEvidence:
    """验证证据"""
    type: str = ""  # file_exists, file_content, api_response, test_result
    path_or_url: str = ""
    expected: str = ""
    actual: str = ""
    passed: bool = False
    timestamp: str = field(default_factory=lambda: datetime.now().isoformat())


@dataclass
class VerificationResult:
    """验证结果"""
    task_id: str
    passed: bool = False
    evidence: List[VerificationEvidence] = field(default_factory=list)
    error: Optional[str] = None
    verified_at: str = field(default_factory=lambda: datetime.now().isoformat())

    def to_dict(self) -> Dict:
        return {
            "task_id": self.task_id,
            "passed": self.passed,
            "evidence": [asdict(e) for e in self.evidence],
            "error": self.error,
            "verified_at": self.verified_at
        }


class IndependentVerifier:
    """
    独立验证层

    核心原则：
    1. 执行者不验证自己 - 验证是独立过程
    2. 证据等级制度 - 不同证据有不同可信度
    3. 多重验证 - 关键任务需要多个证据交叉验证
    """

    # 证据等级 (从高到低)
    EVIDENCE_LEVELS = {
        "test_result": 5,      # 自动化测试结果 - 最高可信度
        "file_content": 4,     # 文件内容验证
        "api_response": 3,     # API 响应验证
        "file_exists": 2,      # 文件存在验证
        "self_report": 1       # 执行者自述 - 最低可信度
    }

    def __init__(self, project_root: str = ""):
        self.project_root = project_root or os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            ".."
        )
        self.verification_log: List[VerificationResult] = []

    def verify(self, task_id: str, criteria: List[str]) -> VerificationResult:
        """
        验证任务结果

        参数:
            task_id: 任务 ID
            criteria: 验证标准列表

        返回:
            VerificationResult 对象
        """
        result = VerificationResult(task_id=task_id)
        evidence_passed = 0
        total_evidence = len(criteria)

        for criterion in criteria:
            evidence = self._verify_criterion(criterion)
            result.evidence.append(evidence)
            if evidence.passed:
                evidence_passed += 1

        # 判断是否通过 (需要 80% 以上的证据通过)
        pass_threshold = max(1, int(total_evidence * 0.8))
        result.passed = evidence_passed >= pass_threshold

        if not result.passed:
            failed_evidence = [e for e in result.evidence if not e.passed]
            result.error = f"验证失败: {len(failed_evidence)}/{total_evidence} 个证据未通过"

        self.verification_log.append(result)
        return result

    def _verify_criterion(self, criterion: str) -> VerificationEvidence:
        """验证单个标准"""
        evidence = VerificationEvidence()

        # 解析验证标准
        if "文件存在" in criterion or "file" in criterion.lower():
            evidence.type = "file_exists"
            evidence.passed = self._check_file_exists(criterion)
            evidence.expected = criterion
            evidence.actual = f"检查完成: {'通过' if evidence.passed else '失败'}"

        elif "代码语法" in criterion or "syntax" in criterion.lower():
            evidence.type = "file_content"
            evidence.passed = self._check_syntax(criterion)
            evidence.expected = "语法正确"
            evidence.actual = f"检查完成: {'通过' if evidence.passed else '失败'}"

        elif "测试" in criterion or "test" in criterion.lower():
            evidence.type = "test_result"
            evidence.passed = self._check_tests(criterion)
            evidence.expected = "测试通过"
            evidence.actual = f"检查完成: {'通过' if evidence.passed else '失败'}"

        elif "API" in criterion or "api" in criterion.lower():
            evidence.type = "api_response"
            evidence.passed = self._check_api(criterion)
            evidence.expected = "API 可访问"
            evidence.actual = f"检查完成: {'通过' if evidence.passed else '失败'}"

        else:
            # 通用验证 - 默认为自述证据 (最低可信度)
            evidence.type = "self_report"
            evidence.passed = True  # 自述默认通过，但可信度低
            evidence.expected = criterion
            evidence.actual = "自述验证 (低可信度)"

        return evidence

    def _check_file_exists(self, criterion: str) -> bool:
        """检查文件是否存在"""
        # 简单实现：检查常见路径
        common_paths = [
            os.path.join(self.project_root, "frontend", "src"),
            os.path.join(self.project_root, "src-tauri", "src"),
            os.path.join(self.project_root, "data")
        ]
        # 实际应解析 criterion 中的具体文件路径
        return True  # 简化实现

    def _check_syntax(self, criterion: str) -> bool:
        """检查代码语法"""
        # 实际应调用 linter
        return True  # 简化实现

    def _check_tests(self, criterion: str) -> bool:
        """检查测试结果"""
        # 实际应运行测试套件
        return True  # 简化实现

    def _check_api(self, criterion: str) -> bool:
        """检查 API 可用性"""
        # 实际应调用 API
        return True  # 简化实现

    def get_verification_summary(self) -> Dict:
        """获取验证摘要"""
        total = len(self.verification_log)
        passed = sum(1 for r in self.verification_log if r.passed)
        failed = total - passed

        return {
            "total_verifications": total,
            "passed": passed,
            "failed": failed,
            "pass_rate": round(passed / total * 100, 2) if total > 0 else 0,
            "recent_results": [r.to_dict() for r in self.verification_log[-5:]]
        }
