# Community governance

Maintainers coordinate contribution review, releases, source provenance and the trusted-key registry. Participation does not require professional credentials. Maintainers must explain disputed decisions in public issues or pull requests.

The lifecycle is **draft → reviewed statements → contested or superseded**. There is no automatic "legally correct" label. Software checks, source review, interpretation review, coverage review and formalization review remain separate. Different people may review different aspects. Review counts are not a correctness score.

Every review names an exact artifact digest and a rationale. Signed events append to `community/reviews.json`; withdrawal and changes-requested events supersede that reviewer's earlier statement for the same aspect and artifact. New source or interpretation snapshots require new review. An old approval remains historical and cannot approve a new artifact.

A public key is added through a pull request with a principal identifier and an explanation of how the contributor demonstrated control of the key and account. The registry is a maintainer assertion about identity, not a credential or expertise certification. Keys have validity intervals and may be revoked. Anyone can run a different explicit trust store.

Competing readings should remain attributable alternatives with source rationales. Neither geography nor number of endorsements silently resolves a legal conflict. An issue should identify the affected source/rule, snapshot, counterexample and proposed change.

Releases are immutable tags. Software consumers opt into a new exact commit and replay both certificates and opposing-argument searches. Corrections identify affected versions and are published as new versions; history is retained for reproducibility.
