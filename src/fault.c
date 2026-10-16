#include <stdint.h>
#include "uart.h"
static void hex(uint32_t v) { int i; for(i=7;i>=0;i--) { uint32_t n=(v>>((uint32_t)i*4U))&15U; uart_putc((char)(n<10U?'0'+n:'A'+n-10U)); } }
static void reg(const char *n,uint32_t v) { uart_puts(n); uart_puts("=0x"); hex(v); uart_puts("\n"); }
void fault_capture(uint32_t *s,uint32_t exc,uint32_t kind) { volatile uint32_t *cfsr=(volatile uint32_t *)0xE000ED28UL,*hfsr=(volatile uint32_t *)0xE000ED2CUL,*mmfar=(volatile uint32_t *)0xE000ED34UL,*bfar=(volatile uint32_t *)0xE000ED38UL; uart_puts("\n*** KERNEL FAULT ***\n"); reg("TYPE",kind); reg("EXC_RETURN",exc); reg("R0",s[0]); reg("R1",s[1]); reg("R2",s[2]); reg("R3",s[3]); reg("R12",s[4]); reg("LR",s[5]); reg("PC",s[6]); reg("XPSR",s[7]); reg("CFSR",*cfsr); reg("HFSR",*hfsr); reg("MMFAR",*mmfar); reg("BFAR",*bfar); uart_puts("SYSTEM HALTED\n"); for(;;) __asm volatile("wfi"); }
