import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
import hashlib
import shutil
import subprocess
import json
from types import SimpleNamespace
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).parent))
spec = importlib.util.spec_from_file_location('loop_facade', Path(__file__).with_name('loop.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class LoopTests(unittest.TestCase):
    def test_recovery_rejects_tampered_transition_identity(self):
        uid = 'task_' + 'a' * 32
        old = {'task_uid':uid,'owner_role':'repository_health_engineer','bootstrap_epoch':1}
        new = dict(old,bootstrap_epoch=2)
        expected = json.dumps(new,sort_keys=True)
        task = {'task_uid':uid,'owner_role':old['owner_role'],'repository':'eng-cc/oasis7','issue_number':1,'loop_binding':old}
        action = {'task_uid':uid,'repository':task['repository'],'issue_number':1,'kind':'bind_loop','expected':expected,
                  'action_id':'bind:'+hashlib.sha256(expected.encode()).hexdigest(),'previous_binding':old,'previous_epoch':1}
        for mutation in ({'action_id':'bind:forged'},{'issue_number':2},{'previous_epoch':0}):
            with patch.object(module,'common_dir',return_value=Path('/absent')), patch.object(module,'recovery_status',return_value={'pending_actions':[{**action,**mutation}]}), patch.object(module,'_trusted_module') as trusted:
                with self.assertRaises(ValueError): module.recovery_task(Path('/absent'),task,Path('/trusted'))
                trusted.assert_not_called()

    def test_recovery_rejects_binding_outside_recorded_transition(self):
        uid = 'task_' + 'a' * 32
        old = {'task_uid':uid,'owner_role':'repository_health_engineer','bootstrap_epoch':1}
        new = dict(old,bootstrap_epoch=2)
        expected = json.dumps(new,sort_keys=True)
        task = {'task_uid':uid,'owner_role':old['owner_role'],'repository':'eng-cc/oasis7','issue_number':1,'loop_binding':old}
        action = {'task_uid':uid,'repository':task['repository'],'issue_number':1,'kind':'bind_loop','expected':expected,'action_id':'bind:'+hashlib.sha256(expected.encode()).hexdigest(),'previous_binding':old,'previous_epoch':1}
        with patch.object(module,'common_dir',return_value=Path('/absent')), patch.object(module,'recovery_status',return_value={'pending_actions':[action]}), patch.object(module,'live_binding',return_value=dict(new,bootstrap_epoch=3)):
            with self.assertRaisesRegex(ValueError,'outside journal'): module.recovery_task(Path('/absent'),task,Path('/trusted'))

    def test_legacy_passes_without_activation(self):
        self.assertEqual(module.validate_task(Path('.'), {'task_uid': 'x'}, None)['status'], 'legacy')

    def test_loop_without_trusted_source_fails_closed(self):
        result = module.validate_task(Path('.'), {'loop_binding': {'loop': 'code'}}, None)
        self.assertEqual(result['status'], 'blocked')

    def test_empty_binding_is_not_legacy(self):
        self.assertEqual(module.validate_task(Path('.'), {'loop_binding': {}}, None)['status'], 'blocked')

    def test_new_task_does_not_consume_in_flight_only_contract(self):
        policy = SimpleNamespace(validate_binding=lambda _: {'blockers': []}, validate_tool_root=lambda *args: {'blockers': []}, validate_dependencies=lambda *args: {'blockers': []})
        contracts = SimpleNamespace(validate_contracts=lambda *args, purpose: {'blockers': ['new_tasks disabled'] if purpose == 'new_tasks' else []})
        task = {'task_uid': 'task_' + 'a' * 32, 'owner_role': 'qa_engineer', 'bootstrap_epoch': 1}
        task['loop_binding'] = dict(task)
        with patch.object(module, '_trusted_module', side_effect=lambda *args: policy if args[-1] == 'loop_policy' else contracts):
            self.assertEqual(module.validate_task(Path('.'), task, Path('.'), purpose='new_tasks')['status'], 'blocked')
            self.assertEqual(module.validate_task(Path('.'), task, Path('.'), purpose='in_flight')['status'], 'passed')

    def test_missing_live_dependency_blocks_admission(self):
        policy = SimpleNamespace(validate_binding=lambda _: {'blockers': []}, validate_tool_root=lambda *args: {'blockers': []})
        binding = {'task_uid': 'task_' + 'a' * 32, 'dependencies': ['task_' + 'b' * 32]}
        with patch.object(module, '_trusted_module', return_value=policy), patch.object(module.subprocess, 'check_output', return_value='[]'):
            result = module.validate_task(Path('.'), {'loop_binding': binding, 'repository': 'fixture/repo'}, Path('.'))
        self.assertEqual(result['status'], 'blocked')
        self.assertIn('dependency task missing', result['blockers'][0])

    def test_in_progress_dependency_blocks_admission(self):
        policy = SimpleNamespace(validate_binding=lambda _: {'blockers': []}, validate_tool_root=lambda *args: {'blockers': []})
        terminal = SimpleNamespace(validate_terminal_delivery=lambda *args: {'status': 'blocked', 'blockers': ['Project has not finalized dependency']})
        binding = {'task_uid': 'task_' + 'a' * 32, 'dependencies': ['task_' + 'b' * 32]}
        with patch.object(module, '_trusted_module', side_effect=lambda *args: terminal if args[-1] == 'loop_terminal' else policy), patch.object(module.subprocess, 'check_output', return_value='[{"number":1}]'):
            result = module.validate_task(Path('.'), {'loop_binding': binding, 'repository': 'fixture/repo'}, Path('.'))
        self.assertEqual(result['status'], 'blocked')
        self.assertIn('not successfully completed', result['blockers'][0])

    def test_completed_dependency_uses_terminal_reader(self):
        policy = SimpleNamespace(validate_binding=lambda _: {'blockers': []}, validate_tool_root=lambda *args: {'blockers': []}, validate_dependencies=lambda *args: {'blockers': []})
        terminal = SimpleNamespace(validate_terminal_delivery=lambda *args: {'status': 'passed', 'blockers': []})
        contracts = SimpleNamespace(validate_contracts=lambda *args, **kwargs: {'blockers': []})
        uid, dependency_uid = 'task_' + 'a' * 32, 'task_' + 'b' * 32
        binding = {'task_uid': uid, 'dependencies': [dependency_uid]}
        modules = {'loop_policy': policy, 'loop_terminal': terminal, 'loop_contracts': contracts}
        with patch.object(module, '_trusted_module', side_effect=lambda *args: modules[args[-1]]), patch.object(module.subprocess, 'check_output', return_value='[{"number":1}]'), patch.object(module, 'live_binding', return_value={'task_uid': dependency_uid, 'dependencies': []}):
            result = module.validate_task(Path('.'), {'task_uid': uid, 'loop_binding': binding, 'repository': 'fixture/repo'}, Path('.'))
        self.assertEqual(result['status'], 'passed', result)

    def test_trusted_effective_fixture_and_candidate_tamper(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'repo'
            root.mkdir()
            git = lambda *args: subprocess.check_output(['git', '-C', str(root), *args], text=True).strip()
            git('init', '-q')
            git('config', 'user.name', 'Fixture')
            git('config', 'user.email', 'fixture@example.invalid')
            helpers = root / 'scripts/pm'
            helpers.mkdir(parents=True)
            for filename in ('loop.py', 'loop_gate.py', 'loop_recovery.py', 'loop_policy.py', 'loop_contracts.py', 'loop_terminal.py', 'loop-policy.v1.json'):
                shutil.copy2(Path(__file__).with_name(filename), helpers / filename)
            product = root / 'doc/product/a.md'
            product.parent.mkdir(parents=True)
            product.write_text('before')
            git('add', '.')
            git('commit', '-qm', 'effective')
            base = git('rev-parse', 'HEAD')
            git('update-ref', 'refs/remotes/origin/main', base)
            trusted = Path(tmp) / 'trusted'
            git('worktree', 'add', '--detach', str(trusted), base)
            product.write_text('after')
            git('add', '.')
            git('commit', '-qm', 'candidate')
            uid = 'task_' + 'a' * 32
            binding = dict(schema='oasis7.loop-task/v1', task_uid=uid, change_id='c', loop='product', owner_role='gameplay_designer', bootstrap_epoch=1, manual_request_ref='user-1', request_key='r', write_scope=['doc/product/**'], out_of_scope=[], input_contracts=[], acceptance_refs=['a'], dependencies=[], target_delivery='pilot', policy_commit=base, policy_digest='sha256:' + hashlib.sha256((helpers / 'loop-policy.v1.json').read_bytes()).hexdigest())
            task = dict(task_uid=uid, owner_role='gameplay_designer', bootstrap_epoch=1, loop_binding=binding)
            result = module.validate_task(root, task, trusted, base, git('rev-parse', 'HEAD'))
            self.assertEqual(result['status'], 'passed', result)
            (trusted / 'scripts/pm/loop_policy.py').write_text('raise Exception("tampered")')
            result = module.validate_task(root, task, trusted, base, git('rev-parse', 'HEAD'))
            self.assertEqual(result['status'], 'blocked', result)

if __name__ == '__main__': unittest.main()
