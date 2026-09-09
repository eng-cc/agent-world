#!/usr/bin/env python3
"""Contract eligibility must be live-authority derived, not caller approved."""
import copy
import hashlib
import importlib.util
from pathlib import Path
import tempfile
import subprocess
import unittest
from types import SimpleNamespace
from unittest.mock import patch

HERE = Path(__file__).resolve().parent

class ContractTests(unittest.TestCase):
    def test_default_obligation_uses_terminal_project_and_receipt_reader(self):
        calls = []
        def terminal(repo, uid, number):
            calls.append((repo, uid, number))
            return {'status': 'passed', 'blockers': []}
        uid = 'task_' + 'b' * 32
        with patch.dict('sys.modules', {'loop_terminal': SimpleNamespace(validate_terminal_delivery=terminal)}):
            self.assertTrue(self.api.GitHubAuthority(self.root).obligation({'task_uid': uid, 'issue_number': 11}))
        self.assertEqual(calls, [('eng-cc/oasis7', uid, 11)])

    def test_default_obligation_rejects_unfinished_terminal_reader(self):
        reader = SimpleNamespace(validate_terminal_delivery=lambda *args: {'status': 'blocked', 'blockers': ['finalizer receipt missing']})
        with patch.dict('sys.modules', {'loop_terminal': reader}):
            self.assertFalse(self.api.GitHubAuthority(self.root).obligation({'task_uid': 'task_' + 'b' * 32, 'issue_number': 11}))

    def setUp(self):
        spec = importlib.util.spec_from_file_location("loop_contracts", HERE / "loop_contracts.py")
        self.api = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.api)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test")
        (self.root / "spec.md").write_text("approved content")
        self.git("add", ".")
        self.git("commit", "-qm", "approved")
        self.source = self.git("rev-parse", "HEAD")
        self.git("commit", "--allow-empty", "-qm", "squash equivalent")
        self.merged = self.git("rev-parse", "HEAD")
        self.contract = dict(schema="oasis7.loop-contract/v1", contract_id="S", revision=1,
            owner_loop="system", source_head=self.source, merged_head=self.merged,
            approval_ref={"repository": "eng-cc/oasis7", "pr_number": 12},
            content_refs=[{"path":"spec.md", "sha256":"sha256:"+hashlib.sha256(b"approved content").hexdigest(), "clauses":["section-1"]}],
            upstream_contracts=[], scope=["pilot"], eligibility={"new_tasks":True,"in_flight":True,"release":True})
        self.ref = {"contract_id":"S", "revision":1,"contract_digest":self.api.contract_digest(self.contract),"publication_ref":{"issue_number":11,"comment_id":123},"consumed_clauses":["section-1"]}
        self.record = {"repository":"eng-cc/oasis7","issue_number":11,"comment_id":123,"author":"owner","permission":"admin","task_uid":"task_"+"b"*32,"contract":copy.deepcopy(self.contract),"pr":{"number":12,"merged":True,"head":self.source,"merge_commit":self.merged}}

    def git(self,*args):
        return subprocess.check_output(["git","-C",str(self.root),*args],text=True).strip()

    def check(self, purpose="in_flight"):
        return self.api.validate_contracts(self.root,self.root,{"input_contracts":[self.ref],"target_delivery":"pilot"},authority_reader=lambda ref:copy.deepcopy(self.record),purpose=purpose)

    def test_squash_equivalent_content_accepted(self):
        self.assertNotEqual(self.source,self.merged)
        self.assertEqual(self.check()["status"],"passed")

    def test_revoked_input_blocks_next_admission(self):
        self.record["contract"]["eligibility"]["in_flight"]=False
        self.assertEqual(self.check()["status"],"blocked")

    def test_new_task_and_release_eligibility_are_separate(self):
        self.record["contract"]["eligibility"]["new_tasks"]=False
        self.assertEqual(self.check()["status"],"passed")
        self.assertEqual(self.check("new_tasks")["status"],"blocked")

    def test_forged_local_approval_is_ignored(self):
        self.ref["approved"]=True
        self.record["permission"]="write"
        self.assertEqual(self.check()["status"],"blocked")

    def test_wrong_publication_or_revision_rejected(self):
        self.record["comment_id"]=999
        self.assertEqual(self.check()["status"],"blocked")
        self.record["comment_id"]=123
        self.record["contract"]["revision"]=2
        self.assertEqual(self.check()["status"],"blocked")

    def test_changed_published_content_rejected(self):
        (self.root/"spec.md").write_text("unapproved")
        self.git("add",".")
        self.git("commit","-qm","drift")
        changed=self.git("rev-parse","HEAD")
        self.record["pr"]["merge_commit"]=changed
        self.record["contract"]["merged_head"]=changed
        self.ref["contract_digest"]=self.api.contract_digest(self.record["contract"])
        self.assertEqual(self.check()["status"],"blocked")

    def test_unknown_clause_and_delivery_rejected(self):
        self.ref["consumed_clauses"]=["missing"]
        self.assertEqual(self.check()["status"],"blocked")

    def test_unfinished_delivery_obligation_blocks_release(self):
        binding={"input_contracts":[self.ref],"target_delivery":"pilot", "delivery_obligations":[{"id":"manual", "task_uid":"task_"+"c"*32}]}
        result=self.api.validate_contracts(self.root,self.root,binding,authority_reader=lambda ref:copy.deepcopy(self.record),purpose="release")
        self.assertEqual(result["status"],"blocked")

    def test_live_reader_uses_server_identity_and_current_permission(self):
        reader=self.api.GitHubAuthority(self.root)
        payload={"marker":self.api.MARKER,"task_uid":self.record["task_uid"],"contract_digest":self.api.contract_digest(self.contract),"contract":self.contract}
        import json
        def api(path,body=None,paginate=False):
            if path.endswith("/issues/11"):
                return {"number":11,"body":"<!-- oasis7-pm-task -->\ntask_uid: "+self.record["task_uid"]}
            if path.endswith("/issues/comments/123"):
                return {"id":123,"issue_url":"https://api.github.com/repos/eng-cc/oasis7/issues/11","body":json.dumps(payload),"user":{"login":"owner"}}
            if path.endswith("/collaborators/owner/permission"):
                return {"permission":"admin"}
            if path.endswith("/pulls/12"):
                return {"number":12,"merged":True,"head":{"sha":self.source},"merge_commit_sha":self.merged,"base":{"repo":{"full_name":"eng-cc/oasis7"}}}
            self.fail(path)
        reader.api=api
        self.assertEqual(reader(self.ref)["permission"],"admin")
        payload["task_uid"]="task_"+"d"*32
        with self.assertRaises(ValueError):
            reader(self.ref)

    def test_publish_retry_reuses_exact_server_comment(self):
        reader=self.api.GitHubAuthority(self.root)
        import json
        payload={"marker":self.api.MARKER,"task_uid":self.record["task_uid"],"contract_digest":self.api.contract_digest(self.contract),"contract":self.contract}
        posts=[]
        def api(path,body=None,paginate=False):
            if path.endswith("/issues/11"):
                return {"number":11,"body":"<!-- oasis7-pm-task -->\ntask_uid: "+self.record["task_uid"]}
            if path=="user":
                return {"login":"owner"}
            if path.endswith("/permission"):
                return {"permission":"admin"}
            if paginate:
                return [[{"id":123,"body":json.dumps(payload)}]]
            posts.append(body)
            self.fail("unexpected write on reconciled retry")
        reader.api=api
        self.assertEqual(reader.publish({"issue_number":11,"task_uid":self.record["task_uid"]},self.contract),{"issue_number":11,"comment_id":123})
        self.assertEqual(posts,[])

    def test_same_digest_does_not_authorize_substituted_clauses(self):
        self.ref["consumed_clauses"]=["section-1","new-unapproved"]
        self.assertEqual(self.check()["status"],"blocked")

if __name__=="__main__":
    unittest.main()
