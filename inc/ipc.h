#ifndef IPC_H
#define IPC_H
#include <stdint.h>
#define IPC_MESSAGE_SIZE 64U
void ipc_init(void); int ipc_send(const uint8_t *msg); int ipc_receive(uint8_t *buf);
#endif
