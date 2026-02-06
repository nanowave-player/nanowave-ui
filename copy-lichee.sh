#!/bin/sh

scp ./target/riscv64gc-unknown-linux-musl/release/nanowave-ui lichee2:/root/nanowave
scp ./scripts/* lichee2:/root/
