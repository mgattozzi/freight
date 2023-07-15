run: build
  ./target/bootstrap_stage0/freight_stage0
  ./target/bootstrap_stage1/freight_stage1
build:
  mkdir -p target/bootstrap_stage0
  # Build crate dependencies
  rustc src/lib.rs --edition 2021 --crate-type=lib --crate-name=freight \
    --out-dir=target/bootstrap_stage0
  # Create the executable
  rustc src/main.rs --edition 2021 --crate-type=bin --crate-name=freight_stage0 \
    --out-dir=target/bootstrap_stage0 -L target/bootstrap_stage0 --extern freight
