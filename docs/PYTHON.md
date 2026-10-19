# Python API

The native extension calls the Rust core. Functions return JSON strings, except `check_context`, which returns `None` on success. They raise `ValueError` on rejected inputs. Oathkeeper 0.4 removes score fields from all reports.

## Install

```sh
python3 tools/lawlib_package.py sync
python -m venv .venv
. .venv/bin/activate
pip install 'maturin>=1.7,<2'
maturin develop --locked
```

## verify_file(path)

```python
import json
import oathkeeper
report = json.loads(oathkeeper.verify_file(
    '.deps/lawlib/us/wa/seattle/cases/long-excessive-fines.case.oath'))
assert report['status'] == 'derivation_verified'
print(report['assurance'])
```

## analyze_file(path)

Runs bounded supporting and opposing argument search on a 0.4 case. It discards the supplied reference proof, independently checks returned certificates, exposes gaps and limits, and computes input sensitivity. The four statuses are supported, opposed, both, and neither.

```python
analysis = json.loads(oathkeeper.analyze_file(
    '.deps/lawlib/us/wa/seattle/cases/long-parking-evidence-review.case.oath'))
print(analysis['status'], analysis['open_requirements'])
```

## analyze(bundle_json, options_json=None)

Accepts serialized JSON. Options include max_groundings, max_nodes, max_depth, max_arguments, sensitivity, and max_sensitivity. A truncated search never establishes absence of an opposing argument. Exact defaults and hard limits are in the [analysis implementation](../crates/oathkeeper-core/src/analysis.rs).

## check_context(bundle_json)

Validates the host-controlled context independently of any candidate proof. Returns a JSON status. It does not judge the merits or authenticate evidence.

## compile_file(path)

Returns the typed module, optional case, source map, import lock, and compilation snapshot digest. It is not a successful proof report.

## verify(bundle_text, format)

Checks a supplied JSON or legacy TOML bundle. Successful reports remain conditional on all declared inputs. Legacy input formats remain readable; their output report shape has changed.

## verify_case(module_json, case_json)

Composes the frozen module with a case and checks the candidate proof. Cases cannot replace library rules or declarations. `.oath` authoring normally uses verify_file instead.

## compare_arguments(left_json, right_json)

Checks two supplied opposing proofs in the same context and exposes their conflict without deciding a legal winner. Use analyze for bounded discovery of arguments.

See the [binding implementation](../crates/oathkeeper-python/src/lib.rs) and [training guidance](TRAINING.md).

## assess_backdoor_roth(input_json)

Runs the bounded 2025 federal IRA model in Rust, returning a JSON assessment with an independently replayed certificate. Money uses integer USD cents. Null is unknown; unsupported scope has no calculated amount. See the [tax module guide](https://github.com/kongeskog/oathkeeper-lawlib/blob/main/us/tax/README.md) for the complete schema and exclusions.

```python
result = json.loads(oathkeeper.assess_backdoor_roth(scenario_json))
if result['certificate'] is not None:
    report = json.loads(oathkeeper.verify(json.dumps(result['certificate']), 'json'))
```
