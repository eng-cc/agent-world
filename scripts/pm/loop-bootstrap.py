#!/usr/bin/env python3
"""Manual bootstrap identity and fixed-base preflight; never starts a model."""
import argparse
import hashlib
import importlib.util
import json
import re
from pathlib import Path
import subprocess
import sys

def load(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), Path(__file__).with_name(name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args], text=True).strip()

def fetch_base(root):
    # Remote's advertised default, never the caller's current HEAD.
    advertised = git(root, 'ls-remote', '--symref', 'origin', 'HEAD')
    branches = [line.split()[1] for line in advertised.splitlines() if line.startswith('ref: refs/heads/')]
    if len(branches) != 1:
        raise ValueError('cannot establish remote default branch')
    # Pin the OID from the same advertisement as the default-branch identity.
    # FETCH_HEAD is shared with other requests and historical contract fetches.
    heads = [line.split()[0] for line in advertised.splitlines()
             if re.fullmatch(r'[0-9a-f]{40}(?:[0-9a-f]{24})?\s+HEAD', line)]
    if len(heads) != 1:
        raise ValueError('cannot establish exact remote default branch OID')
    oid = heads[0]
    git(root, 'fetch', '--no-write-fetch-head', '--no-tags', 'origin', oid)
    if git(root, 'rev-parse', oid + '^{commit}') != oid:
        raise ValueError('fetched default branch object does not match advertised OID')
    return oid

def prepare_request(journal_root, request, root):
    path = journal_root / (hashlib.sha256(request['request_key'].encode()).hexdigest() + '.json')
    store = load('workflow-durable-store')
    with store.locked_json(path, {}) as saved:
        if saved:
            if saved.get('request') != request:
                raise ValueError('manual bootstrap request identity drift; reuse original request or explicitly start another')
            return saved['base_oid']
        base = fetch_base(root)
        saved.update(request=request, base_oid=base)
        return base

def resume(args):
    task = load('github-project-task')
    live = task.github_issue_record(args.repository, args.task_uid)
    if not live:
        raise ValueError('selected existing task is not uniquely readable')
    target = Path(live.get('worktree_hint') or '').resolve()
    if not target.is_dir():
        raise ValueError('canonical worktree missing; selected task recovery required')
    root_common = Path(git(args.root, 'rev-parse', '--git-common-dir'))
    target_common = Path(git(target, 'rev-parse', '--git-common-dir'))
    if (args.root / root_common).resolve() != (target / target_common).resolve():
        raise ValueError('existing task belongs to another clone/common-dir')
    if live.get('status') in {'done', 'deferred'}:
        raise ValueError('terminal task cannot be bootstrapped again')
    binding = live.get('loop_binding')
    if binding is not None:
        task.validate_loop_inputs(target, binding, args.repository, 'in_flight')
    if args.binding:
        supplied = json.loads(Path(args.binding).read_text())
        if supplied != binding:
            raise ValueError('existing task frozen binding mismatch; explicit epoch migration required')
    if args.loop and (binding or {}).get('loop') != args.loop:
        raise ValueError('existing task loop mismatch; no implicit legacy migration')
    command = [sys.executable, str(Path(__file__).with_name('github-project-task.py')), 'refresh-task',
               str(target), '--repo', args.repository, '--task-uid', args.task_uid, '--json']
    subprocess.run(command, check=True, stdout=subprocess.PIPE)
    snapshot = json.loads((target / '.pm/scratch' / args.task_uid / 'bootstrap-task-snapshot.json').read_text())
    subprocess.run([sys.executable, str(Path(__file__).with_name('bootstrap-task-snapshot.py')), 'validate-epoch-identity',
                    '--repo-root', str(target), '--task-uid', args.task_uid,
                    '--request-identity', snapshot['request']['identity']], check=True, stdout=subprocess.PIPE)
    print(json.dumps({'mode': 'resume_existing_task', 'worktree_path': str(target),
                      'branch': git(target, 'symbolic-ref', '--short', 'HEAD'),
                      'pm': {'task_uid': args.task_uid, 'task_path': live['issue_url'], 'loop_binding': binding}}))

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['prepare', 'resume'])
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--binding')
    parser.add_argument('--loop', choices=['product', 'system', 'code'])
    parser.add_argument('--request-key')
    parser.add_argument('--manual-request-ref', required=True)
    parser.add_argument('--task-uid')
    parser.add_argument('--worktree')
    parser.add_argument('--branch')
    parser.add_argument('--repository', default='eng-cc/oasis7')
    args = parser.parse_args()
    if not args.manual_request_ref.strip():
        raise ValueError('explicit manual request reference is required')
    if args.command == 'resume':
        if not args.task_uid: raise ValueError('selected task UID required')
        resume(args)
        return
    if not all((args.binding, args.loop, args.request_key, args.worktree, args.branch)):
        raise ValueError('loop bootstrap needs binding, loop, request key, worktree and branch')
    binding = json.loads(Path(args.binding).read_text())
    if not isinstance(binding, dict): raise ValueError('binding must be an object')
    if binding['loop'] != args.loop or binding['request_key'] != args.request_key or binding['manual_request_ref'] != args.manual_request_ref:
        raise ValueError('manual request differs from frozen binding')
    common = (args.root / Path(git(args.root, 'rev-parse', '--git-common-dir'))).resolve()
    request = dict(request_key=args.request_key, binding=binding, worktree=str(Path(args.worktree).resolve()), branch=args.branch)
    journal = common / 'oasis7-loop-bootstrap-requests' / (hashlib.sha256(args.request_key.encode()).hexdigest() + '.json')
    load('github-project-task').validate_loop_inputs(args.root, binding, args.repository, 'in_flight' if journal.exists() else 'new_tasks')
    print(prepare_request(common / 'oasis7-loop-bootstrap-requests', request, args.root))

if __name__ == '__main__':
    try: main()
    except (ValueError, OSError, subprocess.CalledProcessError) as exc:
        print(f'loop-bootstrap: {exc}', file=sys.stderr)
        raise SystemExit(1)
