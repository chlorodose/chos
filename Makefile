ifeq ("$(wildcard configure/CONFIGURED)","")
$(info You has not configured the build system yet.)
$(info Please run './configure.sh' first.)
$(info If you have already run it, please check for the return value of the configure script.)
$(error exiting due to not configured)
endif

LOCAL_TARGET := $(shell rustc -vV | grep host | cut -d' ' -f2)

ifneq ("${DEBUG}", "")
	QEMU_EXTRA_ARG := -S
else
	QEMU_EXTRA_ARG :=
endif

run: target/esp
	qemu-system-riscv64 -s ${QEMU_EXTRA_ARG} \
		-M virt -cpu rv64 \
		-smp 4 -m 1G \
		-display none -serial stdio \
		-net user -audio driver=pipewire,model=virtio \
		-drive if=pflash,unit=0,readonly=on,format=raw,file=configure/OVMF_RO.fd \
		-drive if=pflash,unit=1,readonly=off,format=raw,file=configure/OVMF_RW.fd \
		-drive file=fat:rw:target/esp,format=raw

test:
	cargo test -Z build-std=std,test --target ${LOCAL_TARGET}

debug:
	rust-gdb \
		-ex "file target/kernel" \
		-ex "target remote localhost:1234"

include configure/configure.mk

clean:
	rm -rf target

kernel: target/kernel
	cp target/kernel .

.PHONY: target/kernel
target/kernel:
	mkdir -p target/esp
	cargo build --bin ${BL} --features ${BL} -Z unstable-options --artifact-dir target --profile=${PROFILE}
	rm -f target/kernel
	mv target/${BL} target/kernel