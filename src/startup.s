.syntax unified
.cpu cortex-m3
.thumb
.global Reset_Handler
.extern main
.extern _estack
.extern _sidata
.extern _sdata
.extern _edata
.extern _sbss
.extern _ebss
.extern fault_capture
.section .vectors,"a",%progbits
.word _estack,Reset_Handler,NMI_Handler,HardFault_Handler,MemManage_Handler,BusFault_Handler,UsageFault_Handler
.word 0,0,0,0,SVC_Handler,DebugMon_Handler,0,PendSV_Handler,SysTick_Handler
.word Default_Handler,Default_Handler,Default_Handler,Default_Handler,Default_Handler,UART0_Handler
.rept 26
.word Default_Handler
.endr
.section .text.Reset_Handler,"ax",%progbits
.thumb_func
Reset_Handler:
 ldr r0,=_estack
 mov sp,r0
 ldr r0,=_sdata
 ldr r1,=_edata
 ldr r2,=_sidata
1: cmp r0,r1
 bcs 2f
 ldr r3,[r2],#4
 str r3,[r0],#4
 b 1b
2: ldr r0,=_sbss
 ldr r1,=_ebss
 movs r2,#0
3: cmp r0,r1
 bcs 4f
 str r2,[r0],#4
 b 3b
4: bl main
 b .
.thumb_func
Fault_Trampoline:
 tst lr,#4
 ite eq
 mrseq r0,msp
 mrsne r0,psp
 mov r1,lr
 b fault_capture
.thumb_func
HardFault_Handler: movs r2,#1; b Fault_Trampoline
.thumb_func
MemManage_Handler: movs r2,#2; b Fault_Trampoline
.thumb_func
BusFault_Handler: movs r2,#3; b Fault_Trampoline
.thumb_func
UsageFault_Handler: movs r2,#4; b Fault_Trampoline
.section .text.Default_Handler,"ax",%progbits
.thumb_func
Default_Handler: b Default_Handler
.weak NMI_Handler
.thumb_set NMI_Handler,Default_Handler
.weak DebugMon_Handler
.thumb_set DebugMon_Handler,Default_Handler
