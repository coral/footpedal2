oofergang

## install

```
rustup target install thumbv6m-none-eabi
cargo install flip-link
cargo install --locked probe-rs-tools
cargo install elf2uf2-rs --locked
```

## build

`cargo build --target=thumbv6m-none-eabi --release`

should just work with

`cargo run --target=thumbv6m-none-eabi --release`

if it doesn't then yeah ???