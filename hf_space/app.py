"""
Yashwanth Micro-OS: Companion Educational Scheduler Simulator
Hugging Face Space Application

NOTE: This companion web application is an interactive educational scheduler
simulator designed for visual inspection and theoretical experimentation.
The genuine bare-metal operating system kernel is implemented in Rust (x86_64).
"""

import gradio as gr
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches

def simulate_scheduler(mode, base_quantum, target_cycle, t1_burst, t2_burst, t3_burst, t4_burst):
    tasks_input = [
        {"id": 1, "name": "Task-1-CPUBound", "priority": 5, "burst": int(t1_burst), "is_interactive": False},
        {"id": 2, "name": "Task-2-Interactive", "priority": 5, "burst": int(t2_burst), "is_interactive": True},
        {"id": 3, "name": "Task-3-SyncWorker", "priority": 8, "burst": int(t3_burst), "is_interactive": False},
        {"id": 4, "name": "Task-4-Diagnostics", "priority": 3, "burst": int(t4_burst), "is_interactive": False},
    ]

    # Initialize runtime state
    tasks = {
        t["id"]: {
            **t,
            "remaining": t["burst"],
            "waiting": 0,
            "turnaround": 0,
            "switches": 0,
            "preemptions": 0,
            "yields": 0,
            "current_quantum": base_quantum,
            "quantum_left": base_quantum,
        }
        for t in tasks_input
    }

    ready_queue = [t["id"] for t in tasks_input]
    current_task = None
    timeline = []
    total_ticks = 0
    max_ticks = 150

    while (ready_queue or current_task is not None) and total_ticks < max_ticks:
        total_ticks += 1

        # Increment waiting time for tasks in ready queue
        for q_id in ready_queue:
            tasks[q_id]["waiting"] += 1

        # Select task if none running
        if current_task is None:
            if not ready_queue:
                break
            current_task = ready_queue.pop(0)
            tasks[current_task]["switches"] += 1

            # Compute quantum
            if mode == "Dynamic Quantum (DQ-RR)":
                n = max(1, len(ready_queue) + 1)
                load_q = int(target_cycle / n)
                load_q = max(2, min(16, load_q))

                task_obj = tasks[current_task]
                behavior_factor = 0.75 if task_obj["yields"] > task_obj["preemptions"] else (1.25 if task_obj["preemptions"] > 0 else 1.0)
                priority_factor = task_obj["priority"] / 5.0
                q_calc = int(load_q * behavior_factor * priority_factor)
                q_calc = max(2, min(16, q_calc))
                tasks[current_task]["current_quantum"] = q_calc
                tasks[current_task]["quantum_left"] = q_calc
            else:
                tasks[current_task]["current_quantum"] = base_quantum
                tasks[current_task]["quantum_left"] = base_quantum

        # Execute running task
        t_obj = tasks[current_task]
        t_obj["remaining"] -= 1
        t_obj["quantum_left"] -= 1
        timeline.append((total_ticks, current_task, t_obj["name"]))

        # Check voluntary yield for interactive task
        yielded = False
        if t_obj["is_interactive"] and total_ticks % 3 == 0 and t_obj["remaining"] > 0:
            t_obj["yields"] += 1
            yielded = True

        # Check completion
        if t_obj["remaining"] == 0:
            t_obj["turnaround"] = total_ticks
            current_task = None
        elif yielded:
            ready_queue.append(current_task)
            current_task = None
        elif t_obj["quantum_left"] == 0:
            t_obj["preemptions"] += 1
            ready_queue.append(current_task)
            current_task = None

    # Prepare results table
    table_data = []
    total_wait = 0
    total_ta = 0
    total_switches = 0
    runtimes = []

    for t_id, data in tasks.items():
        table_data.append({
            "Task ID": data["id"],
            "Task Name": data["name"],
            "Priority": data["priority"],
            "Burst Ticks": data["burst"],
            "Waiting Time": data["waiting"],
            "Turnaround Time": data["turnaround"],
            "Context Switches": data["switches"],
            "Preemptions": data["preemptions"],
            "Voluntary Yields": data["yields"],
        })
        total_wait += data["waiting"]
        total_ta += data["turnaround"]
        total_switches += data["switches"]
        runtimes.append(data["burst"])

    df = pd.DataFrame(table_data)

    n_tasks = len(tasks)
    avg_wait = round(total_wait / n_tasks, 2)
    avg_ta = round(total_ta / n_tasks, 2)

    # Calculate Jain's Fairness Index
    sum_x = sum(runtimes)
    sum_sq = sum(x**2 for x in runtimes)
    fairness = round((sum_x**2) / (n_tasks * sum_sq), 4)

    summary_text = f"""
    ### 📊 Performance Summary
    - **Scheduling Algorithm:** `{mode}`
    - **Total Execution Ticks:** `{total_ticks}`
    - **Total Context Switches:** `{total_switches}`
    - **Average Waiting Time:** `{avg_wait}` ticks
    - **Average Turnaround Time:** `{avg_ta}` ticks
    - **Jain's Fairness Index:** `{fairness}` (1.0 = perfectly balanced)
    """

    # Generate Gantt Chart
    fig, ax = plt.subplots(figsize=(10, 3.2), dpi=120)
    color_map = {1: "#3498db", 2: "#2ecc71", 3: "#e67e22", 4: "#9b59b6"}

    for tick, t_id, _ in timeline:
        ax.barh(y=t_id, width=1, left=tick - 1, color=color_map.get(t_id, "#7f8c8d"), edgecolor="black", height=0.6)

    ax.set_yticks([1, 2, 3, 4])
    ax.set_yticklabels(["T1: CPUBound", "T2: Interactive", "T3: SyncWorker", "T4: Diagnostics"])
    ax.set_xlabel("Monotonic Simulation Ticks (1 tick ≈ 10 ms)")
    ax.set_title(f"CPU Scheduling Timeline ({mode})", fontsize=12, fontweight="bold")
    ax.set_xlim(0, max(20, total_ticks + 2))
    ax.grid(axis="x", linestyle="--", alpha=0.6)

    legend_patches = [
        mpatches.Patch(color="#3498db", label="T1: CPUBound (P5)"),
        mpatches.Patch(color="#2ecc71", label="T2: Interactive (P5)"),
        mpatches.Patch(color="#e67e22", label="T3: SyncWorker (P8)"),
        mpatches.Patch(color="#9b59b6", label="T4: Diagnostics (P3)"),
    ]
    ax.legend(handles=legend_patches, loc="upper right", fontsize=8)
    plt.tight_layout()

    return summary_text, df, fig

with gr.Blocks(title="Yashwanth Micro-OS: Dynamic Quantum Simulator") as demo:
    gr.Markdown("# 🚀 Yashwanth Micro-OS: Dynamic Quantum Scheduler Simulator")
    gr.Markdown(
        "> ⚠️ **Academic Companion Notice:** This interactive tool is a companion visual simulator. "
        "The actual operating system kernel is written in bare-metal **Rust (x86_64)** with Limine bootloader."
    )

    with gr.Row():
        with gr.Column(scale=1):
            mode_input = gr.Radio(
                ["Dynamic Quantum (DQ-RR)", "Static Round Robin (RR)"],
                value="Dynamic Quantum (DQ-RR)",
                label="Scheduling Mode",
            )
            quantum_input = gr.Slider(2, 12, value=6, step=1, label="Base Time Quantum (Ticks)")
            target_input = gr.Slider(10, 60, value=36, step=2, label="Target Cycle Time (Ticks)")

            gr.Markdown("### Workload Parameters (Burst in Ticks)")
            t1_burst = gr.Slider(5, 30, value=18, step=1, label="T1 (CPU-Bound) Burst")
            t2_burst = gr.Slider(3, 20, value=10, step=1, label="T2 (Interactive) Burst")
            t3_burst = gr.Slider(5, 25, value=14, step=1, label="T3 (Sync-Worker) Burst")
            t4_burst = gr.Slider(2, 15, value=6, step=1, label="T4 (Diagnostics) Burst")

            run_btn = gr.Button("Run Simulation", variant="primary")

        with gr.Column(scale=2):
            summary_output = gr.Markdown()
            gantt_output = gr.Plot(label="Scheduling Gantt Chart")
            table_output = gr.Dataframe(label="Task Metrics Table")

    run_btn.click(
        simulate_scheduler,
        inputs=[mode_input, quantum_input, target_input, t1_burst, t2_burst, t3_burst, t4_burst],
        outputs=[summary_output, table_output, gantt_output],
    )

if __name__ == "__main__":
    demo.launch()
