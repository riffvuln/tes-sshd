#!/bin/bash

install_rust() {
  sudo apt install rustup gcc
  rustup update nightly
  rustup default nightly
  rustup toolchain install nightly
}

install_rust
