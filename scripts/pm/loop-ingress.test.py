"""Effective ingress regression: candidate wrappers cannot replace base checks."""
from pathlib import Path
import subprocess
import tempfile
import unittest


class IngressTests(unittest.TestCase):
    def test_workflow_selects_base_script_and_candidate_stub_is_ignored(self):
        root = Path(__file__).resolve().parents[2]
        workflow = (root / '.github/workflows/rust.yml').read_text()
        self.assertIn('git show "${base_ref}:scripts/pm/loop-ci.py"', workflow)
        self.assertNotIn('python3 scripts/pm/loop-ci.py', workflow)
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            git = lambda *args: subprocess.check_output(['git', '-C', str(repo), *args], text=True).strip()
            git('init', '-q')
            git('config', 'user.name', 'Fixture')
            git('config', 'user.email', 'fixture@example.invalid')
            helper = repo / 'scripts/pm/loop-ci.py'
            helper.parent.mkdir(parents=True)
            helper.write_text('raise SystemExit(2)\n')
            git('add', '.')
            git('commit', '-qm', 'effective gate')
            base = git('rev-parse', 'HEAD')
            helper.write_text('raise SystemExit(0)\n')
            selected = repo / 'effective.py'
            selected.write_text(git('show', base + ':scripts/pm/loop-ci.py'))
            self.assertEqual(subprocess.run(['python3', str(selected)]).returncode, 2)

    def test_prepare_does_not_import_candidate_gate(self):
        script = (Path(__file__).resolve().parents[1] / 'prepare-task-pr.sh').read_text()
        self.assertNotIn('from loop_gate import admission', script)
        self.assertIn("'trusted helper bytes mismatch'", script)

    def test_read_credential_is_scoped_to_trusted_admission_step(self):
        workflow = (Path(__file__).resolve().parents[2] / '.github/workflows/rust.yml').read_text()
        admission, planner = workflow.split('      - id: scope', 1)
        self.assertIn('OASIS7_LOOP_READ_TOKEN: ${{ secrets.OASIS7_LOOP_READ_TOKEN }}', admission)
        self.assertNotIn('OASIS7_LOOP_READ_TOKEN', planner)
        self.assertIn('issues: read', admission)
        self.assertIn('pull-requests: read', admission)


if __name__ == '__main__': unittest.main()
