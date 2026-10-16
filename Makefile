PREFIX ?= arm-none-eabi
CC := $(PREFIX)-gcc
OBJCOPY := $(PREFIX)-objcopy
OBJDUMP := $(PREFIX)-objdump
QEMU ?= qemu-system-arm
BUILD := build
TARGET := $(BUILD)/microkernel
SOURCES := $(wildcard src/*.c) src/startup.s
CFLAGS := -mcpu=cortex-m3 -mthumb -O0 -g -nostdlib -ffreestanding -fno-builtin -Wall -Wextra -Werror -Iinc
LDFLAGS := -T linker.ld -nostdlib -Wl,--gc-sections -Wl,-Map,$(TARGET).map
.PHONY: all build clean run debug disasm
all: build
build: $(TARGET).elf $(TARGET).bin
$(TARGET).elf: $(SOURCES) linker.ld
	@mkdir -p $(BUILD)
	$(CC) $(CFLAGS) $(SOURCES) $(LDFLAGS) -o $@
$(TARGET).bin: $(TARGET).elf
	$(OBJCOPY) -O binary $< $@
run: build
	$(QEMU) -M mps2-an385 -cpu cortex-m3 -kernel $(TARGET).elf -nographic
debug: build
	$(QEMU) -M mps2-an385 -cpu cortex-m3 -kernel $(TARGET).elf -nographic -s -S
disasm: build
	$(OBJDUMP) -d -S $(TARGET).elf
clean:
	$(RM) -r $(BUILD)
