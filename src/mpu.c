#include "mpu.h"
#define MPU_TYPE (*(volatile uint32_t *)0xE000ED90UL)
#define MPU_CTRL (*(volatile uint32_t *)0xE000ED94UL)
#define MPU_RNR  (*(volatile uint32_t *)0xE000ED98UL)
#define MPU_RBAR (*(volatile uint32_t *)0xE000ED9CUL)
#define MPU_RASR (*(volatile uint32_t *)0xE000EDA0UL)
#define RASR_ENABLE 1UL
#define RASR_XN (1UL<<28)
#define AP_PRIV_RW (1UL<<24)
#define AP_FULL_RW (3UL<<24)
#define AP_RO_ALL (6UL<<24)
#define SIZE_4K (11UL<<1)
#define SIZE_4M (21UL<<1)
#define SIZE_512M (28UL<<1)
static inline void barrier(void) { __asm volatile("dsb sy\n isb sy" ::: "memory"); }
static void region(uint32_t n,uint32_t base,uint32_t rasr) { MPU_RNR=n; MPU_RBAR=base; MPU_RASR=rasr; }
void mpu_init(void) { (void)MPU_TYPE; MPU_CTRL=0; barrier(); region(0,0x00000000UL,RASR_ENABLE|AP_RO_ALL|SIZE_4M); region(2,0x20010000UL,RASR_ENABLE|RASR_XN|AP_FULL_RW|SIZE_4K); region(3,0x40000000UL,RASR_ENABLE|RASR_XN|AP_PRIV_RW|SIZE_512M); MPU_CTRL=5UL; barrier(); }
void mpu_switch_task(const tcb_t *task) { MPU_CTRL=0; barrier(); region(1,(uint32_t)task->ram_base,RASR_ENABLE|RASR_XN|AP_FULL_RW|SIZE_4K); MPU_CTRL=5UL; barrier(); }
