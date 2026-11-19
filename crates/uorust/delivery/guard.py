"""Source-bound preflight for structural model-driven delivery campaigns."""
from pathlib import Path
import argparse, hashlib, json, shutil, subprocess, tomllib

class Refused(ValueError):
    def __init__(self, code, message):
        self.code = code
        super().__init__(f"{code}: {message}")

def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def identity(binding):
    return hashlib.sha256(json.dumps(binding, sort_keys=True, separators=(',', ':')).encode()).hexdigest()

def require(condition, code, message):
    if not condition:
        raise Refused(code, message)

def semantic_registry(document):
    """Exclude explanatory text and test vectors from the executable model."""
    fields = {'path', 'args', 'result', 'term', 'guard', 'update'}
    return {'models': [{k:v for k,v in row.items() if k in fields}
                       for row in document.get('model', [])],
            'aux': document.get('aux', {}), 'types': document.get('type', [])}

def audit_registry(policy, plan):
    if not policy.get('freeze_manual_semantic_registry'):
        return
    implementation = plan.get('implementation', {})
    try:
        root = Path(implementation['repository'])
        path = implementation['semantic_registry']
        baseline = subprocess.check_output(['git', 'show', implementation['baseline'] + ':' + path],
                                           cwd=root, stderr=subprocess.DEVNULL).decode()
        current = (root/path).read_text()
        same = semantic_registry(tomllib.loads(baseline)) == semantic_registry(tomllib.loads(current))
    except (KeyError, OSError, ValueError, subprocess.CalledProcessError):
        same = False
    require(same, 'DG_MANUAL_SEMANTICS',
            'manual semantic-registry patches are frozen; use the structural runtime/import mechanism')

def validate_report(report, source_scope=None):
    census = report.get('census', {})
    total, lifted = census.get('functions', 0), census.get('lifted', -1)
    require(report.get('sdk_verified') is True and total > 0 and 0 <= lifted <= total,
            'DG_EVIDENCE', 'successful source-bound SDK evidence and a nonempty registry are required')
    require(report.get('lost_lifted') == 0, 'DG_BASELINE_LOSS', 'baseline proofs must be preserved')
    if source_scope:
        require(total == source_scope['registered_functions']
                and report.get('source') == source_scope['canonical_source'],
                'DG_PARTIAL_SCOPE', 'the original registered source and function population must remain intact')
    if report.get('contract_complete'):
        require(lifted == total and report.get('release_gate_passed') is True
                and report.get('source_bound_adopted') is True,
                'DG_FALSE_COMPLETION', 'all functions, complete release acceptance and adoption are required')

def preflight(policy, plan, binding, history):
    require(policy.get('spec') == 'uorust/delivery-policy/1'
            and plan.get('spec') == 'uorust/delivery-plan/1', 'DG_SCHEMA', 'unknown delivery schema')
    require(plan.get('strategy') == policy['strategy']
            and plan.get('work_class') in policy['work_classes'],
            'DG_STRATEGY', 'use the registered structural model-runtime work classes')
    require(not plan.get('leaf_symbols') and not plan.get('per_operation_recipes'),
            'DG_LEAF_PLAN', 'individual function and operation repair queues are prohibited')
    require(plan.get('scope') == 'entire-registry' and not plan.get('excluded_functions')
            and set(policy['required_scopes']) <= set(plan.get('scopes', []))
            and plan.get('finish') == 'all-functions-formally-lifted',
            'DG_PARTIAL_SCOPE', 'the plan must cover the complete registry and structural dependency families')
    authority = plan.get('authority', {})
    try:
        bound = bool(authority.get('revision')) and digest(authority['path']) == authority['sha256']
    except (OSError, KeyError):
        bound = False
    require(bound, 'DG_AUTHORITY', 'the model-runtime authority must be pinned to actual local bytes')
    audit_registry(policy, plan)
    source_scope = plan.get('source_scope')
    if source_scope:
        require(binding.get('registered_source') == source_scope['rust_tree'],
                'DG_PARTIAL_SCOPE', 'consumer Rust sources cannot be changed to reduce the delivery scope')
    require(all(binding.get(k) for k in ['source', 'tool', 'mir', 'sdk', 'verdicts']),
            'DG_INPUT_BINDING', 'source, producer, MIR, SDK and verdict bindings are required')
    key = identity(binding)
    old = history.get(key)
    if old:
        require(old.get('status') != 'failed', 'DG_UNCHANGED_FAILURE',
                'unchanged failed work must be replanned from its recorded cause')
        if old.get('status') == 'verified':
            try:
                valid = bool(old.get('artifacts')) and digest(old['report']) == old['report_sha256']
                valid = valid and all(digest(path) == sha for path, sha in old['artifacts'].items())
            except (OSError, KeyError):
                valid = False
            require(valid, 'DG_RECEIPT', 'prior receipt or its bound artifacts changed')
            report = json.loads(Path(old['report']).read_text())
            validate_report(report, source_scope)
            return {'action': 'reuse', 'identity': key, 'report': old['report']}
    return {'action': 'execute', 'identity': key}

def record(binding, history_path, report_path, artifacts):
    """Preserve the actual report and artifact hashes; never construct a proof."""
    report = json.loads(Path(report_path).read_text())
    validate_report(report)
    key = identity(binding)
    history_path = Path(history_path)
    snapshot = history_path.parent / (history_path.stem + '-receipts') / key / 'report.json'
    snapshot.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(report_path, snapshot)
    history = json.loads(history_path.read_text()) if history_path.exists() else {}
    history[key] = {'status': 'verified', 'report': str(snapshot),
                    'report_sha256': digest(snapshot),
                    'artifacts': {str(path): digest(path) for path in artifacts}}
    pending = history_path.with_suffix('.pending.json')
    pending.write_text(json.dumps(history, indent=2) + '\n')
    pending.replace(history_path)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['policy', 'plan', 'binding', 'history']:
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    try:
        values = {key: json.loads(getattr(args, key).read_text()) for key in ['policy', 'plan', 'binding']}
        history = json.loads(args.history.read_text()) if args.history.exists() else {}
        print(json.dumps(preflight(**values, history=history), sort_keys=True))
    except (Refused, OSError, ValueError, KeyError) as error:
        parser.exit(2, str(error) + '\n')
if __name__ == '__main__':
    main()
