## emit assembly code

### online assembler

- [Compiler Explorer by Matt Godbolt](https://godbolt.org/)
- Compiler options:
```sh
-O --target=<the_target>
# e.g 
-O --target=aarch64-unknown-linux-musl
# e.g
-O --target=x86_64-unknown-linux-musl
```

### locally

- add the targets you wish to cross compile to
```sh
rustup target add <the_target>
# e.g.
rustup target add aarch64-unknown-linux-musl
# e.g.
rustup target add x86_64-unknown-linux-musl
```
- install [cargo-show-asm](https://crates.io/crates/cargo-show-asm)
```sh
cargo install cargo-show-asm
```

#### using just
- install [just](https://github.com/casey/just)
```sh
cargo install just
```
- run any of the recipes defined in [justfile](./justfile)
```sh
# list recipes:
just

# run a recipe
just asm-all simple_add_ten
```

#### manually
- run `cargo-show-asm`
```sh
cargo asm --lib "<the_function>" --full-name --simplify --target=<the_target>

# e.g.
cargo asm --lib "relaxed_atomic_fetch_or" --full-name --simplify --target=aarch64-unknown-linux-musl

# e.g.
cargo asm --lib "simple_add_ten" --full-name --simplify --target=x86_64-unknown-linux-musl
```
