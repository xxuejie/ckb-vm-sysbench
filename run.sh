#!/usr/bin/env bash

set -ex

sudo apt-get update
sudo apt-get install -y build-essential linux-tools-common linux-tools-generic linux-tools-`uname -r`

curl https://sh.rustup.rs -sSf | sh -s -- -y
source ~/.cargo/env

git clone "$1" ckb-vm
cd ckb-vm
git checkout "$2"
cd ..
cargo build --release

mkdir rundata
./target/release/runner --tx-file data/gw_state_validator_0x9ad651b1771ad5514e1fd5a4e439a7bcc6a6f5a81e2e6918e8da8709c407529f.json.gz --cell-index 0 --cell-type input --script-type type > rundata/gw.log
tar czf rundata.tar.gz rundata
