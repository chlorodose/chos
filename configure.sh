#!/usr/bin/env bash

set -e

rm -rf configure
mkdir -p configure

BL_LIST=("limine")
ARCH_LIST=("riscv64")
PROFILE_LIST=("dev" "release")

PLATFORMS_RO=(["riscv64"]="RISCV64_VIRT_CODE")
PLATFORMS_RW=(["riscv64"]="RISCV64_VIRT_VARS")

gum log "Welcome to the CHOS kernel configuration script!"

BL=$(gum choose --limit 1 --header="Select bootloader:" --selected=limine \
    ${BL_LIST[@]}
)
gum log "Selected bootloader: ${BL}"
ARCH=$(gum choose --limit 1 --header="Select architecture:" --selected=riscv64 \
    ${ARCH_LIST[@]}
)
gum log "Selected architecture: ${ARCH}"
PROFILE=$(gum choose --limit 1 --header="Select codegen profile:" --selected=dev \
    ${PROFILE_LIST[@]}
)
gum log "Selected codegen profile: ${PROFILE}"


set +e
gum confirm "Enable expensive tests?" --default="no"
LAST_STATUS=$?
if [[ $LAST_STATUS -eq 0 ]]; then
    EXPENS_TEST="yes"
elif [[ $LAST_STATUS -eq 1 ]]; then
    EXPENS_TEST="no"
else
    exit 1
fi
set -e
gum log "Expensive tests: ${EXPENS_TEST}"

ARCH_UPPER=$(echo ${ARCH} | tr '[:lower:]' '[:upper:]')

echo "BL := ${BL}" >>configure/configure.mk
echo "ARCH := ${ARCH}" >>configure/configure.mk
echo "ARCH_UPPER := ${ARCH_UPPER}" >>configure/configure.mk
echo "PROFILE := ${PROFILE}" >>configure/configure.mk
echo "EXPENS_TEST := ${EXPENS_TEST}" >>configure/configure.mk

ln -sf ../src/bootloader/${BL}/linker.ld configure/linker.ld
echo "include src/bootloader/${BL}/bl.mk" >>configure/configure.mk

ln -sf ../src/arch/${ARCH}/target.json configure/target.json

gum spin --title "Downloading https://raw.githubusercontent.com/retrage/edk2-nightly/refs/heads/master/bin/RELEASE${PLATFORMS_RO["riscv64"]}.fd" -- \
    curl -o configure/OVMF_RO.fd https://raw.githubusercontent.com/retrage/edk2-nightly/refs/heads/master/bin/RELEASE${PLATFORMS_RO["riscv64"]}.fd
gum spin --title "Downloading https://raw.githubusercontent.com/retrage/edk2-nightly/refs/heads/master/bin/RELEASE${PLATFORMS_RW["riscv64"]}.fd" -- \
    curl -o configure/OVMF_RW.fd https://raw.githubusercontent.com/retrage/edk2-nightly/refs/heads/master/bin/RELEASE${PLATFORMS_RW["riscv64"]}.fd
truncate -s 33554432 configure/OVMF_RO.fd
truncate -s 33554432 configure/OVMF_RW.fd

touch configure/CONFIGURED