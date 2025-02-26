watch +args='test':
  cargo watch --clear --exec '{{args}}'

run:
  cargo run -- --job tmp/job.yaml

diff:
  cd tmp/just && git last
