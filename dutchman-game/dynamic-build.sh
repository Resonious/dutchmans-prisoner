sed s_\\[\"rlib\"\\]_\\[\"dylib\"\\]_ <Cargo.toml > tmp.Cargo.toml
mv tmp.Cargo.toml Cargo.toml
