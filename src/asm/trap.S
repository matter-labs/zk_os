# trap.S
# Trap handler and global context
# Steve Operating System
# Stephen Marz
# 24 February 2019
.option norvc
.altmacro
.set NUM_GP_REGS, 32  # Number of registers per context
.set REG_SIZE, 4   # Register size (in bytes)

# Use macros for saving and restoring multiple registers
.macro save_gp i, basereg=t6
	sw	x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro load_gp i, basereg=t6
	lw	x\i, ((\i)*REG_SIZE)(\basereg)
.endm

.section .text
.global m_trap_vector
# This must be aligned by 4 since the last two bits
# of the mtvec register do not contribute to the address
# of this vector.
.align 4
m_trap_vector:
	#  32 gp regs		0
	#  SATP register	128
	#  Trap stack       132
	#  CPU HARTID		136

	csrrw	x31, mscratch, x31
	// save them to the stack
	sw x0, 0(sp)
	sw x1, -4(sp)
	sw x2, -8(sp)
	sw x3, -12(sp)
	sw x4, -16(sp)
	sw x5, -20(sp)
	sw x6, -24(sp)
	sw x7, -28(sp)
	sw x9, -32(sp)
	sw x9, -36(sp)
	sw x10, -40(sp)
	sw x11, -44(sp)
	sw x12, -48(sp)
	sw x13, -52(sp)
	sw x14, -56(sp)
	sw x15, -60(sp)
	sw x16, -64(sp)
	sw x17, -68(sp)
	sw x18, -72(sp)
	sw x19, -76(sp)
	sw x20, -80(sp)
	sw x21, -84(sp)
	sw x22, -88(sp)
	sw x23, -92(sp)
	sw x24, -96(sp)
	sw x25, -100(sp)
	sw x26, -104(sp)
	sw x27, -108(sp)
	sw x28, -112(sp)
	sw x29, -116(sp)
	sw x30, -120(sp)

	mv x30, x31 //t5
	csrr x31, mscratch
	sw x31, -124(sp)
	addi sp, sp, -128

	// disable interrupts
	csrw	mie, zero

	// form ABI
	csrr	a0, mepc
	csrr	a1, mtval
	csrr	a2, mcause
	csrr	a3, mhartid
	csrr	a4, mstatus
	mv		a5, sp
	call	m_trap

	mret

# 	# When we get here, we've returned from m_trap, restore registers
# 	# and return.
# 	# m_trap will return the return address via a0.

# 	csrw	mepc, a0
# 	# Now load the trap frame back into t6
# 	csrr	t6, mscratch

# 	csrr	t1, mstatus
# 	srli	t0, t1, 13
# 	andi	t0, t0, 3
# 	li		t3, 3
# 	bne		t0, t3, 1f
# 1:
# 	# Restore all GP registers
# 	.set	i, 1
# 	.rept	31
# 		load_gp %i
# 		.set	i, i+1
# 	.endr

# 	# Since we ran this loop 31 times starting with i = 1,
# 	# the last one loaded t6 back to its original value.
# 	mret

# .global switch_to_user
# switch_to_user:
#     # a0 - Frame address
# 	# a1 - Program counter
# 	# a2 - SATP Register
#     csrw    mscratch, a0

# 	# Load program counter
# 	lw		a1, 520(a0)
# 	# Load satp
# 	lw		a2, 512(a0)
# 	# Load processor mode
# 	lw		a3, 552(a0)
# 	# Pid
# 	# ld		a4, 544(a0)

# 	# 1 << 7 is MPIE
# 	# Since user mode is 00, we don't need to set anything
# 	# in MPP (bits 12:11)
# 	li		t0, 1 << 7 | 1 << 5 | 1 << 13
# 	# Combine enable bits with mode bits.
# 	slli	a3, a3, 11
# 	or		t0, t0, a3
# 	csrw	mstatus, t0
# 	csrw	mepc, a1
# 	csrw	satp, a2
# 	li		t1, 0xaaa
# 	csrw	mie, t1
# 	la		t2, m_trap_vector
# 	csrw	mtvec, t2
# 	# This fence forces the MMU to flush the TLB. However, since
# 	# we're using the PID as the address space identifier, we might
# 	# only need this when we create a process. Right now, this ensures
# 	# correctness, however it isn't the most efficient.
# 	# sfence.vma
# 	# A0 is the context frame, so we need to reload it back
# 	# and mret so we can start running the program.
# 	mv	t6, a0
# 	.set	i, 0
# 	.rept	32
# 		load_fp %i
# 		.set i, i+1
# 	.endr
# 1:
# 	.set	i, 1
# 	.rept	31
# 		load_gp %i, t6
# 		.set	i, i+1
# 	.endr

#     mret


# .global make_syscall
# make_syscall:
# 	# We're setting this up to work with libgloss
# 	# They want a7 to be the system call number and all parameters
# 	# in a0 - a5
# 	mv	a7, a0
# 	mv	a0, a1
# 	mv	a1, a2
# 	mv	a2, a3
# 	mv	a3, a4
# 	mv	a4, a5
# 	mv	a5, a6
# 	ecall
# 	ret
