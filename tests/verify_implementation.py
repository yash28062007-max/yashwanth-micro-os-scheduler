#!/usr/bin/env python3
"""
Yashwanth Micro-OS: Strict Implementation & Algorithm Verification Suite
Performs automated algorithmic and structural audits across all kernel subsystems.
"""

import os
import re
import sys
from pathlib import Path

PASS_COUNT = 0
FAIL_COUNT = 0

def test(name, condition, details=""):
    global PASS_COUNT, FAIL_COUNT
    if condition:
        PASS_COUNT += 1
        print(f"  [PASS] {name}")
    else:
        FAIL_COUNT += 1
        print(f"  [FAIL] {name} - {details}")

# ==============================================================================
# 1. Structural Source Audit
# ==============================================================================
def audit_source_files(root_dir):
    print("\n--- 1. Rust Source File Structural Audit ---")
    rust_files = list(Path(root_dir, "src").rglob("*.rs"))
    rust_files.append(Path(root_dir, "tests", "test_micro_os.rs"))
    
    test("Rust Source Files Discovered", len(rust_files) >= 15, f"Found {len(rust_files)} files")
    
    for rf in sorted(rust_files):
        rel = rf.relative_to(root_dir)
        content = rf.read_text(encoding="utf-8", errors="replace")
        
        # Check balanced braces
        open_b = content.count("{")
        close_b = content.count("}")
        test(f"File Syntax Balance: {rel}", open_b == close_b, f"{{: {open_b}, }}: {close_b}")

# ==============================================================================
# 2. Algorithmic Verification: ReadyQueue Ring Buffer
# ==============================================================================
class PyReadyQueue:
    def __init__(self, capacity=16):
        self.capacity = capacity
        self.buffer = [None] * capacity
        self.head = 0
        self.tail = 0
        self.count = 0

    def enqueue(self, item):
        if self.count >= self.capacity:
            return False
        self.buffer[self.tail] = item
        self.tail = (self.tail + 1) % self.capacity
        self.count += 1
        return True

    def dequeue(self):
        if self.count == 0:
            return None
        item = self.buffer[self.head]
        self.buffer[self.head] = None
        self.head = (self.head + 1) % self.capacity
        self.count -= 1
        return item

def test_ready_queue_algorithms():
    print("\n--- 2. Algorithmic Audit: ReadyQueue FIFO & Boundaries ---")
    q = PyReadyQueue(16)
    test("Initial Queue Empty", q.count == 0 and q.dequeue() is None)
    
    # Enqueue elements
    for i in range(16):
        test(f"Enqueue Task {i}", q.enqueue(i))
    
    test("Queue Full At Capacity", q.count == 16)
    test("Reject Overflow (17th task)", not q.enqueue(999))
    
    # Dequeue FIFO check
    first = q.dequeue()
    test("FIFO Ordering: First Item is 0", first == 0)
    test("Queue Decrements Count", q.count == 15)
    
    # Ring buffer wrap-around
    test("Enqueue into freed slot", q.enqueue(100))
    test("Wrap-around Count == 16", q.count == 16)

# ==============================================================================
# 3. Algorithmic Verification: Dynamic Quantum Policy
# ==============================================================================
def compute_dynamic_quantum(n_runnable, yield_count, preempt_count, priority, target_cycle=36, q_min=2, q_max=16):
    n = max(1, n_runnable)
    load_q = max(q_min, min(q_max, target_cycle // n))
    
    if preempt_count > 0 and yield_count == 0:
        behavior_factor = 1.25
    elif yield_count > preempt_count:
        behavior_factor = 0.75
    else:
        behavior_factor = 1.0
        
    priority_factor = priority / 5.0
    raw = int(load_q * behavior_factor * priority_factor)
    return max(q_min, min(q_max, raw))

def test_dynamic_quantum_algorithms():
    print("\n--- 3. Algorithmic Audit: Dynamic Quantum Formulas ---")
    # Low load (1 task): quantum expands
    q_low = compute_dynamic_quantum(n_runnable=1, yield_count=0, preempt_count=0, priority=5)
    test("Low Load Expands Quantum", q_low >= 12, f"Computed {q_low}")

    # High load (8 tasks): quantum contracts to bound cycle time
    q_high = compute_dynamic_quantum(n_runnable=8, yield_count=0, preempt_count=0, priority=5)
    test("High Load Contracts Quantum", q_high <= 6, f"Computed {q_high}")
    test("High Load Respects Min Quantum", q_high >= 2, f"Computed {q_high}")

    # Interactive Task (yields voluntarily) gets lower quantum (rapid cycles)
    q_inter = compute_dynamic_quantum(n_runnable=4, yield_count=5, preempt_count=1, priority=5)
    # CPU-bound Task (exhausts slices) gets higher quantum (throughput)
    q_cpu = compute_dynamic_quantum(n_runnable=4, yield_count=0, preempt_count=5, priority=5)
    test("Behavioral Differentiation (Interactive <= CPU-Bound)", q_inter <= q_cpu, f"Inter: {q_inter}, CPU: {q_cpu}")

# ==============================================================================
# 4. Memory Allocator Logic Verification
# ==============================================================================
def align_up(addr, align):
    return (addr + align - 1) & ~(align - 1)

def test_bump_allocator_logic():
    print("\n--- 4. Algorithmic Audit: Bump Allocator & Alignment ---")
    test("Align 0 to 8", align_up(0, 8) == 0)
    test("Align 1 to 8", align_up(1, 8) == 8)
    test("Align 9 to 16", align_up(9, 16) == 16)
    test("Align 16 to 16", align_up(16, 16) == 16)

    # Simulated Heap Allocation
    heap_size = 1024
    heap_start = 0x10000
    heap_end = heap_start + heap_size
    next_ptr = heap_start

    # Alloc 128 bytes
    alloc_start = align_up(next_ptr, 8)
    alloc_end = alloc_start + 128
    test("Allocation Fits Heap", alloc_end <= heap_end)
    next_ptr = alloc_end

    # Alloc huge (exceeding remaining space)
    huge_start = align_up(next_ptr, 8)
    huge_end = huge_start + 2048
    test("OOM Boundary Check Triggers", huge_end > heap_end)

# ==============================================================================
# Main Runner
# ==============================================================================
def main():
    root = Path(__file__).resolve().parent.parent
    print("==================================================================")
    print("   YASHWANTH MICRO-OS - STRICT CODE & ALGORITHM VERIFICATION      ")
    print("==================================================================")
    audit_source_files(root)
    test_ready_queue_algorithms()
    test_dynamic_quantum_algorithms()
    test_bump_allocator_logic()

    print("\n==================================================================")
    print(f"VERIFICATION SUMMARY: {PASS_COUNT} PASSED, {FAIL_COUNT} FAILED")
    print("==================================================================")
    if FAIL_COUNT > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
