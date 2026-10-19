# Verifiable rewards

The kernel can verify derivations under a frozen task. This is useful infrastructure for proof construction, not a reward oracle for unrestricted legal correctness.

## Four separate tasks

1. Formalization translates authoritative text into rules. It requires source review and cannot be scored solely by whether a proof checks.
2. Evidence assessment decides which factual premises are justified. Proof validity does not validate the evidence.
3. Oracle judgment resolves an agreed subjective question. Protocol compliance is different from correctness or calibrated confidence.
4. Proof construction finds a derivation using a host-controlled context. This is the current machine-verifiable training task.

## Freeze the task

The host fixes rules, sources, facts, times, scope, oracle agreements and responses, and the goal. The model submits only proof steps and, when required by the host, scope-condition proofs. `examples/evaluate_proof.py` calls the shared Rust frozen-task API. It rejects replacement contexts, pins the entire task and policy, and checks host input before evaluating a candidate. The expected task digest must be stored by the evaluator, separately from model submissions. A reference proof is optional. If supplied, it must itself verify.

The reward is binary: 1 for a checked derivation, 0 for a rejected candidate. There is no oracle-count penalty. Penalizing honest subjective dependencies would encourage relabeling them as facts. Hosts may explicitly exclude oracle-dependent tasks as a task-selection policy, not as a correctness score.

## Deterministic baseline before RL

`tools/benchmark.py` removes reference proofs, runs deterministic search, and replays its certificates. The 11 tasks include five historical paths from one matter, two unresolved real-record contexts, and four explicitly synthetic duration boundaries. All contexts retain interpretation assumptions. Historical reconstructions contain judicial findings and are not held-out predictions.

No RL model has been trained. Timings are measurements of this run, not general performance guarantees. Legal accuracy, expert agreement, critical omissions, and reviewer time remain unmeasured.

## Evaluation design

Split future datasets by litigation matter, source family, and amendment lineage, rather than random proof files. Freeze editions and clocks. Keep held-out cases distinct from cited findings and oracle training material. Compare deterministic proof search and an independently reviewed source-based checklist before claiming an LLM benefit. Evaluate unsupported conclusions, missed counterarguments, evidence leakage, and search truncation separately.

## Coverage and trust policy

Use `--require-coverage` for tasks intended to prove the declared scope as well as the main claim. A plain derivation task does not check scope. Host-owned rules, evidence, interpretation selection, oracle procedures/responses and policy must be inaccessible to candidate mutation. Do not derive the expected digest from a candidate-supplied task.

Evidence receipts can be authenticated before freezing a task, with a host trust store and externally pinned journal head. Authentication and factual truth remain distinct. The dataset grouping manifest in lawlib prevents overlapping matters, source families and amendment lineages across splits. The current held-out split is empty; legal generalization remains unmeasured.
