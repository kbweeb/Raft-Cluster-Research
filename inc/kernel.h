#ifndef KERNEL_H
#define KERNEL_H
#include <stdint.h>
#define MAX_TASKS 2U
#define TASK_STACK_WORDS 512U
typedef enum { TASK_READY, TASK_RUNNING, TASK_BLOCKED } task_state_t;
typedef struct {
    uint32_t *sp; uint32_t *ram_base; uint32_t ram_size;
    uint32_t wake_tick; uint8_t priority; uint8_t id; task_state_t state;
    const char *name; void (*entry)(void);
} tcb_t;
extern volatile tcb_t *g_current;
void kernel_init(void);
int task_create(void (*entry)(void), uint32_t *stack, uint32_t stack_words, uint32_t *ram_base, uint8_t priority, const char *name);
void kernel_start(void) __attribute__((noreturn));
void scheduler_tick(void); void scheduler_select_next(void); void scheduler_yield(void); void scheduler_sleep(uint32_t ms);
void PendSV_Handler(void) __attribute__((naked)); void SVC_Handler(void) __attribute__((naked));
void svc_dispatch(uint32_t *frame, uint32_t number);
#endif
