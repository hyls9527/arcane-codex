"""
Ralph Engine 综合测试套件

测试覆盖:
1. 意图翻译 (IntentTranslator) - 自然语言到原子任务的转换
2. 任务调度 (TaskDispatcher) - 多实例能力匹配和负载均衡
3. 状态机 (StateMachineManager) - 状态转换合规性
4. 验证流水线 (IndependentVerifier) - 独立验证
5. 引擎集成 (RalphEngine) - 端到端完整流程
"""

import sys
import os
import unittest
import tempfile

# 添加 ralph-engine 目录到路径
RALPH_ENGINE_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, RALPH_ENGINE_DIR)

# 使用相对导入
from core.intent import IntentTranslator, Intent, AtomicTask
from core.dispatcher import TaskDispatcher, InstanceCapability, TaskAssignment
from core.state_machine import StateMachineManager, TaskState
from core.verifier import IndependentVerifier, VerificationResult
from engine import RalphEngine


class TestIntentTranslator(unittest.TestCase):
    """测试意图翻译器"""

    def setUp(self):
        self.translator = IntentTranslator()

    def test_translate_frontend_intent(self):
        """测试前端意图翻译"""
        intent = self.translator.translate("开发前端页面")
        
        self.assertIsInstance(intent, Intent)
        self.assertTrue(len(intent.tasks) > 0)
        self.assertEqual(intent.description, "开发前端页面")
        
        # 验证任务类型
        frontend_tasks = [t for t in intent.tasks if "前端" in t.description.lower() or "frontend" in t.type.lower()]
        self.assertTrue(len(frontend_tasks) > 0)
        self.assertEqual(frontend_tasks[0].priority, "high")

    def test_translate_backend_intent(self):
        """测试后端意图翻译"""
        intent = self.translator.translate("实现后端 API")
        
        self.assertTrue(len(intent.tasks) > 0)
        backend_tasks = [t for t in intent.tasks if "后端" in t.description]
        self.assertTrue(len(backend_tasks) > 0)
        self.assertEqual(backend_tasks[0].type, "coding")

    def test_translate_testing_intent(self):
        """测试测试意图翻译"""
        intent = self.translator.translate("编写单元测试")
        
        testing_tasks = [t for t in intent.tasks if t.type == "testing"]
        self.assertTrue(len(testing_tasks) > 0)

    def test_translate_mixed_intent(self):
        """测试混合意图翻译"""
        intent = self.translator.translate("开发前端页面和后端 API")
        
        self.assertGreaterEqual(len(intent.tasks), 2)
        types = [t.type for t in intent.tasks]
        self.assertIn("coding", types)

    def test_translate_general_intent(self):
        """测试通用意图翻译"""
        intent = self.translator.translate("随便做点什么")
        
        self.assertEqual(len(intent.tasks), 1)
        self.assertEqual(intent.tasks[0].type, "coding")

    def test_task_generation(self):
        """测试任务生成包含必要字段"""
        intent = self.translator.translate("后端开发任务")
        task = intent.tasks[0]
        
        self.assertIsNotNone(task.id)
        self.assertIsInstance(task.id, str)
        self.assertTrue(len(task.id) > 0)
        self.assertIsInstance(task.verification_criteria, list)
        self.assertIn(task.status, ["pending"])
        self.assertIn(task.priority, ["high", "normal", "low"])

    def test_intent_to_dict(self):
        """测试意图序列化"""
        intent = self.translator.translate("前端测试")
        d = intent.to_dict()
        
        self.assertIn("id", d)
        self.assertIn("description", d)
        self.assertIn("tasks", d)
        self.assertIn("status", d)
        self.assertIn("created_at", d)
        self.assertIsInstance(d["tasks"], list)

    def test_task_to_dict(self):
        """测试任务序列化"""
        task = AtomicTask(
            id="test-001",
            description="测试任务",
            type="coding",
            priority="high"
        )
        d = task.to_dict()
        
        self.assertEqual(d["id"], "test-001")
        self.assertEqual(d["description"], "测试任务")
        self.assertEqual(d["type"], "coding")
        self.assertEqual(d["priority"], "high")


class TestTaskDispatcher(unittest.TestCase):
    """测试任务调度器"""

    def setUp(self):
        # 使用唯一临时状态文件避免测试间干扰
        import uuid
        unique_id = uuid.uuid4().hex[:8]
        self.temp_state = os.path.join(tempfile.gettempdir(), f"ralph_disp_{unique_id}.json")
        # 创建空文件确保没有旧数据
        with open(self.temp_state, 'w') as f:
            f.write('{}')
        self.dispatcher = TaskDispatcher(state_file=self.temp_state)

    def test_register_instance(self):
        """测试实例注册"""
        self.dispatcher.register_instance("agent-001", ["frontend", "testing"])
        
        self.assertIn("agent-001", self.dispatcher.instances)
        inst = self.dispatcher.instances["agent-001"]
        self.assertEqual(inst.instance_id, "agent-001")
        self.assertIn("frontend", inst.capabilities)
        self.assertIn("testing", inst.capabilities)

    def test_register_multiple_instances(self):
        """测试多实例注册"""
        self.dispatcher.register_instance("frontend-agent", ["frontend"])
        self.dispatcher.register_instance("backend-agent", ["backend"])
        self.dispatcher.register_instance("test-agent", ["testing"])
        
        self.assertEqual(len(self.dispatcher.instances), 3)

    def test_dispatch_to_capable_instance(self):
        """测试基于能力的任务分配"""
        self.dispatcher.register_instance("frontend-agent", ["frontend"])
        self.dispatcher.register_instance("backend-agent", ["backend"])
        
        tasks = [
            AtomicTask(id="task-1", description="前端开发", type="coding"),
        ]
        
        assignments = self.dispatcher.dispatch(tasks)
        
        self.assertEqual(len(assignments), 1)
        # 应该分配给有 frontend 能力的实例
        self.assertEqual(assignments[0].instance_id, "frontend-agent")

    def test_dispatch_load_balancing(self):
        """测试负载均衡"""
        self.dispatcher.register_instance("agent-a", ["coding"], max_concurrent=5)
        self.dispatcher.register_instance("agent-b", ["coding"], max_concurrent=5)
        
        # agent-a 已经有 3 个任务，agent-b 有 1 个
        self.dispatcher.instances["agent-a"].current_load = 3
        self.dispatcher.instances["agent-b"].current_load = 1
        
        tasks = [AtomicTask(id="task-1", type="coding")]
        assignments = self.dispatcher.dispatch(tasks)
        
        # 应该分配给负载更低的 agent-b
        self.assertEqual(assignments[0].instance_id, "agent-b")

    def test_dispatch_respects_max_concurrent(self):
        """测试最大并发限制"""
        self.dispatcher.register_instance("agent-a", ["coding"], max_concurrent=2)
        self.dispatcher.instances["agent-a"].current_load = 2  # 已满
        
        self.dispatcher.register_instance("agent-b", ["coding"], max_concurrent=3)
        
        tasks = [AtomicTask(id="task-1", type="coding")]
        assignments = self.dispatcher.dispatch(tasks)
        
        # 应该跳过满负载的 agent-a
        self.assertEqual(assignments[0].instance_id, "agent-b")

    def test_dispatch_dependency_ordering(self):
        """测试依赖任务排序"""
        self.dispatcher.register_instance("agent-a", ["coding"])
        
        task2 = AtomicTask(id="task-2", dependencies=["task-1"], type="coding")
        task1 = AtomicTask(id="task-1", type="coding")
        
        assignments = self.dispatcher.dispatch([task2, task1])
        
        # task-1 应该先被分配
        self.assertEqual(assignments[0].task_id, "task-1")
        self.assertEqual(assignments[1].task_id, "task-2")

    def test_dispatch_no_available_instance(self):
        """测试没有可用实例"""
        tasks = [AtomicTask(id="task-1", type="coding")]
        assignments = self.dispatcher.dispatch(tasks)
        
        # 没有注册实例，应该没有分配
        self.assertEqual(len(assignments), 0)

    def test_complete_assignment_reduces_load(self):
        """测试完成任务后负载减少"""
        self.dispatcher.register_instance("agent-a", ["coding"])
        
        tasks = [AtomicTask(id="task-1", type="coding")]
        self.dispatcher.dispatch(tasks)
        
        self.assertEqual(self.dispatcher.instances["agent-a"].current_load, 1)
        
        self.dispatcher.complete_assignment("task-1", success=True)
        
        self.assertEqual(self.dispatcher.instances["agent-a"].current_load, 0)
        self.assertEqual(self.dispatcher.instances["agent-a"].status, "idle")

    def test_skip_error_instances(self):
        """测试跳过错误状态实例"""
        self.dispatcher.register_instance("agent-error", ["coding"])
        self.dispatcher.instances["agent-error"].status = "error"
        
        self.dispatcher.register_instance("agent-ok", ["coding"])
        
        tasks = [AtomicTask(id="task-1", type="coding")]
        assignments = self.dispatcher.dispatch(tasks)
        
        self.assertEqual(assignments[0].instance_id, "agent-ok")

    def test_get_instance_status(self):
        """测试获取实例状态"""
        self.dispatcher.register_instance("agent-a", ["frontend", "backend"], max_concurrent=5)
        
        status = self.dispatcher.get_instance_status()
        
        self.assertIn("agent-a", status)
        self.assertEqual(status["agent-a"]["capabilities"], ["frontend", "backend"])
        self.assertEqual(status["agent-a"]["max_concurrent"], 5)


class TestStateMachine(unittest.TestCase):
    """测试状态机"""

    def setUp(self):
        # 使用唯一临时状态文件避免测试间干扰
        import uuid
        unique_id = uuid.uuid4().hex[:8]
        self.temp_state = os.path.join(tempfile.gettempdir(), f"ralph_sm_{unique_id}.json")
        # 删除可能存在的旧文件
        if os.path.exists(self.temp_state):
            os.remove(self.temp_state)
        self.sm = StateMachineManager(state_file=self.temp_state)
        # 确保绝对干净
        self.sm.task_states.clear()
        self.sm.transition_log.clear()

    def test_initial_state_is_idle(self):
        """测试初始状态为 IDLE"""
        state = self.sm.get_state("new-task-001")
        self.assertEqual(state, TaskState.IDLE)

    def test_valid_transition(self):
        """测试有效状态转换"""
        result = self.sm.transition("task-001", TaskState.PLANNING, reason="测试")
        self.assertTrue(result)
        self.assertEqual(self.sm.get_state("task-001"), TaskState.PLANNING)

    def test_invalid_transition(self):
        """测试无效状态转换"""
        # 不能从 IDLE 直接到 COMPLETED
        result = self.sm.transition("task-001", TaskState.COMPLETED, reason="测试")
        self.assertFalse(result)

    def test_full_lifecycle(self):
        """测试完整生命周期"""
        task_id = "lifecycle-001"
        
        self.assertTrue(self.sm.transition(task_id, TaskState.PLANNING))
        self.assertTrue(self.sm.transition(task_id, TaskState.DISPATCHED))
        self.assertTrue(self.sm.transition(task_id, TaskState.EXECUTING))
        self.assertTrue(self.sm.transition(task_id, TaskState.VERIFYING))
        self.assertTrue(self.sm.transition(task_id, TaskState.COMPLETED))
        
        self.assertEqual(self.sm.get_state(task_id), TaskState.COMPLETED)

    def test_failed_retry_lifecycle(self):
        """测试失败重试生命周期"""
        task_id = "retry-001"
        
        self.sm.transition(task_id, TaskState.PLANNING)
        self.sm.transition(task_id, TaskState.DISPATCHED)
        self.sm.transition(task_id, TaskState.EXECUTING)
        self.sm.transition(task_id, TaskState.VERIFYING)
        self.sm.transition(task_id, TaskState.FAILED)
        
        # 重试
        self.assertTrue(self.sm.transition(task_id, TaskState.DISPATCHED, reason="重试"))
        self.assertEqual(self.sm.get_state(task_id), TaskState.DISPATCHED)

    def test_completed_is_terminal(self):
        """测试 COMPLETED 是终态"""
        task_id = "terminal-001"
        
        self.sm.transition(task_id, TaskState.PLANNING)
        self.sm.transition(task_id, TaskState.DISPATCHED)
        self.sm.transition(task_id, TaskState.EXECUTING)
        self.sm.transition(task_id, TaskState.VERIFYING)
        self.sm.transition(task_id, TaskState.COMPLETED)
        
        # 不能再转换
        result = self.sm.transition(task_id, TaskState.PLANNING, reason="不应该成功")
        self.assertFalse(result)

    def test_transition_history(self):
        """测试转换历史记录"""
        task_id = "history-001"
        
        self.sm.transition(task_id, TaskState.PLANNING, reason="步骤1")
        self.sm.transition(task_id, TaskState.DISPATCHED, reason="步骤2")
        
        history = self.sm.get_transition_history(task_id)
        self.assertEqual(len(history), 2)
        self.assertEqual(history[0]["reason"], "步骤1")
        self.assertEqual(history[1]["reason"], "步骤2")

    def test_get_tasks_by_state(self):
        """测试按状态获取任务"""
        self.sm.transition("task-a", TaskState.PLANNING)
        self.sm.transition("task-b", TaskState.PLANNING)
        self.sm.transition("task-c", TaskState.DISPATCHED)
        
        planning = self.sm.get_tasks_by_state(TaskState.PLANNING)
        self.assertEqual(len(planning), 2)
        self.assertIn("task-a", planning)
        self.assertIn("task-b", planning)

    def test_get_summary(self):
        """测试状态摘要"""
        # 需要通过合法状态转换
        self.sm.transition("t1", TaskState.PLANNING)
        self.sm.transition("t2", TaskState.PLANNING)
        self.sm.transition("t2", TaskState.DISPATCHED)  # PLANNING -> DISPATCHED
        self.sm.transition("t3", TaskState.PLANNING)
        self.sm.transition("t3", TaskState.DISPATCHED)
        self.sm.transition("t3", TaskState.EXECUTING)
        self.sm.transition("t3", TaskState.VERIFYING)
        self.sm.transition("t3", TaskState.COMPLETED)
        
        summary = self.sm.get_summary()
        
        self.assertEqual(summary["planning"]["count"], 1)  # t1
        self.assertEqual(summary["dispatched"]["count"], 1)  # t2
        self.assertEqual(summary["completed"]["count"], 1)  # t3

    def test_transition_reason_recorded(self):
        """测试转换原因被记录"""
        self.sm.transition("task-x", TaskState.PLANNING, reason="重要原因", verified=True)
        
        history = self.sm.get_transition_history("task-x")
        self.assertEqual(history[0]["reason"], "重要原因")
        self.assertTrue(history[0]["verified"])


class TestIndependentVerifier(unittest.TestCase):
    """测试独立验证层"""

    def setUp(self):
        self.verifier = IndependentVerifier()

    def test_verify_all_criteria_passed(self):
        """测试全部标准通过"""
        result = self.verifier.verify("task-001", [
            "文件存在",
            "代码语法正确",
        ])
        
        self.assertTrue(result.passed)
        self.assertEqual(result.task_id, "task-001")

    def test_verify_with_mixed_results(self):
        """测试混合结果"""
        result = self.verifier.verify("task-002", [
            "文件存在",
            "通用验证标准",  # 会被当作 self_report，默认通过
        ])
        
        # 80% 阈值，全部通过
        self.assertTrue(result.passed)

    def test_verification_result_to_dict(self):
        """测试验证结果序列化"""
        result = self.verifier.verify("task-003", ["测试通过"])
        d = result.to_dict()
        
        self.assertIn("task_id", d)
        self.assertIn("passed", d)
        self.assertIn("evidence", d)
        self.assertIn("verified_at", d)

    def test_evidence_types(self):
        """测试不同证据类型"""
        # 文件存在
        result1 = self.verifier.verify("t1", ["文件存在"])
        self.assertEqual(result1.evidence[0].type, "file_exists")
        
        # 代码语法
        result2 = self.verifier.verify("t2", ["代码语法检查"])
        self.assertEqual(result2.evidence[0].type, "file_content")
        
        # 测试
        result3 = self.verifier.verify("t3", ["测试覆盖率"])
        self.assertEqual(result3.evidence[0].type, "test_result")
        
        # API
        result4 = self.verifier.verify("t4", ["API 响应验证"])
        self.assertEqual(result4.evidence[0].type, "api_response")
        
        # 通用 (self_report)
        result5 = self.verifier.verify("t5", ["其他验证"])
        self.assertEqual(result5.evidence[0].type, "self_report")

    def test_verification_summary(self):
        """测试验证摘要"""
        self.verifier.verify("t1", ["标准1"])
        self.verifier.verify("t2", ["标准2"])
        
        summary = self.verifier.get_verification_summary()
        
        self.assertEqual(summary["total_verifications"], 2)
        self.assertEqual(summary["passed"], 2)
        self.assertIn("pass_rate", summary)

    def test_evidence_levels(self):
        """测试证据等级"""
        levels = IndependentVerifier.EVIDENCE_LEVELS
        self.assertGreater(levels["test_result"], levels["file_exists"])
        self.assertGreater(levels["file_content"], levels["self_report"])


class TestRalphEngineIntegration(unittest.TestCase):
    """Ralph 引擎集成测试"""

    def setUp(self):
        # 使用临时目录存储状态文件
        self.temp_dir = tempfile.mkdtemp()
        state_file = os.path.join(self.temp_dir, "sm.json")
        dispatch_file = os.path.join(self.temp_dir, "disp.json")
        
        self.engine = RalphEngine()
        # 替换为临时状态文件
        self.engine.state_machine.state_file = state_file
        self.engine.dispatcher.state_file = dispatch_file
        self.engine.state_machine.task_states.clear()
        self.engine.state_machine.transition_log.clear()
        self.engine.dispatcher.instances.clear()
        self.engine.dispatcher.assignments.clear()
        self.engine.verifier.verification_log.clear()
        self.engine.intents.clear()
        self.engine.tasks.clear()

    def test_submit_frontend_intent(self):
        """测试提交前端意图"""
        result = self.engine.submit_intent("开发前端登录页面")
        
        self.assertIn("intent_id", result)
        self.assertGreater(result["task_count"], 0)
        self.assertEqual(result["status"], "dispatched")
        self.assertEqual(result["description"], "开发前端登录页面")

    def test_multi_instance_dispatch(self):
        """测试多实例任务分发"""
        # 注册多个实例
        self.engine.register_instance("frontend-agent", ["frontend", "coding"], max_concurrent=3)
        self.engine.register_instance("backend-agent", ["backend", "coding"], max_concurrent=3)
        self.engine.register_instance("test-agent", ["testing"], max_concurrent=3)
        
        # 提交混合意图
        result = self.engine.submit_intent("前端和后端开发")
        
        self.assertGreaterEqual(result["task_count"], 1)
        self.assertEqual(result["status"], "dispatched")
        
        # 验证任务被分配
        assignments = result["assignments"]
        self.assertGreater(len(assignments), 0)

    def test_full_lifecycle(self):
        """测试完整生命周期: 意图 -> 翻译 -> 调度 -> 执行 -> 验证 -> 完成"""
        self.engine.register_instance("full-stack-agent", ["frontend", "backend", "coding", "testing"])
        
        # 1. 提交意图
        result = self.engine.submit_intent("前端开发任务")
        intent_id = result["intent_id"]
        
        self.assertIn(intent_id, self.engine.intents)
        self.assertEqual(self.engine.intents[intent_id].description, "前端开发任务")
        
        # 2. 获取任务
        self.assertGreater(len(result["tasks"]), 0)
        task = result["tasks"][0]
        task_id = task["id"]
        
        self.assertIn(task_id, self.engine.tasks)
        
        # 3. 状态机反映进度
        state = self.engine.state_machine.get_state(task_id)
        self.assertEqual(state, TaskState.DISPATCHED)
        
        # 4. 验证任务 (engine 会自动完成 DISPATCHED -> EXECUTING -> VERIFYING)
        verify_result = self.engine.verify_task(task_id)
        
        self.assertIn("passed", verify_result)
        self.assertTrue(verify_result["passed"])
        
        # 5. 任务状态更新为完成
        final_state = self.engine.state_machine.get_state(task_id)
        self.assertEqual(final_state, TaskState.COMPLETED)

    def test_get_status(self):
        """测试获取引擎状态"""
        self.engine.register_instance("agent-1", ["coding"])
        
        status = self.engine.get_status()
        
        self.assertIn("intents", status)
        self.assertIn("instances", status)
        self.assertIn("state_summary", status)
        self.assertIn("verification_summary", status)

    def test_retry_failed_tasks(self):
        """测试重试失败任务"""
        self.engine.register_instance("retry-agent", ["coding", "frontend"])
        
        result = self.engine.submit_intent("前端开发任务")
        task_id = result["tasks"][0]["id"]
        
        # 模拟验证失败
        self.engine.state_machine.transition(task_id, TaskState.EXECUTING, reason="测试执行")
        self.engine.state_machine.transition(task_id, TaskState.VERIFYING, reason="开始验证")
        self.engine.state_machine.transition(task_id, TaskState.FAILED, reason="验证失败", verified=True)
        self.engine.tasks[task_id].status = "failed"
        
        # 重试
        retry_result = self.engine.retry_failed_tasks()
        
        self.assertGreater(retry_result["count"], 0)
        self.assertIn(task_id, retry_result["retried"])
        
        # 状态回到 DISPATCHED
        state = self.engine.state_machine.get_state(task_id)
        self.assertEqual(state, TaskState.DISPATCHED)

    def test_no_instance_no_dispatch(self):
        """测试没有实例时无法调度"""
        # 不注册任何实例
        result = self.engine.submit_intent("开发前端")
        
        # 意图被翻译，但没有分配
        self.assertEqual(result["status"], "dispatched")
        self.assertEqual(len(result["assignments"]), 0)

    def test_state_machine_prevents_illegal_transitions(self):
        """测试状态机防止非法转换"""
        self.engine.register_instance("agent-x", ["coding"])
        result = self.engine.submit_intent("开发功能")
        task_id = result["tasks"][0]["id"]
        
        # 已经是 DISPATCHED 状态
        state = self.engine.state_machine.get_state(task_id)
        self.assertEqual(state, TaskState.DISPATCHED)
        
        # 尝试非法转换: DISPATCHED -> COMPLETED (跳过 EXECUTING 和 VERIFYING)
        illegal = self.engine.state_machine.transition(task_id, TaskState.COMPLETED, reason="非法跳跃")
        self.assertFalse(illegal)

    def test_multiple_intents_isolation(self):
        """测试多个意图隔离"""
        self.engine.register_instance("iso-agent", ["coding", "frontend"])
        
        r1 = self.engine.submit_intent("前端项目A")
        r2 = self.engine.submit_intent("前端项目B")
        
        self.assertNotEqual(r1["intent_id"], r2["intent_id"])
        self.assertEqual(len(self.engine.intents), 2)
        
        # 每个意图的任务独立
        tasks1 = [t for t in self.engine.tasks.values() 
                  if any(t.id in a["task_id"] for a in r1["assignments"])]
        tasks2 = [t for t in self.engine.tasks.values() 
                  if any(t.id in a["task_id"] for a in r2["assignments"])]
        
        # 验证任务不重叠
        ids1 = set(t.id for t in tasks1)
        ids2 = set(t.id for t in tasks2)
        self.assertEqual(len(ids1 & ids2), 0)

    def test_verification_updates_task_status(self):
        """测试验证结果更新任务状态"""
        self.engine.register_instance("verify-agent", ["coding"])
        
        result = self.engine.submit_intent("编码任务")
        task_id = result["tasks"][0]["id"]
        
        # 初始状态
        self.assertEqual(self.engine.tasks[task_id].status, "dispatched")
        
        # 执行到验证阶段
        self.engine.state_machine.transition(task_id, TaskState.EXECUTING, reason="执行")
        self.engine.state_machine.transition(task_id, TaskState.VERIFYING, reason="验证")
        
        # 验证
        verify_result = self.engine.verify_task(task_id)
        
        # 状态更新
        self.assertEqual(self.engine.tasks[task_id].status, "completed")
        self.assertEqual(self.engine.state_machine.get_state(task_id), TaskState.COMPLETED)


def run_tests():
    """运行所有测试并输出报告"""
    # 创建测试套件
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()
    
    suite.addTests(loader.loadTestsFromTestCase(TestIntentTranslator))
    suite.addTests(loader.loadTestsFromTestCase(TestTaskDispatcher))
    suite.addTests(loader.loadTestsFromTestCase(TestStateMachine))
    suite.addTests(loader.loadTestsFromTestCase(TestIndependentVerifier))
    suite.addTests(loader.loadTestsFromTestCase(TestRalphEngineIntegration))
    
    # 运行测试
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    
    # 输出摘要
    print("\n" + "=" * 70)
    print("Ralph 引擎测试报告")
    print("=" * 70)
    print(f"测试模块: 5")
    print(f"  - 意图翻译器 (IntentTranslator): 8 测试")
    print(f"  - 任务调度器 (TaskDispatcher): 10 测试")
    print(f"  - 状态机 (StateMachineManager): 10 测试")
    print(f"  - 独立验证层 (IndependentVerifier): 6 测试")
    print(f"  - 引擎集成 (RalphEngine): 9 测试")
    print(f"总测试数: {result.testsRun}")
    print(f"通过: {result.testsRun - len(result.failures) - len(result.errors)}")
    print(f"失败: {len(result.failures)}")
    print(f"错误: {len(result.errors)}")
    
    if result.failures:
        print("\n失败详情:")
        for test, traceback in result.failures:
            print(f"  - {test}: {traceback}")
    
    if result.errors:
        print("\n错误详情:")
        for test, traceback in result.errors:
            print(f"  - {test}: {traceback}")
    
    print("=" * 70)
    
    return result


if __name__ == "__main__":
    run_tests()
