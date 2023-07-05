/*
    Entry point of all programs (_start).

    It initializes DWARF call frame information, the stack pointer, the
    frame pointer (needed for closures to work in start_rust) and the global
    pointer. Then it calls _start_rust.
*/

.option norvc

.section .init, "ax"
.global _start

_start:
  .option push
.option norelax
	la		gp, __global_pointer$
.option pop
	# SATP should be zero, but let's make sure. Each HART has its own
	# SATP register.
	csrw	satp, zero
	# Any hardware threads (hart) that are not bootstrapping
	# need to wait for an IPI
	csrr	t0, mhartid
	bnez	t0, 3f

	# Set all bytes in the BSS section to zero.
	la 		a0, _sbss
	la		a1, _ebss
	bgeu	a0, a1, 2f
1:
	sw		zero, (a0)
	addi	a0, a0, 4
	bltu	a0, a1, 1b
2:
	# The stack grows from bottom to top, so we put the stack pointer
	# to the very end of the stack range.
	la		sp, _sstack
	# Setting `mstatus` register:
	# 0b01 << 11: Machine's previous protection mode is 2 (MPP=2).
	li		t0, 0b11 << 11 | (1 << 13)
	csrw	mstatus, t0
	# Do not allow interrupts while running kinit
	csrw	mie, zero
	# Machine's exception program counter (MEPC) is set to `kinit`.
	la		t1, kinit
	csrw	mepc, t1
	# Set the return address to get us into supervisor mode
	la		ra, 2f
	# We use mret here so that the mstatus register is properly updated.
	mret
2:
	# We set the return address (ra above) to this label. When kinit() is finished
	# in Rust, it will return here.

	# Setting `mstatus` (supervisor status) register:
	# 0b01 << 11 : Previous protection mode is 1 (MPP=01 [Supervisor]).
	# 1 << 7     : Previous machine interrupt-enable bit is 1 (MPIE=1 [Enabled])
	# 1 << 5     : Previous interrupt-enable bit is 1 (SPIE=1 [Enabled]).
	# We set the "previous" bits because the mret will write the current bits
	# with the previous bits.
	li		t0, (0b00 << 11) | (1 << 7) | (1 << 5) | (1 << 13)
	csrw	mstatus, t0
	# Machine's trap vector base address is set to `m_trap_vector`, for
	# "machine" trap vector.
	la		t2, m_trap_vector
	csrw	mtvec, t2
	# Jump to first process. We put the MPP = 00 for user mode, so after
	# mret, we will jump to the first process' addresss in user mode.
	la		ra, 4f
	mret
3:
	# Parked harts go here
	wfi
	mret
4:
	# wfi = wait for interrupt. This is a hint to the harts to shut everything needed
	# down. However, the RISC-V specification allows for wfi to do nothing. Anyway,
	# with QEMU, this will save some CPU!
	wfi
	j		4b

# /*
#     Trap entry point (_start_trap)

#     Saves caller saved registers ra, t0..6, a0..7, calls _start_trap_rust,
#     restores caller saved registers and then returns.
# */
# .section .trap, "ax"
# .global default_start_trap

# default_start_trap:
#     addi sp, sp, -64

#     sw ra, 0(sp)
#     sw t0, 4(sp)
#     sw t1, 8(sp)
#     sw t2, 12(sp)
#     sw t3, 16(sp)
#     sw t4, 20(sp)
#     sw t5, 24(sp)
#     sw t6, 28(sp)
#     sw a0, 32(sp)
#     sw a1, 36(sp)
#     sw a2, 40(sp)
#     sw a3, 44(sp)
#     sw a4, 48(sp)
#     sw a5, 52(sp)
#     sw a6, 56(sp)
#     sw a7, 60(sp)

#     add a0, sp, zero
#     jal ra, _start_trap_rust

#     sw ra, 0(sp)
#     sw t0, 4(sp)
#     sw t1, 8(sp)
#     sw t2, 12(sp)
#     sw t3, 16(sp)
#     sw t4, 20(sp)
#     sw t5, 24(sp)
#     sw t6, 28(sp)
#     sw a0, 32(sp)
#     sw a1, 36(sp)
#     sw a2, 40(sp)
#     sw a3, 44(sp)
#     sw a4, 48(sp)
#     sw a5, 52(sp)
#     sw a6, 56(sp)
#     sw a7, 60(sp)

#     addi sp, sp, 64
#     mret

# /* Make sure there is an abort when linking */
# .section .text.abort
# .globl abort
# abort:
#     j abort