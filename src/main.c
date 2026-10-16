#include <stdint.h>
#include "kernel.h"
#include "uart.h"
#include "systick.h"
#include "syscall.h"
extern uint32_t __task_a_ram_start__,__task_b_ram_start__;
static uint32_t shell_stack[TASK_STACK_WORDS] __attribute__((section(".task_a_ram"),aligned(1024)));
static uint32_t worker_stack[TASK_STACK_WORDS] __attribute__((section(".task_b_ram"),aligned(1024)));
static int equal(const char *a,const char *b) { while(*a&&*b&&*a==*b){a++;b++;} return *a==*b; }
static void shell(void) { uint8_t in[64],out[64]; uint32_t n=0; sys_send((uint8_t *)"microkernel shell: help | ps | crash\n> "); for(;;) { int r=sys_receive(out); if(r==2) { sys_send((uint8_t *)"[sensor] "); sys_send(out); sys_send((uint8_t *)"\n> "); } else if(r==1) { char c=(char)out[0]; if(c=='\r'||c=='\n') { in[n]=0; sys_send((uint8_t *)"\n"); if(equal((char *)in,"help")) sys_send((uint8_t *)"help ps crash\n"); else if(equal((char *)in,"ps")) sys_send((uint8_t *)"shell READY prio=1; worker READY prio=2\n"); else if(equal((char *)in,"crash")) { volatile uint32_t *bad=(volatile uint32_t *)0x20012000UL; *bad=0xDEADBEEFUL; } else if(n) sys_send((uint8_t *)"unknown command\n"); n=0; sys_send((uint8_t *)"> "); } else if(c==8||c==127) { if(n)n--; } else if(n<63U) { in[n++]=(uint8_t)c; } } sys_sleep(10U); } }
static void worker(void) { uint32_t sample=0; uint8_t msg[64]; for(;;) { uint32_t v=1000U+(sample++%97U),i=0U,t=v; const char *p="sample="; while(*p)msg[i++]=(uint8_t)*p++; { uint32_t d=1U; while(t/d>=10U)d*=10U; do { msg[i++]=(uint8_t)('0'+t/d); t%=d; d/=10U; } while(d); } msg[i]=0; (void)sys_send(msg); sys_sleep(250U); } }
int main(void) { uart_init(115200U); uart_puts("\nM3 microkernel boot\n"); kernel_init(); (void)task_create(shell,shell_stack,TASK_STACK_WORDS,&__task_a_ram_start__,1U,"shell"); (void)task_create(worker,worker_stack,TASK_STACK_WORDS,&__task_b_ram_start__,2U,"worker"); systick_init(25000000U); kernel_start(); }
