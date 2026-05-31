#!/bin/bash
#
# SPDX-License-Identifier: Apache-2.0 or MIT
# Copyright 2021 Sony Group Corporation
#
# Ref: https://github.com/libseccomp-rs/libseccomp-rs/blob/v0.4.0/scripts/install_libseccomp.sh

set -o errexit

# installed libseccomp version by default
DEFAULT_LIBSECCOMP_VER="v2.6.0"
WORK_DIR="$(mktemp -d --tmpdir build-libseccomp.XXXXX)"

function finish() {
    rm -rf "${WORK_DIR}"
}

trap finish EXIT

function libseccomp_sha256() {
    case "$1" in
        v2.5.4)
            echo "d82902400405cf0068574ef3dc1fe5f5926207543ba1ae6f8e7a1576351dcbdb"
            ;;
        v2.6.0)
            echo "83b6085232d1588c379dc9b9cae47bb37407cf262e6e74993c61ba72d2a784dc"
            ;;
        *)
            echo "unsupported libseccomp version $1; add its release checksum before using it" >&2
            return 1
            ;;
    esac
}


function build_and_install_gperf() {
    gperf_version="3.1"
    gperf_url="https://ftp.gnu.org/gnu/gperf"
    gperf_tarball="gperf-${gperf_version}.tar.gz"
    gperf_tarball_url="${gperf_url}/${gperf_tarball}"
    gperf_sha256="588546b945bba4b70b6a3a616e80b4ab466e3f33024a352fc2198112cdbb3ae2"

    echo "Build and install gperf version ${gperf_version}"
    gperf_install_dir="$(mktemp -d --tmpdir build-gperf.XXXXX)"
    curl -fsSLO "${gperf_tarball_url}"
    echo "${gperf_sha256}  ${gperf_tarball}" | sha256sum -c -
    tar -xf "${gperf_tarball}"
    pushd "gperf-${gperf_version}"
    ./configure --prefix="${gperf_install_dir}"
    make
    make install
    export PATH=$PATH:"${gperf_install_dir}"/bin
    popd
    echo "Gperf installed successfully"
}

function build_and_install_libseccomp() {
    libseccomp_version=${opt_ver}
    libseccomp_install_dir=${opt_dir}
    mkdir -p "${libseccomp_install_dir}"

    if [[ ${libseccomp_version} != v* ]]; then
        echo "libseccomp version must be a pinned release tag (for example, v2.6.0)" >&2
        exit 1
    fi

    libseccomp_release="${libseccomp_version#v}"
    libseccomp_tarball="libseccomp-${libseccomp_release}.tar.gz"
    libseccomp_tarball_url="https://github.com/seccomp/libseccomp/releases/download/${libseccomp_version}/${libseccomp_tarball}"
    libseccomp_sha256=$(libseccomp_sha256 "${libseccomp_version}")

    echo "Build and install libseccomp version ${libseccomp_version}"
    curl -fsSLO "${libseccomp_tarball_url}"
    echo "${libseccomp_sha256}  ${libseccomp_tarball}" | sha256sum -c -
    tar -xf "${libseccomp_tarball}"
    pushd "libseccomp-${libseccomp_release}"

    if [[ ${opt_musl} -eq 1 ]]; then
        # Set FORTIFY_SOURCE=1 because the musl-libc does not have some functions about FORTIFY_SOURCE=2
        cflags="-U_FORTIFY_SOURCE -D_FORTIFY_SOURCE=1 -O2"
        ./configure --prefix="${libseccomp_install_dir}" CFLAGS="${cflags}" --enable-static
    else
        ./configure --prefix="${libseccomp_install_dir}" --enable-static
    fi
    make
    make install
    popd
    echo "Libseccomp installed successfully"
}

#
# Print out script usage details
#
function usage() {
cat <<EOF
Build and install libseccomp library from sources

USAGE:
  install_libseccomp [-m] [-v VERSION] [-i DIR]

OPTIONS:
  -h            : show this help message
  -m            : install libseccomp library for musl-libc [default: GNU-libc]
  -v [VERSION]  : specify the pinned libseccomp release tag to install [default: ${DEFAULT_LIBSECCOMP_VER}]
                  The installer only accepts release tags with an in-script SHA-256 checksum.
  -i [DIR]      : specify the directory for installing libseccomp library [default: /usr/local]
EOF
}

function main() {
    local opt_ver=${DEFAULT_LIBSECCOMP_VER}
    local opt_musl=0
    local opt_dir="/usr/local"

    while getopts "hmi:v:" opt; do
        case $opt in
            m)
                opt_musl=1
                ;;
            i)
                opt_dir="${OPTARG}"
                ;;
            v)
                opt_ver="${OPTARG}"
                ;;
            h|*)
                usage
                exit 1
                ;;
        esac
    done

    libseccomp_sha256 "${opt_ver}" >/dev/null

    pushd "${WORK_DIR}"
    # gperf is required for building the libseccomp.
    build_and_install_gperf
    build_and_install_libseccomp
    popd
}

main "$@"
