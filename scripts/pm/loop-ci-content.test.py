"""Repository-only hosted checks never claim local live admission."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from loop_ci_content import validate_ci_content
from loop_contracts import MARKER, contract_digest


class ContentTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        shutil.copytree(Path(__file__).parent, self.root / 'scripts/pm', ignore=shutil.ignore_patterns('__pycache__'))
        (self.root / '.gitignore').write_text('__pycache__/\n')
        self.spec = self.root / 'doc/engineering/spec.md'; self.spec.parent.mkdir(parents=True); self.spec.write_text('approved')
        self.git('init', '-q'); self.git('config', 'user.name', 'Fixture'); self.git('config', 'user.email', 'fixture@example.invalid')
        self.git('add', '.'); self.git('commit', '-qm', 'effective')
        self.base = self.git('rev-parse', 'HEAD'); self.git('update-ref', 'refs/remotes/origin/main', self.base)
        self.contract = dict(schema='oasis7.loop-contract/v1', contract_id='S', revision=1, owner_loop='system', source_head=self.base, merged_head=self.base, approval_ref={'repository':'eng-cc/oasis7','pr_number':2}, content_refs=[{'path':'doc/engineering/spec.md','sha256':'sha256:'+hashlib.sha256(b'approved').hexdigest(),'clauses':['a']}], upstream_contracts=[], scope=['pilot'], eligibility={'new_tasks':False,'in_flight':False,'release':False})
        self.reference = dict(contract_id='S',revision=1,contract_digest=contract_digest(self.contract),publication_ref={'issue_number':1,'comment_id':3},consumed_clauses=['a'])
        self.binding = dict(schema='oasis7.loop-task/v1',task_uid='task_'+'a'*32,change_id='c',loop='system',owner_role='repository_health_engineer',bootstrap_epoch=1,manual_request_ref='user',request_key='r',write_scope=['doc/engineering/**'],out_of_scope=[],input_contracts=[self.reference],acceptance_refs=['a'],dependencies=[],target_delivery='pilot',policy_commit=self.base,policy_digest='sha256:'+hashlib.sha256((self.root/'scripts/pm/loop-policy.v1.json').read_bytes()).hexdigest())
        self.calls=[]

    def git(self,*args):
        return subprocess.check_output(['git','-C',str(self.root),*args],text=True).strip()

    def reader(self,repo,path):
        self.calls.append(path)
        if path=='issues/comments/3':
            return {'id':3,'issue_url':'https://api.github.com/repos/eng-cc/oasis7/issues/1','body':json.dumps({'marker':MARKER,'contract_digest':contract_digest(self.contract),'contract':self.contract})}
        if path=='pulls/2':
            return {'number':2,'merged':True,'head':{'sha':self.base},'merge_commit_sha':self.base,'base':{'repo':{'full_name':repo}}}
        self.fail('unexpected non-repository read: '+path)

    def check(self):
        return validate_ci_content(self.root,self.root,self.binding,self.base,self.base,'eng-cc/oasis7',self.reader)

    def test_content_pass_is_explicitly_not_live_eligibility_admission(self):
        result=self.check()
        self.assertEqual(result['status'],'passed',result)
        self.assertTrue(result['local_live_admission_required'])
        self.assertEqual(self.calls,['issues/comments/3','pulls/2'])

    def test_changed_publication_content_blocks(self):
        self.contract['content_refs'][0]['sha256']='sha256:'+'0'*64
        self.assertEqual(self.check()['status'],'blocked')

    def test_content_hash_mismatch_blocks_even_with_updated_reference(self):
        self.contract['content_refs'][0]['sha256']='sha256:'+'0'*64
        self.reference['contract_digest']=contract_digest(self.contract)
        self.assertEqual(self.check()['status'],'blocked')


if __name__=='__main__': unittest.main()
