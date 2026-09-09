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
