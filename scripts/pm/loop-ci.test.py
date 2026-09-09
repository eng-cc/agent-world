import base64
import importlib.util
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('loop_ci', Path(__file__).with_name('loop-ci.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
UID = 'task_' + 'a' * 32


class CIGateTests(unittest.TestCase):
    def test_project_read_token_is_required_without_local_auth_fallback(self):
        with patch.dict(module.os.environ, {'GH_TOKEN': 'fixture-default-token'}, clear=True):
            with self.assertRaisesRegex(ValueError, 'activation requires OASIS7_LOOP_READ_TOKEN'):
                module.project_read_environment('fixture/repo')

    def test_project_read_probe_rejects_permission_failure(self):
        response = module.subprocess.CompletedProcess([], 1, '', 'permission denied')
        with patch.dict(module.os.environ, {'OASIS7_LOOP_READ_TOKEN': 'fixture-read-token'}, clear=True), patch.object(module.subprocess, 'run', return_value=response):
            with self.assertRaisesRegex(ValueError, 'cannot read canonical Project'):
                module.project_read_environment('fixture/repo')

    def test_project_read_credential_only_reaches_trusted_child_environment(self):
        response = module.subprocess.CompletedProcess([], 0, '{"id":"PVT_fixture","number":1}', '')
        with patch.dict(module.os.environ, {'GH_TOKEN': 'fixture-default-token', 'OASIS7_LOOP_READ_TOKEN': 'fixture-read-token'}, clear=True), patch.object(module.subprocess, 'run', return_value=response) as probe:
            environment = module.project_read_environment('fixture/repo')
            self.assertEqual(environment['GH_TOKEN'], 'fixture-read-token')
            self.assertNotIn('OASIS7_LOOP_READ_TOKEN', environment)
            self.assertEqual(module.os.environ['GH_TOKEN'], 'fixture-default-token')
            self.assertEqual(probe.call_args.kwargs['env'], environment)

    def invoke(self, body, head='b' * 40, history=None):
        responses = [json.dumps({'body': UID, 'head': {'sha': head}}), json.dumps([{'number': 1}]), json.dumps({'body': UID + '\n- pr_number: `2`\n' + body}), json.dumps(history or [])]
        with patch.object(module, 'run', side_effect=responses), patch('sys.argv', ['loop-ci.py', '--repository', 'fixture/repo', '--pr-number', '2', '--base', 'a' * 40, '--head', 'b' * 40]), patch('sys.stdout', new_callable=io.StringIO):
            return module.main()

    def test_live_legacy_passes(self):
        self.assertEqual(self.invoke(''), 0)

    def test_live_head_drift_blocks(self):
        self.assertEqual(self.invoke('', head='c' * 40), 2)

    def test_deleted_binding_cannot_become_legacy(self):
        self.assertEqual(self.invoke('', history=[[{'body': '<!-- oasis7-loop-binding-history -->'}]]), 2)

    def test_malformed_binding_cannot_pass_as_legacy(self):
        self.assertEqual(self.invoke('loop_binding_b64: malformed'), 2)

    def test_loop_without_effective_policy_fails_closed(self):
        encoded = base64.urlsafe_b64encode(json.dumps({'task_uid': UID, 'loop': 'code'}).encode()).decode()
        self.assertEqual(self.invoke('- loop_binding_b64: `' + encoded + '`'), 2)


if __name__ == '__main__': unittest.main()
