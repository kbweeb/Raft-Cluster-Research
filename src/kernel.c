#include "kernel.h"
#include "mpu.h"
#include "ipc.h"
#include "uart.h"
#include "systick.h"
#include "syscall.h"
#define ICSR (*(volatile uint32_t *)0xE000ED04UL)
#define SHPR3 (*(volatile uint32_t *)0xE000ED20UL)
#define PENDSV (1UL<<28)
static tcb_t tasks[MAX_TASKS]; static uint32_t count, cursor; static volatile uint32_t first_start;
volatile tcb_t *g_current;
static void task_exit(void) __attribute__((noreturn));
static void task_exit(void) { for(;;) scheduler_yield(); }
void kernel_init(void) { count=cursor=first_start=0; g_current=0; SHPR3=(SHPR3&0x0000FFFFUL)|0xFEFF0000UL; ipc_init(); mpu_init(); }
int task_create(void (*entry)(void),uint32_t *stack,uint32_t words,uint32_t *ram,uint8_t priority,const char *name) {
 uint32_t *sp; tcb_t *t; if(!entry||!stack||count>=MAX_TASKS||words<32U||priority>7U) return -1; sp=stack+words; sp=(uint32_t *)((uint32_t)sp&~7UL);
 *(--sp)=0x01000000UL; *(--sp)=((uint32_t)entry)|1UL; *(--sp)=((uint32_t)task_exit)|1UL; *(--sp)=0; *(--sp)=0; *(--sp)=0; *(--sp)=0; *(--sp)=0;
 *(--sp)=0; *(--sp)=0; *(--sp)=0; *(--sp)=0; *(--sp)=0; *(--sp)=0; *(--sp)=0; *(--sp)=0;
 t=&tasks[count]; t->sp=sp; t->ram_base=ram; t->ram_size=4096U; t->wake_tick=0; t->priority=priority; t->id=(uint8_t)count; t->state=TASK_READY; t->name=name; t->entry=entry; return (int)count++;
}
void scheduler_select_next(void) { uint32_t i,best=8U,chosen=0; for(i=0;i<count;i++) if(tasks[i].state==TASK_READY && tasks[i].priority<best) best=tasks[i].priority; for(i=1;i<=count;i++) { uint32_t n=(cursor+i)%count; if(tasks[n].state==TASK_READY && tasks[n].priority==best) { chosen=n; break; } } if(g_current&&g_current->state==TASK_RUNNING) g_current->state=TASK_READY; cursor=chosen; g_current=&tasks[chosen]; g_current->state=TASK_RUNNING; }
void scheduler_yield(void) { ICSR=PENDSV; }
void scheduler_sleep(uint32_t ms) { ((tcb_t *)g_current)->wake_tick=systick_now()+ms; ((tcb_t *)g_current)->state=TASK_BLOCKED; ICSR=PENDSV; }
void scheduler_tick(void) { uint32_t i,now=systick_now(),woke=0U; for(i=0;i<count;i++) if(tasks[i].state==TASK_BLOCKED && (int32_t)(now-tasks[i].wake_tick)>=0) { tasks[i].state=TASK_READY; woke=1U; } if(woke && g_current) ICSR=PENDSV; }
void kernel_start(void) { if(!count) for(;;){} cursor=count-1U; scheduler_select_next(); mpu_switch_task((const tcb_t *)g_current); first_start=1U; __asm volatile("svc 0" ::: "memory"); for(;;){} }
void svc_dispatch(uint32_t *f,uint32_t n) { uint8_t *p=(uint8_t *)f[0]; if(first_start) return; switch(n) { case SVC_YIELD: scheduler_yield(); break; case SVC_SLEEP: scheduler_sleep(f[0]); break; case SVC_SEND: if(((tcb_t *)g_current)->id==1U) f[0]=(uint32_t)ipc_send(p); else { uart_puts((const char *)p); f[0]=0; } break; case SVC_RECEIVE: { char c; if(uart_getc(&c)) { p[0]=(uint8_t)c; p[1]=0; f[0]=1; } else f[0]=(uint32_t)(ipc_receive(p)?2:0); } break; default: f[0]=(uint32_t)-1; break; } }
void SVC_Handler(void) { __asm volatile(
 "tst lr,#4\n ite eq\n mrseq r0,msp\n mrsne r0,psp\n ldr r1,[r0,#24]\n ldrb r1,[r1,#-2]\n ldr r2,=first_start\n ldr r3,[r2]\n cbz r3,1f\n movs r3,#0\n str r3,[r2]\n ldr r0,=g_current\n ldr r0,[r0]\n ldr r0,[r0]\n ldmia r0!,{r4-r11}\n msr psp,r0\n movs r0,#3\n msr control,r0\n isb\n ldr lr,=0xFFFFFFFD\n bx lr\n 1: b svc_dispatch\n"); }
void PendSV_Handler(void) { __asm volatile(
 "mrs r0,psp\n stmdb r0!,{r4-r11}\n ldr r1,=g_current\n ldr r2,[r1]\n str r0,[r2]\n push {r3,lr}\n bl scheduler_select_next\n ldr r0,=g_current\n ldr r0,[r0]\n bl mpu_switch_task\n pop {r3,lr}\n ldr r1,=g_current\n ldr r1,[r1]\n ldr r0,[r1]\n ldmia r0!,{r4-r11}\n msr psp,r0\n bx lr\n"); }
