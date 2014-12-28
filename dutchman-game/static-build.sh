sed s_\\[\"dylib\"\\]_\\[\"rlib\"\\]_ <Cargo.toml > tmp.Cargo.toml
mv tmp.Cargo.toml Cargo.toml
