target/esp: target/kernel
	mkdir -p target/esp
	mkdir -p target/esp/EFI/BOOT
	dd if=${LIMINE_PATH}/BOOT${ARCH_UPPER}.EFI of=target/esp/EFI/BOOT/BOOT${ARCH_UPPER}.EFI >/dev/null
	dd if=target/kernel of=target/esp/kernel >/dev/null
	printf "timeout: 0\nserial: yes\nrandomise_memory: ${EXPENS_TEST}\nverbose: yes\n/CHOS:\nprotocol: limine\npath: boot():/kernel\nkaslr: yes" >target/esp/limine.conf