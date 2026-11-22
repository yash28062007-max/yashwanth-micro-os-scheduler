---
title: Yashwanth Micro-OS Dynamic Quantum Scheduler
emoji: ⏱️
colorFrom: blue
colorTo: indigo
sdk: gradio
sdk_version: 4.20.0
app_file: app.py
pinned: false
license: mit
---

# Yashwanth Micro-OS: Companion Scheduler Simulator

This interactive Hugging Face Space is an educational companion visualizer for the **Yashwanth Micro-OS** operating systems engineering project (CA-6).

> ⚠️ **DISCLAIMER:** This application is an interactive educational **scheduler simulator** running on Python/Gradio. The real educational operating system is a bare-metal **Rust kernel (x86_64)** running directly on Limine bootloader and QEMU.

## Features
- **Dynamic Quantum Round Robin (DQ-RR) vs Static Round Robin (RR)**
- Interactive task workload configuration (CPU-bound, interactive yields, synchronization workers)
- High-resolution scheduling timeline (Gantt chart)
- Metrics calculation: Waiting Time, Turnaround Time, Context-Switch Count, Preemptions, Jain's Fairness Index

## Setup Instructions for Local Execution
```bash
git clone https://huggingface.co/spaces/<your-username>/yashwanth-micro-os-scheduler
cd yashwanth-micro-os-scheduler
pip install -r requirements.txt
python app.py
```
Open your browser at `http://127.0.0.1:7860`.
