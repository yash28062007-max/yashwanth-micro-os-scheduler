import sys
import os

sys.path.insert(0, os.path.abspath("hf_space"))

try:
    import app
    print("[PASS] app.py imported successfully")
except Exception as e:
    print(f"[FAIL] import failed: {e}")
    sys.exit(1)

# Test simulate_scheduler with Dynamic Quantum mode
try:
    summary_text, df, fig = app.simulate_scheduler(
        "Dynamic Quantum (DQ-RR)",
        6, 36, 18, 10, 14, 6
    )
    print("[PASS] simulate_scheduler (Dynamic Quantum) executed successfully")
    print(f"Task DataFrame Shape: {df.shape}")
    print("\nSummary preview:")
    print(summary_text[:250])
except Exception as e:
    print(f"[FAIL] execution failed: {e}")
    sys.exit(1)

# Also test Static Round Robin mode
try:
    summary_text, df, fig = app.simulate_scheduler(
        "Static Round Robin (RR)",
        6, 36, 18, 10, 14, 6
    )
    print("\n[PASS] simulate_scheduler (Static Round Robin) executed successfully")
    print(f"Task DataFrame Shape: {df.shape}")
except Exception as e:
    print(f"[FAIL] Static mode failed: {e}")
    sys.exit(1)

print("\n[ALL TESTS PASSED FOR HF GRADIO SIMULATOR]")
