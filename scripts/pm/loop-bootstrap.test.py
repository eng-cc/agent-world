#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

PATH = Path(__file__).with_name('loop-bootstrap.py')

class ManualBase(unittest.TestCase):
    def test_same_request_reuses_fetched_base_and_rejects_scope_drift(self):
        spec = importlib.util.spec_from_file_location('loop_bootstrap', PATH)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            request = {'request_key': 'one', 'binding': {'loop': 'code'}, 'worktree': '/task', 'branch': 'codex/task'}
            with patch.object(module, 'fetch_base', return_value='a' * 40) as fetch:
                self.assertEqual(module.prepare_request(root, request, root), 'a' * 40)
                self.assertEqual(module.prepare_request(root, request, root), 'a' * 40)
                fetch.assert_called_once()
                with self.assertRaises(ValueError):
                    module.prepare_request(root, dict(request, branch='different'), root)

if __name__ == '__main__': unittest.main()
