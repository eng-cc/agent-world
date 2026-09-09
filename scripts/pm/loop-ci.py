#!/usr/bin/env python3
"""Independent PR loop gate: read live task, execute only effective helpers."""
import argparse
import base64
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


def run(*args):
    return subprocess.check_output(args, text=True).strip()


def project_read_environment(repository):
    """No runner gh login fallback: provision an explicit read credential."""
    token = os.environ.get('OASIS7_LOOP_READ_TOKEN')
    if not token:
        raise ValueError('loop CI activation requires OASIS7_LOOP_READ_TOKEN with selected Project read access; GITHUB_TOKEN is repository-scoped')
    environment = {**os.environ, 'GH_TOKEN': token}
    environment.pop('OASIS7_LOOP_READ_TOKEN', None)
    probe = subprocess.run(['gh', 'project', 'view', '1', '--owner', repository.split('/')[0], '--format', 'json'], env=environment, capture_output=True, text=True)
    if probe.returncode:
        raise ValueError('OASIS7_LOOP_READ_TOKEN cannot read canonical Project 1; verify read-only credential permissions and organization authorization')
    project = json.loads(probe.stdout)
    if not isinstance(project, dict) or not project.get('id') or project.get('number') != 1:
        raise ValueError('canonical Project credential probe identity mismatch')
    return environment


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo-root', type=Path, default=Path.cwd())
    parser.add_argument('--repository', required=True)
    parser.add_argument('--pr-number', required=True, type=int)
    parser.add_argument('--base', required=True)
    parser.add_argument('--head', required=True)
    args = parser.parse_args()
    try:
        pr = json.loads(run('gh', 'api', f'repos/{args.repository}/pulls/{args.pr_number}'))
        if pr['head']['sha'] != args.head: raise ValueError('live PR HEAD differs from CI source HEAD')
        uids = set(re.findall(r'task_[0-9a-f]{32}', pr.get('body') or ''))
        if len(uids) == 1:
            uid = next(iter(uids))
            hits = json.loads(run('gh', 'issue', 'list', '-R', args.repository, '--state', 'all', '--search', uid + ' in:body', '--json', 'number', '--limit', '5'))
            if len(hits) != 1: raise ValueError('live task Issue is ambiguous or missing')
            number = hits[0]['number']
        else:
            refs = set(re.findall(r'(?:Refs|Fixes|Closes)\s+#(\d+)', pr.get('body') or '', re.I))
            if uids or len(refs) != 1: raise ValueError('PR must identify exactly one canonical task Issue')
            number, uid = int(next(iter(refs))), None
        issue = json.loads(run('gh', 'api', f'repos/{args.repository}/issues/{number}'))
        body = issue.get('body', '')
        issue_uids = re.findall(r'^task_uid: (task_[0-9a-f]{32})$', body, re.MULTILINE)
        if uid is None:
            if len(issue_uids) != 1: raise ValueError('Issue UID missing')
            uid = issue_uids[0]
        if uid not in body: raise ValueError('Issue UID mismatch')
        bound_pr = re.findall(r'^- pr_number: `([0-9]+)`$', body, re.MULTILINE)
        if bound_pr != [str(args.pr_number)]:
            raise ValueError('live task Issue does not bind this PR number; refresh task PR identity')
        if 'loop_binding_b64:' not in body:
            pages = json.loads(run('gh', 'api', f'repos/{args.repository}/issues/{number}/comments', '--paginate', '--slurp'))
            if not isinstance(pages, list): raise ValueError('lineage readback unavailable')
            comments = [item for page in pages for item in (page if isinstance(page, list) else [page])]
            if any('oasis7-loop-binding-history' in str(item.get('body', '')) for item in comments if isinstance(item, dict)):
                raise ValueError('loop binding deleted after immutable history')
            print(json.dumps({'status': 'legacy', 'task_uid': uid}))
            return 0
        matches = re.findall(r'^- loop_binding_b64: `([^`]+)`$', body, re.MULTILINE)
        if len(matches) != 1: raise ValueError('malformed live binding')
        binding = json.loads(base64.b64decode(matches[0] + '=' * (-len(matches[0]) % 4), altchars=b'-_', validate=True))
        if binding.get('task_uid') != uid: raise ValueError('loop task UID mismatch')
        commit = binding.get('policy_commit', '')
        if not re.fullmatch(r'[0-9a-f]{40}', commit): raise ValueError('missing immutable effective policy')
        read_environment = project_read_environment(args.repository)
        subprocess.run(['git', '-C', str(args.repo_root), 'fetch', '--no-tags', 'origin', 'main:refs/remotes/origin/main'], check=True, capture_output=True)
        subprocess.run(['git', '-C', str(args.repo_root), 'merge-base', '--is-ancestor', commit, 'refs/remotes/origin/main'], check=True)
        with tempfile.TemporaryDirectory(prefix='oasis7-loop-tools-') as tmp:
            tool_root = Path(tmp) / 'tools'
            subprocess.run(['git', '-C', str(args.repo_root), 'worktree', 'add', '--detach', str(tool_root), commit], check=True, capture_output=True)
            try:
                # All Python code below is loaded from the verified effective commit.
                code = 'import json,sys; from pathlib import Path; from loop import validate_task; t,r,b,base,head,repo=sys.argv[1:]; b=json.loads(b); task={**b,"loop_binding":b,"repository":repo}; result=validate_task(Path(r),task,Path(t),base,head); print(json.dumps(result)); sys.exit(0 if result["status"]=="passed" else 2)'
                result = subprocess.run([sys.executable, '-c', code, str(tool_root), str(args.repo_root.resolve()), json.dumps(binding), args.base, args.head, args.repository], cwd=tool_root / 'scripts/pm', env=read_environment)
                return result.returncode
            finally:
                subprocess.run(['git', '-C', str(args.repo_root), 'worktree', 'remove', str(tool_root)], check=True, capture_output=True)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as exc:
        print(json.dumps({'status': 'blocked', 'blockers': [str(exc)]}))
        return 2


if __name__ == '__main__': raise SystemExit(main())
