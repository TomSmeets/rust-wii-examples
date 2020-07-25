# Triangle example ported to rust

## Building

```bash
    export DEVKITPRO=...
    cargo +nightly build -Z build-std=core,alloc --target powerpc-unknown-eabi.json
    elf2dol ./target/powerpc-unknown-eabi/debug/rust-wii ./target/powerpc-unknown-eabi/debug/rust-wii.dol
```

Or with nix installed

```bash
    nix-build ./env.nix && ./result
```

