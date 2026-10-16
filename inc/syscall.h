#ifndef SYSCALL_H
#define SYSCALL_H
#include <stdint.h>
enum { SVC_YIELD=0, SVC_SLEEP=1, SVC_SEND=2, SVC_RECEIVE=3 };
static inline void sys_yield(void) { __asm volatile("svc %0" :: "I"(SVC_YIELD) : "memory"); }
static inline void sys_sleep(uint32_t ms) { register uint32_t r0 __asm("r0")=ms; __asm volatile("svc %1" : "+r"(r0) : "I"(SVC_SLEEP) : "memory"); }
static inline int sys_send(uint8_t *msg) { register uint32_t r0 __asm("r0")=(uint32_t)msg; __asm volatile("svc %1" : "+r"(r0) : "I"(SVC_SEND) : "memory"); return (int)r0; }
static inline int sys_receive(uint8_t *buf) { register uint32_t r0 __asm("r0")=(uint32_t)buf; __asm volatile("svc %1" : "+r"(r0) : "I"(SVC_RECEIVE) : "memory"); return (int)r0; }
#endif
