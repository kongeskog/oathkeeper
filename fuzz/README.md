Run from the Oathkeeper repository with nightly Rust and cargo-fuzz installed:

```sh
cargo +nightly fuzz run parser -- -max_len=65536 -max_total_time=60
cargo +nightly fuzz run certificate_reader -- -max_len=262144 -max_total_time=60
```

Seed parser campaigns with ordinary .oath modules; seed certificate campaigns with
certificates emitted for the fixed fuzz.answer program in the target. Readers enforce
size/depth limits before typed deserialization. The ordinary test suite also runs
512 deterministic mutations each of source text and a valid certificate, including
checking any mutated certificate that still parses. These bounded smoke tests do
not establish the absence of parser or verifier vulnerabilities.
