watch +args='test':
  cargo watch --clear --exec '{{args}}'

ci: forbid
  cargo lclippy --workspace --all-targets -- --deny warnings
  cargo fmt --all -- --check
  cargo ltest --workspace

forbid:
  ./bin/forbid
