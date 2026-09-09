#!/usr/bin/env python3
"""Behavior tests for immutable loop scope policy; temporary Git only."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

HERE = Path(__file__).resolve().parent

class PolicyTests(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location("loop_policy", HERE / "loop_policy.py")
        self.api = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.api)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test")
        self.write("scripts/pm/loop-policy.v1.json", (HERE / "loop-policy.v1.json").read_text())
        self.write("doc/product/a.md", "product")
        self.write("doc/engineering/a.md", "system")
        self.write("src/a.rs", "code")
        self.git("add", ".")
        self.git("commit", "-qm", "base")
        self.base = self.git("rev-parse", "HEAD")
        self.binding = dict(schema="oasis7.loop-task/v1", task_uid="task_" + "a"*32,
            change_id="change-test", loop="product", owner_role="gameplay_designer",
            bootstrap_epoch=1, manual_request_ref="user-message-1", request_key="request-1",
            write_scope=["doc/product/**"], out_of_scope=[], input_contracts=[],
            acceptance_refs=["acceptance-1"], dependencies=[], target_delivery="pilot",
            policy_digest="sha256:"+hashlib.sha256((self.root / "scripts/pm/loop-policy.v1.json").read_bytes()).hexdigest(),
            policy_commit=self.base)

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.root), *args], text=True).strip()

    def write(self, path, value):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(value)

    def check(self):
        self.git("add", ".")
        self.git("commit", "-qm", "candidate")
        return self.api.validate_scope(self.root, self.root, self.binding, self.base, self.git("rev-parse", "HEAD"))

    def test_product_allowed(self):
        self.write("doc/product/a.md", "changed")
        self.assertEqual(self.check()["status"], "passed")

    def test_cross_loop_rename_checks_both_endpoints(self):
        self.git("mv", "doc/engineering/a.md", "doc/product/stolen.md")
        self.assertEqual(self.check()["status"], "blocked")

    def test_cross_loop_deletion_rejected(self):
        (self.root / "src/a.rs").unlink()
        self.assertEqual(self.check()["status"], "blocked")

    def test_unknown_path_rejected(self):
        self.binding.update(loop="code", write_scope=["**"])
        self.write("unexpected/unknown.bin", "unknown")
        self.assertEqual(self.check()["status"], "blocked")

    def test_symlink_and_mode_change_rejected(self):
        (self.root / "doc/product/link.md").symlink_to("../../src/a.rs")
        self.assertEqual(self.check()["status"], "blocked")

    def test_document_executable_mode_rejected(self):
        (self.root / "doc/product/a.md").chmod(0o755)
        self.assertEqual(self.check()["status"], "blocked")

    def test_candidate_policy_cannot_reclassify_itself(self):
        self.binding.update(loop="code", write_scope=["**"])
        policy = json.loads((self.root / "scripts/pm/loop-policy.v1.json").read_text())
        policy["rules"] = [{"pattern": "**", "loop": "code"}]
        self.write("scripts/pm/loop-policy.v1.json", json.dumps(policy))
        self.write("doc/product/a.md", "candidate claims code")
        self.assertEqual(self.check()["status"], "blocked")

    def test_binding_missing_identity_and_cycle_rejected(self):
        self.binding["dependencies"] = [self.binding["task_uid"]]
        self.assertEqual(self.api.validate_binding(self.binding)["status"], "blocked")
        del self.binding["manual_request_ref"]
        self.assertEqual(self.api.validate_binding(self.binding)["status"], "blocked")

    def test_policy_digest_drift_rejected(self):
        self.binding["policy_digest"] = "sha256:" + "0"*64
        self.write("doc/product/a.md", "changed")
        self.assertEqual(self.check()["status"], "blocked")

    def test_out_of_scope_overrides_allow(self):
        self.binding["out_of_scope"] = ["doc/product/a.md"]
        self.write("doc/product/a.md", "changed")
        self.assertEqual(self.check()["status"], "blocked")

    def test_executable_source_under_doc_is_not_document(self):
        self.write("doc/product/executable.py", "print('side effect')")
        self.assertEqual(self.check()["status"], "blocked")

    def test_trusted_tool_checkout_and_main_anchor(self):
        self.git("update-ref","refs/remotes/origin/main",self.base)
        self.assertEqual(self.api.validate_tool_root(self.root,self.root,self.binding)["status"],"passed")
        self.write("scripts/pm/shadow.py","untracked executable authority")
        self.assertEqual(self.api.validate_tool_root(self.root,self.root,self.binding)["status"],"blocked")

    def test_candidate_commit_cannot_be_effective_tool(self):
        self.git("update-ref","refs/remotes/origin/main",self.base)
        self.write("doc/product/a.md","candidate")
        self.git("add",".")
        self.git("commit","-qm","candidate")
        self.binding["policy_commit"]=self.git("rev-parse","HEAD")
        self.assertEqual(self.api.validate_tool_root(self.root,self.root,self.binding)["status"],"blocked")

    def test_dependency_closure_cycle_rejected(self):
        import copy
        other=copy.deepcopy(self.binding)
        other["task_uid"]="task_"+"b"*32
        self.binding["dependencies"]=[other["task_uid"]]
        other["dependencies"]=[self.binding["task_uid"]]
        self.assertEqual(self.api.validate_dependencies(self.binding,{other["task_uid"]:other})["status"],"blocked")

if __name__ == "__main__":
    unittest.main()
