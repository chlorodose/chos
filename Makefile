ifeq ("$(wildcard configure/CONFIGURED)","")
$(info You has not configured the build system yet.)
$(info Please run './configure.sh' first.)
$(info If you have already run it, please check for the return value of the configure script.)
$(error exiting due to not configured)
endif

run: target/esp
	qemu-system-riscv64 \
		-M virt -cpu rv64 \
		-smp 4 -m 1G \
		-display gtk -serial stdio \
		-device ramfb -device qemu-xhci -device usb-kbd -device usb-mouse \
		-drive if=pflash,unit=0,readonly=on,format=raw,file=configure/OVMF_RO.fd \
		-drive if=pflash,unit=1,readonly=off,format=raw,file=configure/OVMF_RW.fd \
		-drive file=fat:rw:target/esp,format=raw

include configure/configure.mk

clean:
	rm -rf target

target/kernel:
	mkdir -p target/esp
	cargo build --package bootloader --bin ${BL} -Z unstable-options --artifact-dir target --profile=${PROFILE}
	mv target/${BL} target/kernel