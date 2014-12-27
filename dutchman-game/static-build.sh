sed 's/["dylib"]/["staticlib"]/' <Cargo.toml > tmp.Cargo.toml
mv tmp.Cargo.toml Cargo.toml
