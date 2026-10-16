#ifndef UART_H
#define UART_H
#include <stdint.h>
void uart_init(uint32_t baud); void uart_putc(char c); void uart_puts(const char *s); int uart_getc(char *c); void uart_poll_rx(void); void UART0_Handler(void);
#endif
