"""Bounded adapter admission and external-index smoke; no source edits."""
import json
from pathlib import Path
import subprocess

NODE = '/data/CoordExp/codex-tools/web-codex/dependencies/codegraph/node_modules/@colbymchenry/codegraph-linux-x64/node'
ADAPTER = str(Path(__file__).with_name('codegraph-adapter.mjs'))
ROOTS = [
    '/data/ms-swift/swift',
    '/root/miniconda3/envs/ms/lib/python3.12/site-packages/transformers',
    '/root/miniconda3/envs/ms/lib/python3.12/site-packages/vllm',
]

def call(root):
    request = {'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call',
               'params': {'name': 'codegraph_status', 'arguments': {'projectPath': root}}}
    result = subprocess.run([NODE, ADAPTER], input=json.dumps(request) + '\n',
                            text=True, capture_output=True, check=True, timeout=120)
    return json.loads(result.stdout)['result']

if __name__ == '__main__':
    for root in ROOTS:
        result = call(root)
        assert not result['isError'], result
        print(json.dumps({'root': root, 'status': result}))
    for root in ['/root', '/data/ms-swift', str(Path(ROOTS[1]).parent), ROOTS[1] + '/models']:
        result = call(root)
        assert result['isError'] and 'explicitly allowed external source root' in result['content'][0]['text'], result
    assert not call('/data/CoordExp')['isError']
    print('PASS: three external roots admitted; parents/subdirectories rejected; CoordExp preserved')
