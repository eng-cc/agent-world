#!/usr/bin/env python3
"""Real shell + temporary Git + fake GitHub manual bootstrap/resume acceptance."""
import json
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
UID = 'task_' + '1' * 32
FAKE = r'''#!/usr/bin/env python3
import json, os, pathlib, sys
a=sys.argv[1:]; p=pathlib.Path(os.environ['FAKE_GH_STATE'])
s=json.loads(p.read_text()) if p.exists() else {'creates':0,'fields':{}}
def emit(v):
 p.write_text(json.dumps(s)); print(json.dumps(v) if not isinstance(v,str) else v)
def val(k): return a[a.index(k)+1]
url='https://github.com/eng-cc/oasis7/issues/1'
if a[:2]==['issue','list']: emit([{'number':1,'url':url,'title':'[PM] fixture','state':'OPEN'}] if s.get('body') else [])
elif a[:2] in (['issue','create'],['issue','edit']):
 s['body']=pathlib.Path(val('--body-file')).read_text()
 if a[1]=='create': s['creates']+=1
 emit(url)
elif a[:2]==['issue','view']: emit({'number':1,'url':url,'title':'[PM] fixture','state':'OPEN','stateReason':None,'body':s['body']})
elif a[:2]==['issue','comment']:
 comments=s.setdefault('comments',[]); comments.append({'id':len(comments)+1,'body':pathlib.Path(val('--body-file')).read_text()}); emit(url+'#issuecomment-'+str(len(comments)))
elif a[:2]==['api','repos/eng-cc/oasis7/issues/1/comments']: emit([s.get('comments',[])])
elif a[:2]==['api','repos/eng-cc/oasis7/issues/1']: emit({'number':1,'body':s['body'],'url':url})
elif a[:1]==['api'] and a[1].startswith('repos/eng-cc/oasis7/issues/comments/'): emit(s['comments'][int(a[1].rsplit('/',1)[1])-1])
elif a[:2]==['project','view']: emit({'id':'P','number':1})
elif a[:2]==['project','item-add']: emit({'id':'I'})
elif a[:2]==['project','field-list']:
 choices={'Status':['Todo','In Progress'],'PM Status':['candidate','committed'],'Workflow Phase':['bootstrap','execution'], 'Owner Role':['repository_health_engineer'], 'Module':['engineering'],'Priority':['P2'],'Test Tier Required':['n/a'],'Loop':['product','system','code']}
 fields=[{'id':n,'name':n,'type':'ProjectV2SingleSelectField','options':[{'id':v,'name':v} for v in values]} for n,values in choices.items()]
 fields += [{'id':n,'name':n,'type':'ProjectV2Field'} for n in ['Task UID','Canonical Worktree','Change ID','Blocked Reason','PR','Last PM Update']]
 emit({'fields':fields})
elif a[:2]==['project','item-edit']:
 s['fields'][val('--field-id')]=val('--text') if '--text' in a else val('--single-select-option-id'); emit({})
elif a[:2]==['api','graphql']:
 node={'id':'I','project':{'id':'P','number':1,'owner':{'login':'eng-cc'}},'fieldValues':{'pageInfo':{'hasNextPage':False},'nodes':[{'text':v,'field':{'name':k}} for k,v in s['fields'].items()]}}
 emit({'data':{'nodes':[node]}})
else: raise SystemExit('unsupported fake gh '+repr(a))
'''

class BootstrapEndToEnd(unittest.TestCase):
    def test_full_flags_and_explicit_resume_reuse_task(self):
        with tempfile.TemporaryDirectory() as directory:
            temp = Path(directory)
            root = temp / 'repo'
            (root / 'scripts').mkdir(parents=True)
            shutil.copytree(ROOT / 'scripts/pm', root / 'scripts/pm', ignore=shutil.ignore_patterns('__pycache__'))
            for name in ['new-task-worktree.sh', 'worktree-harness-lib.sh']:
                shutil.copy2(ROOT / 'scripts' / name, root / 'scripts' / name)
            (root / '.gitignore').write_text('.pm/\ntarget\n__pycache__/\n')
            cargo = root / 'scripts/cargo-dev.sh'
            cargo.write_text('#!/bin/sh\nprintf "%s\\n" "$TEST_SHARED_TARGET"\n')
            cargo.chmod(0o755)
            def git(*args): return subprocess.check_output(['git','-C',str(root),*args],text=True).strip()
            git('init','-q','-b','main'); git('config','user.email','test@example.invalid'); git('config','user.name','Test')
            git('add','.'); git('commit','-qm','fixture')
            origin = temp / 'origin.git'
            subprocess.run(['git','init','--bare','-q',str(origin)],check=True)
            git('remote','add','origin','https://github.com/eng-cc/oasis7.git')
            git('config',f'url.{origin}.insteadOf','https://github.com/eng-cc/oasis7.git')
            git('push','-q','origin','main'); git('symbolic-ref','refs/remotes/origin/HEAD','refs/remotes/origin/main')
            subprocess.run(['git','--git-dir',str(origin),'symbolic-ref','HEAD','refs/heads/main'],check=True)
            base = git('rev-parse','HEAD')
            binary = temp / 'bin'; binary.mkdir()
            gh = binary / 'gh'; gh.write_text(FAKE); gh.chmod(0o755)
            state = temp / 'github.json'
            env = dict(os.environ, PATH=str(binary)+os.pathsep+os.environ['PATH'], FAKE_GH_STATE=str(state),
                       TEST_SHARED_TARGET=str(temp/'target'), PYTHONDONTWRITEBYTECODE='1')
            binding = dict(schema='oasis7.loop-task/v1',task_uid=UID,change_id='change-fixture',loop='code',owner_role='repository_health_engineer',bootstrap_epoch=1,
                manual_request_ref='message:1',request_key='request:1',write_scope=['scripts/**'],out_of_scope=[],input_contracts=[],acceptance_refs=['M06'],dependencies=[],
                target_delivery='fixture',policy_digest='sha256:'+hashlib.sha256((root/'scripts/pm/loop-policy.v1.json').read_bytes()).hexdigest(),policy_commit=base)
            source = temp / 'binding.json'; source.write_text(json.dumps(binding))
            target = temp / 'task'
            command = ['bash','scripts/new-task-worktree.sh','engineering','loop-test','--path',str(target),'--branch','codex/loop-test',
                '--pm-owner-role','repository_health_engineer','--pm-title','fixture','--pm-source-ref','fixture','--pm-acceptance','M06',
                '--pm-loop','code','--pm-loop-binding',str(source),'--pm-request-key','request:1','--pm-manual-request-ref','message:1','--json']
            result = subprocess.run(command,cwd=root,env=env,text=True,capture_output=True)
            self.assertEqual(result.returncode,0,result.stderr)
            self.assertEqual(json.loads(state.read_text())['creates'],1)
            snapshot = json.loads((target/'.pm/scratch'/UID/'bootstrap-task-snapshot.json').read_text())
            self.assertEqual(snapshot['git']['base']['oid'],base)
            self.assertEqual(snapshot['task']['loop_binding'],binding)
            # A later remote default head must not silently rebase the same request.
            (root/'later.txt').write_text('unrelated upstream change\n')
            git('add','later.txt'); git('commit','-qm','later'); git('push','-q','origin','main')
            retry = subprocess.run(command,cwd=target,env=env,text=True,capture_output=True)
            self.assertEqual(retry.returncode,0,retry.stderr)
            self.assertEqual(json.loads(retry.stdout)['pm']['task_uid'],UID)
            resume = subprocess.run(['bash','scripts/new-task-worktree.sh','--pm-task-uid',UID,'--pm-loop','code','--pm-manual-request-ref','message:2'],cwd=target,env=env,text=True,capture_output=True)
            self.assertEqual(resume.returncode,0,resume.stderr)
            self.assertEqual(json.loads(resume.stdout)['pm']['task_uid'],UID)
            self.assertEqual(json.loads(state.read_text())['creates'],1)
            self.assertEqual(json.loads(state.read_text())['fields']['Loop'],'code')
            revised = dict(binding, bootstrap_epoch=2, write_scope=['scripts/pm/**'])
            source.write_text(json.dumps(revised))
            bind_command = ['python3',str(target/'scripts/pm/github-project-task.py'),'bind-loop',str(target),
                            '--task-uid',UID,'--loop-binding',str(source),'--manual-request-ref','message:3','--json']
            denied = subprocess.run(bind_command,cwd=root,env=env,text=True,capture_output=True)
            self.assertNotEqual(denied.returncode,0)
            migrated = subprocess.run(bind_command+['--migrate-epoch','2'],cwd=root,env=env,text=True,capture_output=True)
            self.assertEqual(migrated.returncode,0,migrated.stderr)
            snapshot = json.loads((target/'.pm/scratch'/UID/'bootstrap-task-snapshot.json').read_text())
            self.assertEqual(snapshot['task']['bootstrap_epoch'],2)
            self.assertTrue((target/'.pm/scratch'/UID/'bootstrap-task-snapshot.epoch-1.json').exists())
            trusted = temp / 'trusted'
            git('worktree','add','--detach',str(trusted),base)
            facade = ['python3',str(trusted/'scripts/pm/loop.py'),'bind','--repo-root',str(target),
                      '--tool-root',str(trusted),'--task-uid',UID,'--loop-binding',str(source),
                      '--manual-request-ref',revised['manual_request_ref'],'--json']
            # Run the real adapter to completion, then lose only its response to
            # the facade. Production code and all readbacks remain unchanged.
            inject = "import sys,subprocess; sys.path.insert(0,sys.argv[1]); import loop; original=subprocess.check_output\ndef lost(args,*a,**kw):\n result=original(args,*a,**kw)\n if 'bind-loop' in args: raise subprocess.CalledProcessError(1,args)\n return result\nsubprocess.check_output=lost; sys.argv=sys.argv[2:]; raise SystemExit(loop.main())"
            lost = subprocess.run(['python3','-c',inject,str(trusted/'scripts/pm'),*facade[1:]],cwd=trusted,env=env,text=True,capture_output=True)
            self.assertNotEqual(lost.returncode,0,lost.stdout+lost.stderr)
            recover = subprocess.run(['python3',str(trusted/'scripts/pm/loop.py'),'recover','--repo-root',str(target),'--tool-root',str(trusted),'--task-uid',UID,'--manual-request-ref','message:recover','--json'],cwd=trusted,env=env,text=True,capture_output=True)
            recovered = json.loads(recover.stdout)
            self.assertEqual(recovered.get('pending_actions'),[],recover.stdout+recover.stderr)
            bound = subprocess.run(facade,cwd=trusted,env=env,text=True,capture_output=True)
            self.assertEqual(bound.returncode,0,bound.stdout+bound.stderr)
            self.assertEqual(json.loads(bound.stdout)['status'],'bound')
            common = Path(git('rev-parse','--path-format=absolute','--git-common-dir'))
            journals = list((common/'oasis7-loop-recovery').glob('*.actions.jsonl'))
            self.assertEqual(len(journals),1)
            events = [json.loads(line) for line in journals[0].read_text().splitlines()]
            self.assertEqual(events[0]['kind'],'bind_loop')
            self.assertTrue(events[-1]['reconciled'])
            again = subprocess.run(facade,cwd=trusted,env=env,text=True,capture_output=True)
            self.assertEqual(again.returncode,0,again.stdout+again.stderr)
            self.assertEqual(json.loads(state.read_text())['creates'],1)
            local_gate = ['python3',str(trusted/'scripts/pm/loop-local-gate.py'),'--root',str(target),'--task-uid',UID,'--base',base,'--head',base,'--tool-root',str(trusted),'--json']
            admitted = subprocess.run(local_gate,cwd=trusted,env=env,text=True,capture_output=True)
            self.assertEqual(admitted.returncode,0,admitted.stdout+admitted.stderr)
            stale = local_gate.copy(); stale[stale.index('--head')+1]='0'*40
            rejected = subprocess.run(stale,cwd=trusted,env=env,text=True,capture_output=True)
            self.assertNotEqual(rejected.returncode,0)

if __name__=='__main__': unittest.main()
