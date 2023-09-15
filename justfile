run: build
  ./target/debug/freight run help
doc: build
  ./target/debug/freight doc
build:
  rm -rf target
  mkdir -p target/bootstrap
  # Build crate dependencies
  rustc src/lib.rs --edition 2021 --crate-type=lib --crate-name=freight \
    --out-dir=target/bootstrap
  # Create the executable
  rustc src/main.rs --edition 2021 --crate-type=bin --crate-name=freight \
    --out-dir=target/bootstrap -L target/bootstrap --extern freight
  ./target/bootstrap/freight build
test: build
  mkdir -p target/test
  # Test that we can pass args to the tests
  ./target/debug/freight test ignored-arg -- --list
  # Actually run the tests
  ./target/debug/freight test
