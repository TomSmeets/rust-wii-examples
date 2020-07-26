{ pkgs ? import <nixpkgs> {} }: let
  devkit = import (builtins.fetchurl https://gist.githubusercontent.com/TomSmeets/e306c414216def4fc8c12d0914717a7c/raw/7374086ad6d50318010157a1424c4d4d3ed42ad5/devkitppc.nix) { inherit pkgs; };
  # devkit = import ../default.nix { inherit pkgs; };
in pkgs.writeShellScript "build.sh" ''
    export DEVKITPRO="${devkit}"
    export PATH="$DEVKITPRO/devkitPPC/bin:$DEVKITPRO/tools/bin:$PATH"
    cargo +nightly build -Z build-std=core,alloc --target powerpc-unknown-eabi.json
    elf2dol ./target/powerpc-unknown-eabi/debug/rust-wii ./target/powerpc-unknown-eabi/debug/rust-wii.dol
''
