{ pkgs ? import <nixpkgs> {} }: let
  devkit = import (builtins.fetchurl https://gist.githubusercontent.com/TomSmeets/e306c414216def4fc8c12d0914717a7c/raw/e99f59fcd3f0ad4a54e123d80b9a8a1b38ea152a/devkitppc.nix) { inherit pkgs; };

in pkgs.writeShellScript "build.sh" ''
    export DEVKITPRO="${devkit}"
    export PATH="$DEVKITPRO/devkitPPC/bin:$DEVKITPRO/tools/bin:$PATH"
    cargo +nightly build -Z build-std=core,alloc --target powerpc-unknown-eabi.json
    elf2dol ./target/powerpc-unknown-eabi/debug/rust-wii ./target/powerpc-unknown-eabi/debug/rust-wii.dol
''
