run: build
  ./target/bootstrap/freight build
  ./target/debug/freight help
build:
  rm -rf target
  mkdir -p target/bootstrap
  # Build crate dependencies
  rustc src/lib.rs --edition 2021 --crate-type=lib --crate-name=freight \
    --out-dir=target/bootstrap
  # Create the executable
  rustc src/main.rs --edition 2021 --crate-type=bin --crate-name=freight \
    --out-dir=target/bootstrap -L target/bootstrap --extern freight
