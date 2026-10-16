ENTRY(Reset_Handler)
MEMORY {
  FLASH      (rx)  : ORIGIN = 0x00000000, LENGTH = 4M
  KERNEL_RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
  SHARED_RAM (rwx) : ORIGIN = 0x20010000, LENGTH = 4K
  TASK_A_RAM (rwx) : ORIGIN = 0x20011000, LENGTH = 4K
  TASK_B_RAM (rwx) : ORIGIN = 0x20012000, LENGTH = 4K
}
_estack = ORIGIN(KERNEL_RAM) + LENGTH(KERNEL_RAM);
SECTIONS {
  .vectors ORIGIN(FLASH) : ALIGN(256) { KEEP(*(.vectors)) } > FLASH
  .text : ALIGN(4) { *(.text.Reset_Handler) *(.text*) *(.glue_7*) } > FLASH
  .rodata : ALIGN(4) { *(.rodata*) } > FLASH
  _sidata = LOADADDR(.data);
  .data : ALIGN(4) { _sdata = .; *(.data*) . = ALIGN(4); _edata = .; } > KERNEL_RAM AT > FLASH
  .bss (NOLOAD) : ALIGN(8) { _sbss = .; *(.bss*) *(COMMON) . = ALIGN(8); _ebss = .; } > KERNEL_RAM
  .shared (NOLOAD) : ALIGN(32) { __shared_start__ = .; *(.shared*) . = ALIGN(32); __shared_end__ = .; } > SHARED_RAM
  .task_a_ram (NOLOAD) : ALIGN(1024) { __task_a_ram_start__ = .; *(.task_a_ram*) . = ALIGN(1024); __task_a_ram_end__ = .; } > TASK_A_RAM
  .task_b_ram (NOLOAD) : ALIGN(1024) { __task_b_ram_start__ = .; *(.task_b_ram*) . = ALIGN(1024); __task_b_ram_end__ = .; } > TASK_B_RAM
}
