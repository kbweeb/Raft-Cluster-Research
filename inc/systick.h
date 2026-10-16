#ifndef SYSTICK_H
#define SYSTICK_H
#include <stdint.h>
void systick_init(uint32_t core_hz); uint32_t systick_now(void); void SysTick_Handler(void);
#endif
