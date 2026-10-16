#include "systick.h"
#include "kernel.h"
#include "uart.h"
#define STCTRL (*(volatile uint32_t *)0xE000E010UL)
#define STLOAD (*(volatile uint32_t *)0xE000E014UL)
#define STVAL  (*(volatile uint32_t *)0xE000E018UL)
static volatile uint32_t ms;
void systick_init(uint32_t core_hz) { STCTRL=0; STLOAD=(core_hz/1000U)-1U; STVAL=0; STCTRL=7U; }
uint32_t systick_now(void) { return ms; }
void SysTick_Handler(void) { ms++; uart_poll_rx(); scheduler_tick(); }
