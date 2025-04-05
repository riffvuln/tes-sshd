#!/bin/sh

sudo apt install rustup
sudo apt install gcc
sudo rustup update nightly
sudo rustup target add x86_64-unknown-linux-gnu
