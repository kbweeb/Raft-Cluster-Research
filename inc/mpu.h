#ifndef MPU_H
#define MPU_H
#include "kernel.h"
void mpu_init(void); void mpu_switch_task(const tcb_t *task);
#endif
