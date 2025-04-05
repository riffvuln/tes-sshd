#!/bin/bash

install_rust() {
  sudo apt install rustup gcc
  sudo rustup update nightly
  sudo rustup default nightly
  sudo rustup toolchain install nightly
}

install_rust
