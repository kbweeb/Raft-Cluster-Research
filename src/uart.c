#include "uart.h"
#include "nvic.h"
#define UB 0x40004000UL
#define UD (*(volatile uint32_t *)(UB+0x00UL))
#define US (*(volatile uint32_t *)(UB+0x04UL))
#define UC (*(volatile uint32_t *)(UB+0x08UL))
#define UI (*(volatile uint32_t *)(UB+0x0CUL))
#define UBD (*(volatile uint32_t *)(UB+0x10UL))
#define TXFULL 1UL
#define RXFULL 2UL
#define TXEN 1UL
#define RXEN 2UL
#define RXIE 8UL
static volatile char rx[128]; static volatile uint32_t head, tail;
void uart_init(uint32_t baud) { head=tail=0; UC=0; UBD=25000000UL/baud; UC=TXEN|RXEN|RXIE; nvic_clear_pending_irq(UART0_IRQ); nvic_enable_irq(UART0_IRQ); }
void uart_putc(char c) { while (US&TXFULL){} UD=(uint32_t)(uint8_t)c; }
void uart_puts(const char *s) { while(*s) { if(*s=='\n') uart_putc('\r'); uart_putc(*s++); } }
int uart_getc(char *c) { if(head==tail) return 0; *c=rx[tail]; tail=(tail+1U)&127U; return 1; }
void uart_poll_rx(void) { while(US&RXFULL) { uint32_t n=(head+1U)&127U; char c=(char)UD; if(n!=tail) { rx[head]=c; head=n; } } }
void UART0_Handler(void) { uart_poll_rx(); }
