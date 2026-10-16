#include "nvic.h"
#define ISER ((volatile uint32_t *)0xE000E100UL)
#define ICPR ((volatile uint32_t *)0xE000E280UL)
void nvic_enable_irq(uint32_t irq) { ISER[irq >> 5U] = 1UL << (irq & 31U); }
void nvic_clear_pending_irq(uint32_t irq) { ICPR[irq >> 5U] = 1UL << (irq & 31U); }
