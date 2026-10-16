#include "ipc.h"
typedef struct { uint8_t bytes[IPC_MESSAGE_SIZE]; } ipc_slot_t;
typedef struct { volatile uint32_t head, tail; ipc_slot_t slots[4]; } ipc_queue_t;
static ipc_queue_t queue __attribute__((section(".shared"), aligned(32)));
static inline void dmb(void) { __asm volatile("dmb sy" ::: "memory"); }
void ipc_init(void) { queue.head=queue.tail=0U; dmb(); }
int ipc_send(const uint8_t *msg) { uint32_t h=queue.head, n=(h+1U)&3U; uint32_t i; if(n==queue.tail) return -1; for(i=0;i<IPC_MESSAGE_SIZE-1U && msg[i];i++) queue.slots[h].bytes[i]=msg[i]; queue.slots[h].bytes[i]=0; dmb(); queue.head=n; return 0; }
int ipc_receive(uint8_t *buf) { uint32_t t=queue.tail,i; if(t==queue.head) return 0; dmb(); for(i=0;i<IPC_MESSAGE_SIZE;i++) { buf[i]=queue.slots[t].bytes[i]; if(!buf[i]) break; } dmb(); queue.tail=(t+1U)&3U; return 1; }
