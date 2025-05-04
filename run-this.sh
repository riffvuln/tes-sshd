#!/bin/bash

install_rust() {
  sudo apt install rustup gcc
  rustup update nightly
  rustup default nightly
  rustup toolchain install nightly
}

install_mojo() {
  curl -ssL https://magic.modular.com/cde5b750-6171-4f22-8fca-63e11fe34ea4 | bash
  source /root/.bashrc
}

setup_workdir() {
  echo "alias workdir='cd /home/runner/work/tes-sshd/tes-sshd/Projects'" >> ~/.bashrc
  source ~/.bashrc
}

setup_workdir
install_rust
install_mojo
