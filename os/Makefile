.PHONY: run build clean

APP=target/riscv64gc-unknown-none-elf/release/os
BOOTLOADER=../bootloader/rustsbi-qemu.bin

build:
	cargo build --release

run: build
	qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -kernel $(APP)

clean:
	cargo clean
