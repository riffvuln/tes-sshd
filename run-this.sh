#!/bin/sh

# Function to install a package and check for errors
install_package() {
  sudo apt install -y "$1"
  if [ $? -ne 0 ]; then
    echo "Failed to install $1"
    exit 1
  fi
}

# Function to run a command and check for errors
run_command() {
  eval "$1"
  if [ $? -ne 0 ]; then
    echo "Failed to execute: $1"
    exit 1
  fi
}

# Install required packages
install_package "rustup"
install_package "gcc"

# Update rustup and add target
run_command "sudo rustup update nightly"
run_command "sudo rustup target add x86_64-unknown-linux-gnu"
