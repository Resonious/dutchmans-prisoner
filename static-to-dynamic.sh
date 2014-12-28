mv Cargo.toml Static.Cargo.toml
mv Dynamic.Cargo.toml Cargo.toml

sed 's/extern crate dutchman_game;/\/\/ extern crate dutchman_game;/' <src/main.rs > src/tmp.main.rs
mv src/tmp.main.rs src/main.rs
sed 's/fn test_loop_fn()/fn static_test_loop_fn()/' <src/main.rs > src/tmp.main.rs
mv src/tmp.main.rs src/main.rs
sed 's/fn dynamic_test_loop_fn()/fn test_loop_fn()/' <src/main.rs > src/tmp.main.rs
mv src/tmp.main.rs src/main.rs
