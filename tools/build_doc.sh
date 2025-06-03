#!/bin/bash
TOPDIR="$(dirname "$(readlink -f "${0}")")"
cargo doc --features="debug,user_properties,physics,avian,rapier"
mdbook --version &>/dev/null && (cd "${TOPDIR}/../book" && mdbook build)