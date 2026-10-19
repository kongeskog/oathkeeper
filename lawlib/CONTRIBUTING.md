# Contributing to lawlib

Contributions and reviews are open to everyone. Legal expertise is welcome; professional status is not a gate. Distinguish what you checked from what you assume. Never approve on another person's behalf.

1. Choose a bounded question and identify the source editions, forum and relevant dates.
2. Edit canonical `.oath` code. Include a coverage contract, source inventory, exclusions and known omissions. Keep missing evidence explicit.
3. Map every premise and conclusion to a source passage and an interpretation note. Explain exceptions, alternative readings, procedural limits and authority treatment. Changes in legal interpretation need their own rationale.
4. Include reproducible cases, boundaries, missing evidence, contrary evidence and scope failures. Label synthetic tests. Preserve the distinction between a judicial finding supplied as input and an independently established fact.
5. Regenerate JSON and review packets with the pinned software tools, then run `python tools/check.py`. Submit source and generated artifacts together.
6. Request community review of the exact artifact digest. Authors can explain their own work; they cannot label it an independent review.

A merge records a contribution; it does not certify legal correctness. Drafts remain usable as explicitly unreviewed models. A review records an individual's assessment of a named aspect and exact snapshot, not a universal approval of every legal use.

Source materials need recorded provenance and rights. Do not add private client records or secrets. Original contributions are Apache-2.0; source material keeps its own rights.
