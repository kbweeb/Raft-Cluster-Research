#ifndef NVIC_H
#define NVIC_H
#include <stdint.h>
#define UART0_IRQ 5U
void nvic_enable_irq(uint32_t irq); void nvic_clear_pending_irq(uint32_t irq);
#endif
