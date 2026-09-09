#!/usr/bin/env python3
"""Production receipts require fresh local admission, independently of CI."""
import importlib.util
from pathlib import Path
import unittest
import base64
import json
import tempfile
import subprocess
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('pr_gate', Path(__file__).with_name('pr-lifecycle-gate.py'))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class ProductionLoopTests(unittest.TestCase):
    def run_gate(self, admission=None, fresh=None, ready=True):
        data = {'number': 12, 'repository': 'owner/repo', 'baseRefOid': 'a' * 40, 'headRefOid': 'b' * 40}
        def decide(data, admin, *, evidence_mode):
            result = {'ready_for_merge': ready, 'status': 'ready' if ready else 'held', 'blockers': [] if ready else ['hold']}
            if ready and evidence_mode == 'production': result['readiness_receipt'] = {'head_oid': data['headRefOid']}
            return result
        with patch.object(gate, 'decision', side_effect=decide), patch.object(gate, 'local_loop_admission', side_effect=admission) as check, patch.object(gate, 'read_pr_identity', return_value=fresh or data):
            result = gate.production_decision(data, False, Path('/canonical'), 'task_uid', None)
        return result, check

    def test_revoked_contract_cannot_mint_receipt(self):
        result, _ = self.run_gate(admission=ValueError('contract revoked'))
        self.assertFalse(result['ready_for_merge'])
        self.assertNotIn('readiness_receipt', result)
        self.assertIn('contract revoked', ' '.join(result['blockers']))

    def test_failed_authority_cannot_mint_receipt(self):
        result, _ = self.run_gate(admission=ValueError('effective tool root unavailable'))
        self.assertFalse(result['ready_for_merge'])
        self.assertNotIn('readiness_receipt', result)

    def test_head_or_base_drift_cannot_mint_receipt(self):
        for key in ('headRefOid', 'baseRefOid'):
            fresh = {'number': 12, 'baseRefOid': 'a' * 40, 'headRefOid': 'b' * 40, key: 'c' * 40}
            result, _ = self.run_gate(fresh=fresh)
            self.assertFalse(result['ready_for_merge'])
            self.assertNotIn('readiness_receipt', result)

    def test_admission_receives_exact_pr_identity(self):
        result, check = self.run_gate()
        self.assertIn('readiness_receipt', result)
        check.assert_called_once_with(Path('/canonical'), 'task_uid', 'a' * 40, 'b' * 40, None)

    def test_hold_preserved_without_admission(self):
        result, check = self.run_gate(ready=False)
        self.assertEqual(result['status'], 'held')
        check.assert_not_called()
        self.assertNotIn('readiness_receipt', result)


class TrustedIngressTests(unittest.TestCase):
    def test_candidate_policy_commit_cannot_execute(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / '.pm/github-project-sync').mkdir(parents=True)
            binding = {'policy_commit': 'c' * 40}
            task = {'repository': 'owner/repo', 'issue_number': 3, 'loop_binding': binding}
            (root / '.pm/github-project-sync/tasks.json').write_text(json.dumps({'tasks': {'uid': task}}))
            encoded = base64.urlsafe_b64encode(json.dumps(binding).encode()).decode()
            issue = json.dumps({'body': f'- loop_binding_b64: `{encoded}`'})
            with patch.object(gate.subprocess, 'check_output', side_effect=[issue, 'c' * 40, '/common', '/common']), patch.object(gate.subprocess, 'run', side_effect=[None, subprocess.CalledProcessError(1, ['git', 'merge-base'])]) as execute:
                with self.assertRaises(subprocess.CalledProcessError):
                    gate.local_loop_admission(root, 'uid', 'a' * 40, 'b' * 40, root)
                self.assertEqual(execute.call_count, 2)
                self.assertTrue(all(call.args[0][0] == 'git' for call in execute.call_args_list))

    def test_tampered_helper_cannot_execute(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / '.pm/github-project-sync').mkdir(parents=True)
            helper = root / 'scripts/pm/loop-local-gate.py'
            helper.parent.mkdir(parents=True)
            helper.write_text('candidate code')
            binding = {'policy_commit': 'c' * 40}
            task = {'repository': 'owner/repo', 'issue_number': 3, 'loop_binding': binding}
            (root / '.pm/github-project-sync/tasks.json').write_text(json.dumps({'tasks': {'uid': task}}))
            encoded = base64.urlsafe_b64encode(json.dumps(binding).encode()).decode()
            issue = json.dumps({'body': f'- loop_binding_b64: `{encoded}`'})
            with patch.object(gate.subprocess, 'check_output', side_effect=[issue, 'c' * 40, '/common', '/common', b'trusted code']), patch.object(gate.subprocess, 'run') as execute:
                with self.assertRaisesRegex(ValueError, 'bytes differ'):
                    gate.local_loop_admission(root, 'uid', 'a' * 40, 'b' * 40, root)
                self.assertTrue(all(call.args[0][0] == 'git' for call in execute.call_args_list))

    def test_canonical_head_drift_cannot_execute(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / '.pm/github-project-sync').mkdir(parents=True)
            helper = root / 'scripts/pm/loop-local-gate.py'
            helper.parent.mkdir(parents=True)
            helper.write_text('trusted code')
            binding = {'policy_commit': 'c' * 40}
            task = {'repository': 'owner/repo', 'issue_number': 3, 'loop_binding': binding}
            (root / '.pm/github-project-sync/tasks.json').write_text(json.dumps({'tasks': {'uid': task}}))
            encoded = base64.urlsafe_b64encode(json.dumps(binding).encode()).decode()
            issue = json.dumps({'body': f'- loop_binding_b64: `{encoded}`'})
            with patch.object(gate.subprocess, 'check_output', side_effect=[issue, 'c' * 40, '/common', '/common', b'trusted code', 'd' * 40]), patch.object(gate.subprocess, 'run') as execute:
                with self.assertRaisesRegex(ValueError, 'worktree HEAD'):
                    gate.local_loop_admission(root, 'uid', 'a' * 40, 'b' * 40, root)
                self.assertTrue(all(call.args[0][0] == 'git' for call in execute.call_args_list))


if __name__ == '__main__': unittest.main()
