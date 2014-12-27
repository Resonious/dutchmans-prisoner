sed s/["staticlib"]/["dylib"]/ <Cargo.toml > tmp.Cargo.toml
mv tmp.Cargo.toml Cargo.toml
