# Implementation tracking

The accepted technical plan is the implementation contract. Repositories are independent; source snapshots live outside both repositories unless explicitly included as fixtures.

- [x] Core values, IR, protocol, exact primitives, canonical hashing
- [x] Parser, resolution, typing, generics, imports, metadata
- [x] Deterministic execution and knowledge propagation
- [x] Independent derivation checker
- [x] Packages, locks, source auditing, test harness, CLI
- [x] Standard library in Oath
- [x] Legal models and household, Constitution, IRA packages
- [x] Conformance, adversarial, property, portability tests and fuzz targets
- [x] Documentation, schemas, CI configuration, pinned integration and initial commits

See VALIDATION.md for executed checks and docs/RELEASE.md for limits and future
maintenance work. Hosted CI and instrumented nightly fuzz campaigns require
the repositories' eventual hosting configuration.

No legal-review status may be upgraded from Unreviewed merely because tests pass.
