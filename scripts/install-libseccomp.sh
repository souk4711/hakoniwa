#!/bin/bash
#
# SPDX-License-Identifier: Apache-2.0 or MIT
# Copyright 2021 Sony Group Corporation
#
# Ref: https://github.com/libseccomp-rs/libseccomp-rs/blob/v0.4.0/scripts/install_libseccomp.sh

set -o errexit

# installed libseccomp version by default
DEFAULT_LIBSECCOMP_VER="v2.5.4"
TENTATIVE_HEAD_VER="9.9.9"
WORK_DIR="$(mktemp -d --tmpdir build-libseccomp.XXXXX)"

function finish() {
    rm -rf "${WORK_DIR}"
}

trap finish EXIT

function build_and_install_gperf() {
    gperf_version="3.1"
    gperf_url="https://ftp.gnu.org/gnu/gperf"
    gperf_tarball="gperf-${gperf_version}.tar.gz"
    gperf_tarball_url="${gperf_url}/${gperf_tarball}"

    echo "Build and install gperf version ${gperf_version}"
    gperf_install_dir="$(mktemp -d --tmpdir build-gperf.XXXXX)"
    curl -sLO "${gperf_tarball_url}"
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

    echo "Build and install libseccomp version ${libseccomp_version}"
    git clone --depth=1 "https://github.com/seccomp/libseccomp.git" --branch "${libseccomp_version}" --single-branch
    pushd libseccomp
    if [[ ${libseccomp_version} == "main" ]]; then
        # Specify the tentative version of the libseccomp library because some
        # functions of the Rust bindings are restricted based on the version.
        sed -i "/^AC_INIT/ s/0.0.0/$TENTATIVE_HEAD_VER/" configure.ac
    fi

    ./autogen.sh
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
  -v [VERSION]  : specify the version of installed libseccomp library [default: ${DEFAULT_LIBSECCOMP_VER}]
                  If you want to install the HEAD of the libseccomp library (the main branch of the repository),
                  specify "main" and the version will be tentatively ${TENTATIVE_HEAD_VER}.
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

    pushd "${WORK_DIR}"
    # gperf is required for building the libseccomp.
    build_and_install_gperf
    build_and_install_libseccomp
    popd
}

main "$@"
