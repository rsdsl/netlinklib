#! /bin/bash

TARGETS="radvd pppoe3 dslite netlinkd dhcp4d"

for T in ${TARGETS}; do
	cd ${T}

	set -e

	cargo add --git https://github.com/rsdsl/netlinklib.git rsdsl_netlinklib
	cargo update
	cargo clippy --target x86_64-unknown-linux-musl

	set +e

	git add Cargo.*
	git commit -m "update netlinklib"
	git push origin $(git branch --show-current) --tags
	git push himbeergit $(git branch --show-current) --tags

	cd ..
done

cd dnsd

set -e

cargo add --git https://github.com/rsdsl/dhcp4d.git rsdsl_dhcp4d
cargo update
cargo clippy --target x86_64-unknown-linux-musl

set +e

git add Cargo.*
git commit -m "update dhcp4d"
git push origin $(git branch --show-current) --tags
git push himbeergit $(git branch --show-current) --tags
