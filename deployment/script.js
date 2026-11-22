// Yashwanth Micro-OS: Companion Educational Scheduler Simulator
// Pure Client-Side JavaScript Engine

const TASK_CONFIGS = [
  { id: 1, name: "Task-1-CPUBound", priority: 5, color: "#38bdf8", isInteractive: false },
  { id: 2, name: "Task-2-Interactive", priority: 5, color: "#34d399", isInteractive: true },
  { id: 3, name: "Task-3-SyncWorker", priority: 8, color: "#fbbf24", isInteractive: false },
  { id: 4, name: "Task-4-Diagnostics", priority: 3, color: "#c084fc", isInteractive: false },
];

function initControls() {
  const sliders = [
    { id: "baseQuantum", valId: "valBaseQuantum" },
    { id: "targetCycle", valId: "valTargetCycle" },
    { id: "t1Burst", valId: "valT1Burst" },
    { id: "t2Burst", valId: "valT2Burst" },
    { id: "t3Burst", valId: "valT3Burst" },
    { id: "t4Burst", valId: "valT4Burst" },
  ];

  sliders.forEach(({ id, valId }) => {
    const el = document.getElementById(id);
    const valEl = document.getElementById(valId);
    if (el && valEl) {
      el.addEventListener("input", () => {
        valEl.textContent = el.value;
      });
    }
  });

  document.getElementById("btnRun").addEventListener("click", runSimulation);
  document.getElementById("btnReset").addEventListener("click", resetDefaults);

  // Run initial simulation on load
  runSimulation();
}

function resetDefaults() {
  document.getElementById("schedMode").value = "DQ-RR";
  document.getElementById("baseQuantum").value = 6;
  document.getElementById("valBaseQuantum").textContent = "6";
  document.getElementById("targetCycle").value = 36;
  document.getElementById("valTargetCycle").textContent = "36";
  
  document.getElementById("t1Burst").value = 18;
  document.getElementById("valT1Burst").textContent = "18";
  document.getElementById("t2Burst").value = 10;
  document.getElementById("valT2Burst").textContent = "10";
  document.getElementById("t3Burst").value = 14;
  document.getElementById("valT3Burst").textContent = "14";
  document.getElementById("t4Burst").value = 6;
  document.getElementById("valT4Burst").textContent = "6";

  runSimulation();
}

function runSimulation() {
  const mode = document.getElementById("schedMode").value;
  const baseQuantum = parseInt(document.getElementById("baseQuantum").value, 10);
  const targetCycle = parseInt(document.getElementById("targetCycle").value, 10);

  const bursts = {
    1: parseInt(document.getElementById("t1Burst").value, 10),
    2: parseInt(document.getElementById("t2Burst").value, 10),
    3: parseInt(document.getElementById("t3Burst").value, 10),
    4: parseInt(document.getElementById("t4Burst").value, 10),
  };

  // Initialize runtime state
  const tasks = {};
  TASK_CONFIGS.forEach((cfg) => {
    tasks[cfg.id] = {
      ...cfg,
      burst: bursts[cfg.id],
      remaining: bursts[cfg.id],
      waiting: 0,
      turnaround: 0,
      switches: 0,
      preemptions: 0,
      yields: 0,
      currentQuantum: baseQuantum,
      quantumLeft: baseQuantum,
    };
  });

  const readyQueue = [1, 2, 3, 4];
  let currentTask = null;
  const timeline = [];
  const logEvents = [];
  let totalTicks = 0;
  const maxTicks = 200;

  logEvents.push(`[SYSTEM] Initialized 4 tasks into ReadyQueue. Mode: ${mode}`);

  while ((readyQueue.length > 0 || currentTask !== null) && totalTicks < maxTicks) {
    totalTicks++;

    // Increment waiting time for tasks waiting in ready queue
    readyQueue.forEach((qId) => {
      tasks[qId].waiting++;
    });

    // Select next task if CPU is idle
    if (currentTask === null) {
      if (readyQueue.length === 0) break;
      currentTask = readyQueue.shift();
      tasks[currentTask].switches++;

      // Dynamic Quantum Computation
      if (mode === "DQ-RR") {
        const n = Math.max(1, readyQueue.length + 1);
        let loadQ = Math.floor(targetCycle / n);
        loadQ = Math.max(2, Math.min(16, loadQ));

        const tObj = tasks[currentTask];
        let behaviorFactor = 1.0;
        if (tObj.yields > tObj.preemptions) {
          behaviorFactor = 0.75; // Reward interactive tasks
        } else if (tObj.preemptions > 0) {
          behaviorFactor = 1.25; // Expand compute slices
        }

        const priorityFactor = tObj.priority / 5.0;
        let qCalc = Math.floor(loadQ * behaviorFactor * priorityFactor);
        qCalc = Math.max(2, Math.min(16, qCalc));

        tObj.currentQuantum = qCalc;
        tObj.quantumLeft = qCalc;
      } else {
        tasks[currentTask].currentQuantum = baseQuantum;
        tasks[currentTask].quantumLeft = baseQuantum;
      }

      logEvents.push(`[TICK ${String(totalTicks).padStart(3, '0')}] SWITCH -> ${tasks[currentTask].name} | Quantum: ${tasks[currentTask].currentQuantum} ticks`);
    }

    // Execute running task for 1 tick
    const tObj = tasks[currentTask];
    tObj.remaining--;
    tObj.quantumLeft--;
    timeline.push({ tick: totalTicks, taskId: currentTask, name: tObj.name, color: tObj.color });

    // Interactive cooperative yield check
    let yielded = false;
    if (tObj.isInteractive && totalTicks % 3 === 0 && tObj.remaining > 0) {
      tObj.yields++;
      yielded = true;
      logEvents.push(`[TICK ${String(totalTicks).padStart(3, '0')}] YIELD  <- ${tObj.name} voluntarily yields CPU`);
    }

    // Completion or preemption check
    if (tObj.remaining === 0) {
      tObj.turnaround = totalTicks;
      logEvents.push(`[TICK ${String(totalTicks).padStart(3, '0')}] FINISH ** ${tObj.name} completed total burst`);
      currentTask = null;
    } else if (yielded) {
      readyQueue.push(currentTask);
      currentTask = null;
    } else if (tObj.quantumLeft === 0) {
      tObj.preemptions++;
      logEvents.push(`[TICK ${String(totalTicks).padStart(3, '0')}] PREEMPT || ${tObj.name} exhausted quantum slice`);
      readyQueue.push(currentTask);
      currentTask = null;
    }
  }

  updateMetrics(tasks, totalTicks, mode);
  updateTable(tasks);
  drawTimeline(timeline, totalTicks);
  document.getElementById("traceLog").textContent = logEvents.join("\n");
}

function updateMetrics(tasks, totalTicks, mode) {
  let totalWait = 0;
  let totalTa = 0;
  let totalSwitches = 0;
  const runtimes = [];

  Object.values(tasks).forEach((t) => {
    totalWait += t.waiting;
    totalTa += t.turnaround;
    totalSwitches += t.switches;
    runtimes.push(t.burst);
  });

  const n = Object.keys(tasks).length;
  const avgWait = (totalWait / n).toFixed(2);
  const avgTa = (totalTa / n).toFixed(2);

  // Jain's Fairness Index
  const sumX = runtimes.reduce((a, b) => a + b, 0);
  const sumSq = runtimes.reduce((a, b) => a + b * b, 0);
  const fairness = (Math.pow(sumX, 2) / (n * sumSq)).toFixed(4);

  document.getElementById("metricTicks").textContent = totalTicks;
  document.getElementById("metricSwitches").textContent = totalSwitches;
  document.getElementById("metricAvgWait").textContent = `${avgWait}`;
  document.getElementById("metricAvgTa").textContent = `${avgTa}`;
  document.getElementById("metricFairness").textContent = fairness;
}

function updateTable(tasks) {
  const tbody = document.getElementById("tableBody");
  tbody.innerHTML = "";

  Object.values(tasks).forEach((t) => {
    const tr = document.createElement("tr");
    tr.innerHTML = `
      <td><strong>${t.id}</strong></td>
      <td><span class="badge-dot" style="background:${t.color};"></span>${t.name}</td>
      <td>${t.priority}</td>
      <td>${t.burst}</td>
      <td>${t.waiting}</td>
      <td>${t.turnaround}</td>
      <td>${t.switches}</td>
      <td>${t.preemptions}</td>
      <td>${t.yields}</td>
    `;
    tbody.appendChild(tr);
  });
}

function drawTimeline(timeline, totalTicks) {
  const canvas = document.getElementById("timelineCanvas");
  const ctx = canvas.getContext("2d");

  // Handle high-DPI displays
  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  ctx.scale(dpr, dpr);

  const width = rect.width;
  const height = rect.height;

  ctx.clearRect(0, 0, width, height);

  const margin = { top: 20, right: 30, bottom: 40, left: 110 };
  const plotWidth = width - margin.left - margin.right;
  const plotHeight = height - margin.top - margin.bottom;

  const rowHeight = plotHeight / 4;
  const taskYMap = { 1: 0, 2: 1, 3: 2, 4: 3 };

  // Draw Lane Backgrounds & Labels
  TASK_CONFIGS.forEach((cfg) => {
    const row = taskYMap[cfg.id];
    const y = margin.top + row * rowHeight;

    // Row alternating background
    ctx.fillStyle = row % 2 === 0 ? "rgba(255, 255, 255, 0.02)" : "transparent";
    ctx.fillRect(margin.left, y, plotWidth, rowHeight);

    // Row divider
    ctx.strokeStyle = "rgba(51, 65, 85, 0.6)";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(margin.left, y + rowHeight);
    ctx.lineTo(margin.left + plotWidth, y + rowHeight);
    ctx.stroke();

    // Row Label
    ctx.fillStyle = "#94a3b8";
    ctx.font = "11px monospace";
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    ctx.fillText(`T${cfg.id}: ${cfg.name.split("-")[2] || cfg.name}`, margin.left - 10, y + rowHeight / 2);
  });

  const maxAxisTicks = Math.max(20, totalTicks + 2);
  const tickWidth = plotWidth / maxAxisTicks;

  // Draw Vertical Grid Lines & Ticks
  const step = maxAxisTicks > 60 ? 10 : (maxAxisTicks > 30 ? 5 : 2);
  ctx.fillStyle = "#64748b";
  ctx.font = "10px monospace";
  ctx.textAlign = "center";
  ctx.textBaseline = "top";

  for (let t = 0; t <= maxAxisTicks; t += step) {
    const x = margin.left + t * tickWidth;

    ctx.strokeStyle = "rgba(51, 65, 85, 0.35)";
    ctx.beginPath();
    ctx.moveTo(x, margin.top);
    ctx.lineTo(x, margin.top + plotHeight);
    ctx.stroke();

    ctx.fillText(`${t}`, x, margin.top + plotHeight + 8);
  }

  // Draw X Axis Title
  ctx.fillStyle = "#94a3b8";
  ctx.font = "11px sans-serif";
  ctx.fillText("Execution Ticks (1 tick ≈ 10 ms)", margin.left + plotWidth / 2, height - 12);

  // Draw Execution Slices
  timeline.forEach(({ tick, taskId, color }) => {
    const row = taskYMap[taskId];
    const x = margin.left + (tick - 1) * tickWidth;
    const y = margin.top + row * rowHeight + 4;
    const barHeight = rowHeight - 8;

    ctx.fillStyle = color;
    ctx.fillRect(x, y, Math.max(1, tickWidth - 0.5), barHeight);
  });
}

window.addEventListener("DOMContentLoaded", initControls);
window.addEventListener("resize", () => {
  const canvas = document.getElementById("timelineCanvas");
  if (canvas) {
    runSimulation();
  }
});
