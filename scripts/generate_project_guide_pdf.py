import os
import sys
from reportlab.lib.pagesizes import letter
from reportlab.lib import colors
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import inch
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, PageBreak, KeepTogether
)
from reportlab.pdfgen import canvas

class NumberedCanvas(canvas.Canvas):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self._saved_page_states = []

    def showPage(self):
        self._saved_page_states.append(dict(self.__dict__))
        self._startPage()

    def save(self):
        num_pages = len(self._saved_page_states)
        for state in self._saved_page_states:
            self.__dict__.update(state)
            self.draw_page_decorations(num_pages)
            super().showPage()
        super().save()

    def draw_page_decorations(self, total_pages):
        self.saveState()
        self.setFont("Helvetica", 8)
        self.setFillColor(colors.HexColor("#718096"))
        
        # Header (pages 2 to total)
        if self._pageNumber > 1:
            self.drawString(45, 11 * inch - 36, "YASHWANTH MICRO-OS | YASHWANTH K")
            self.drawRightString(8.5 * inch - 45, 11 * inch - 36, f"Page {self._pageNumber}")
            self.setStrokeColor(colors.HexColor("#CBD5E0"))
            self.setLineWidth(0.5)
            self.line(45, 11 * inch - 42, 8.5 * inch - 45, 11 * inch - 42)

        # Footer (all pages)
        self.setStrokeColor(colors.HexColor("#CBD5E0"))
        self.setLineWidth(0.5)
        self.line(45, 45, 8.5 * inch - 45, 45)
        self.drawString(45, 32, "YASHWANTH MICRO-OS | YASHWANTH K")
        self.drawRightString(8.5 * inch - 45, 32, f"Page {self._pageNumber}")
        self.restoreState()

def build_pdf(filename):
    doc = SimpleDocTemplate(
        filename,
        pagesize=letter,
        leftMargin=45,
        rightMargin=45,
        topMargin=54,
        bottomMargin=54
    )

    styles = getSampleStyleSheet()

    # Custom styles
    title_style = ParagraphStyle(
        'CoverTitle',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=24,
        leading=28,
        textColor=colors.HexColor("#0F294A"),
        alignment=1, # Center
        spaceAfter=12
    )

    subtitle_style = ParagraphStyle(
        'CoverSubtitle',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=12,
        leading=16,
        textColor=colors.HexColor("#4A5568"),
        alignment=1,
        spaceAfter=30
    )

    prep_style = ParagraphStyle(
        'CoverPrep',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=10,
        leading=14,
        textColor=colors.HexColor("#718096"),
        alignment=1,
        spaceAfter=6
    )

    name_style = ParagraphStyle(
        'CoverName',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=18,
        leading=22,
        textColor=colors.HexColor("#0F294A"),
        alignment=1,
        spaceAfter=18
    )

    desc_style = ParagraphStyle(
        'CoverDesc',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=10,
        leading=15,
        textColor=colors.HexColor("#4A5568"),
        alignment=1,
        spaceAfter=36
    )

    h1_style = ParagraphStyle(
        'Heading1_Custom',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=13,
        leading=17,
        textColor=colors.HexColor("#0F294A"),
        spaceBefore=10,
        spaceAfter=5
    )

    h2_style = ParagraphStyle(
        'Heading2_Custom',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=10.5,
        leading=14,
        textColor=colors.HexColor("#1A365D"),
        spaceBefore=7,
        spaceAfter=3
    )

    body_style = ParagraphStyle(
        'Body_Custom',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=8.5,
        leading=11.5,
        textColor=colors.HexColor("#2D3748"),
        spaceAfter=4
    )

    bullet_style = ParagraphStyle(
        'Bullet_Custom',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=8.5,
        leading=11.5,
        textColor=colors.HexColor("#2D3748"),
        leftIndent=12,
        spaceAfter=2
    )

    code_style = ParagraphStyle(
        'Code_Custom',
        parent=styles['Normal'],
        fontName='Courier',
        fontSize=7.5,
        leading=10,
        textColor=colors.HexColor("#1A202C")
    )

    link_style = ParagraphStyle(
        'Link_Custom',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=8.5,
        leading=11.5,
        textColor=colors.HexColor("#2B6CB0"),
        spaceAfter=4
    )

    story = []

    # =========================================================================
    # PAGE 1: COVER
    # =========================================================================
    story.append(Spacer(1, 140))
    story.append(Paragraph("YASHWANTH MICRO-OS", title_style))
    story.append(Paragraph("Source Code to Bootable ISO – Project Guide", subtitle_style))
    story.append(Spacer(1, 20))
    story.append(Paragraph("Prepared for", prep_style))
    story.append(Paragraph("YASHWANTH K", name_style))
    story.append(Paragraph("Repository links, kernel architecture, dynamic quantum scheduler, ISO generation workflow, verification, and boot testing.", desc_style))
    
    story.append(Spacer(1, 40))

    cover_meta = [
        [Paragraph("<b>Project:</b>", body_style), Paragraph("Yashwanth Micro-OS (x86_64 Dynamic Quantum Kernel)", body_style)],
        [Paragraph("<b>Student Name:</b>", body_style), Paragraph("Yashwanth K (USN: 25BTRCL293) | Branch: CSE-AIML-E", body_style)],
        [Paragraph("<b>Course & Campus:</b>", body_style), Paragraph("25PCCS201 – Operating Systems | JAIN, FET Campus", body_style)],
        [Paragraph("<b>Source Code:</b>", body_style), Paragraph("<font color='#2B6CB0'><u>GitHub (yash28062007-max/yashwanth-micro-os-scheduler)</u></font>", body_style)],
        [Paragraph("<b>Interactive Space:</b>", body_style), Paragraph("<font color='#2B6CB0'><u>Hugging Face (Yashbot07/yashwanth-micro-os-scheduler)</u></font>", body_style)],
    ]
    t_cover = Table(cover_meta, colWidths=[130, 310])
    t_cover.setStyle(TableStyle([
        ('ALIGN', (0,0), (-1,-1), 'LEFT'),
        ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
        ('BOTTOMPADDING', (0,0), (-1,-1), 4),
        ('TOPPADDING', (0,0), (-1,-1), 4),
        ('LEFTPADDING', (0,0), (-1,-1), 8),
        ('RIGHTPADDING', (0,0), (-1,-1), 8),
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#F7FAFC")),
        ('BOX', (0,0), (-1,-1), 0.5, colors.HexColor("#E2E8F0")),
        ('INNERGRID', (0,0), (-1,-1), 0.5, colors.HexColor("#EDF2F7")),
    ]))
    story.append(t_cover)
    story.append(PageBreak())

    # =========================================================================
    # PAGE 2: PROJECT LINKS, OVERVIEW & REQUIREMENTS
    # =========================================================================
    story.append(Paragraph("1. Project Links", h1_style))
    story.append(Paragraph("Use the following links to access the source code, reports, and interactive deployment:", body_style))
    story.append(Paragraph("<b>GitHub – Source code & Academic Reports:</b>", body_style))
    story.append(Paragraph('<a href="https://github.com/yash28062007-max/yashwanth-micro-os-scheduler"><font color="#2B6CB0"><u>https://github.com/yash28062007-max/yashwanth-micro-os-scheduler</u></font></a>', link_style))
    story.append(Paragraph("<b>Hugging Face – Interactive Scheduler Simulator Space:</b>", body_style))
    story.append(Paragraph('<a href="https://huggingface.co/spaces/Yashbot07/yashwanth-micro-os-scheduler"><font color="#2B6CB0"><u>https://huggingface.co/spaces/Yashbot07/yashwanth-micro-os-scheduler</u></font></a>', link_style))
    story.append(Paragraph("The GitHub repository contains the full bare-metal Rust kernel, Limine bootloader configuration, automated test suites, academic continuous assessment reports (CA-2, CA-3, CA-5), and the companion desktop environment. The Hugging Face Space hosts an interactive Python/Gradio scheduler simulator.", body_style))

    story.append(Paragraph("2. What “Source Code to Bootable ISO” Means", h1_style))
    story.append(Paragraph("The source repository contains the complete low-level code required to compile a bare-metal kernel and assemble a bootable ISO image. The build process compiles a Rust <code>no_std</code> kernel targeting <code>x86_64-unknown-none</code>, links it with a specialized linker script, arranges the Limine bootloader assets, and uses <code>xorriso</code> to assemble a hybrid ISO bootable on both legacy BIOS and modern UEFI hardware.", body_style))

    box_data = [[Paragraph("<b>Important:</b> The repository generates a standalone bootable image (<code>yashwanth_micro_os.iso</code>, ~2.5 MB). It executes directly on bare-metal x86_64 hardware or hypervisors (VirtualBox, QEMU) without depending on an existing Linux distribution.", body_style)]]
    t_box = Table(box_data, colWidths=[510])
    t_box.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#FEFCBF")),
        ('BOX', (0,0), (-1,-1), 1, colors.HexColor("#D69E2E")),
        ('TOPPADDING', (0,0), (-1,-1), 6),
        ('BOTTOMPADDING', (0,0), (-1,-1), 6),
        ('LEFTPADDING', (0,0), (-1,-1), 10),
        ('RIGHTPADDING', (0,0), (-1,-1), 10),
    ]))
    story.append(t_box)
    story.append(Spacer(1, 4))

    story.append(Paragraph("3. Requirements", h1_style))
    story.append(Paragraph("Prepare an environment with the following toolchain packages:", body_style))
    story.append(Paragraph("• <b>Rust Nightly Toolchain:</b> <code>rustc</code> and <code>cargo</code> with <code>llvm-tools-preview</code> and <code>rust-src</code> component.", bullet_style))
    story.append(Paragraph("• <b>LLVM Linker:</b> <code>lld</code> for linking 64-bit ELF binaries at high-memory addresses.", bullet_style))
    story.append(Paragraph("• <b>ISO Assembly Tools:</b> <code>xorriso</code> and <code>mtools</code> for creating hybrid El Torito ISOs.", bullet_style))
    story.append(Paragraph("• <b>Hypervisors:</b> <code>qemu-system-x86_64</code> or Oracle VM VirtualBox (64-bit).", bullet_style))

    story.append(Paragraph("Install required build packages on Ubuntu / Debian:", h2_style))
    cmd_box1 = [[Paragraph("sudo apt update<br/>sudo apt install -y git build-essential xorriso mtools qemu-system-x86 curl<br/>curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly", code_style)]]
    t_cmd1 = Table(cmd_box1, colWidths=[510])
    t_cmd1.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#EDF2F7")),
        ('BOX', (0,0), (-1,-1), 0.5, colors.HexColor("#CBD5E0")),
        ('PADDING', (0,0), (-1,-1), 5),
    ]))
    story.append(t_cmd1)
    story.append(Spacer(1, 4))

    story.append(Paragraph("4. Download the Source Code", h1_style))
    story.append(Paragraph("Clone the GitHub repository to your local machine:", body_style))
    cmd_box2 = [[Paragraph("git clone https://github.com/yash28062007-max/yashwanth-micro-os-scheduler.git<br/>cd yashwanth-micro-os-scheduler<br/>ls -la", code_style)]]
    t_cmd2 = Table(cmd_box2, colWidths=[510])
    t_cmd2.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#EDF2F7")),
        ('BOX', (0,0), (-1,-1), 0.5, colors.HexColor("#CBD5E0")),
        ('PADDING', (0,0), (-1,-1), 5),
    ]))
    story.append(t_cmd2)
    story.append(PageBreak())

    # =========================================================================
    # PAGE 3: KERNEL ARCHITECTURE, SCHEDULER & BUILD WORKFLOW
    # =========================================================================
    story.append(Paragraph("5. Kernel Architecture & Hardware Subsystems", h1_style))
    story.append(Paragraph("The Yashwanth Micro-OS kernel is implemented in Rust (<code>no_std</code>) with direct bare-metal hardware control:", body_style))
    story.append(Paragraph("• <b>Global Descriptor Table (GDT):</b> Initialized with 64-bit Kernel Code (0x08) and Kernel Data (0x10) selectors.", bullet_style))
    story.append(Paragraph("• <b>Interrupt Descriptor Table (IDT):</b> 256 vector handlers mapped via the <code>x86_64</code> crate.", bullet_style))
    story.append(Paragraph("• <b>Dual 8259 PIC:</b> Hardware IRQs 0–15 remapped to interrupt vectors 32–47 to avoid CPU exception collisions.", bullet_style))
    story.append(Paragraph("• <b>Intel 8254 PIT Timer:</b> Configured in Mode 2 (rate generator) at 100 Hz (10 ms ticks on IRQ0) driving preemption.", bullet_style))
    story.append(Paragraph("• <b>Memory Management:</b> 128 KiB bump allocator protected by atomic SpinLocks ensuring lock safety.", bullet_style))
    story.append(Paragraph("• <b>Linear Framebuffer:</b> Reads Limine framebuffer structure and renders text with an embedded 8x16 font.", bullet_style))

    story.append(Paragraph("6. Dynamic Quantum Round-Robin (DQ-RR) Scheduler", h1_style))
    story.append(Paragraph("Standard Round Robin with static time slices suffers from high turnaround time on CPU-bound workloads and sluggish response times on interactive tasks. The Dynamic Quantum Scheduler solves this through runtime adaptation:", body_style))
    story.append(Paragraph("• <b>Quantum Calculation:</b> <code>DQ = clamp(round(base_quantum * (remaining_burst / mean_burst)), 4, 20) ticks</code>.", bullet_style))
    story.append(Paragraph("• <b>CPU-Bound Tasks:</b> Allocated extended time quanta (up to 20 ticks) to maximize CPU execution and reduce context switches.", bullet_style))
    story.append(Paragraph("• <b>I/O-Interactive Tasks:</b> When a task voluntarily yields before quantum expiry, its quantum is recalibrated for fast responsiveness.", bullet_style))
    story.append(Paragraph("• <b>Task States:</b> TCB tracks Ready, Running, Blocked, and Terminated states with priority levels (1–10).", bullet_style))

    story.append(Paragraph("7. Run the ISO Build Pipeline", h1_style))
    story.append(Paragraph("The kernel is compiled and packaged into a bootable ISO using the following steps:", body_style))
    
    cmd_box3 = [[Paragraph(
        "# 1. Compile the bare-metal kernel with nightly Rust<br/>"
        "cargo +nightly build --release --target x86_64-unknown-none -Zbuild-std=core,alloc<br/><br/>"
        "# 2. Prepare ISO root staging directory<br/>"
        "mkdir -p build/iso_root/boot/limine<br/>"
        "cp target/x86_64-unknown-none/release/yashwanth_micro_os build/iso_root/boot/<br/>"
        "cp limine.conf build/iso_root/boot/limine/<br/>"
        "cp limine/limine-bios.sys limine/limine-bios-cd.bin limine/limine-uefi-cd.bin build/iso_root/boot/limine/<br/><br/>"
        "# 3. Assemble hybrid El Torito ISO with xorriso<br/>"
        "xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 \\<br/>"
        "  -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image \\<br/>"
        "  --protective-msdos-label build/iso_root -o build/yashwanth_micro_os.iso<br/><br/>"
        "# 4. Install Limine BIOS stages into MBR sectors<br/>"
        "./limine bios-install build/yashwanth_micro_os.iso",
        code_style
    )]]
    t_cmd3 = Table(cmd_box3, colWidths=[510])
    t_cmd3.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#EDF2F7")),
        ('BOX', (0,0), (-1,-1), 0.5, colors.HexColor("#CBD5E0")),
        ('PADDING', (0,0), (-1,-1), 5),
    ]))
    story.append(t_cmd3)
    story.append(PageBreak())

    # =========================================================================
    # PAGE 4: VERIFICATION, TESTING, DESKTOP ENVIRONMENT & HUGGING FACE
    # =========================================================================
    story.append(Paragraph("8. Verify the Generated ISO", h1_style))
    story.append(Paragraph("Confirm the ISO exists and verify its hybrid boot sectors:", body_style))
    cmd_box4 = [[Paragraph("ls -lh build/yashwanth_micro_os.iso<br/>file build/yashwanth_micro_os.iso<br/>sha256sum build/yashwanth_micro_os.iso", code_style)]]
    t_cmd4 = Table(cmd_box4, colWidths=[510])
    t_cmd4.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#EDF2F7")),
        ('BOX', (0,0), (-1,-1), 0.5, colors.HexColor("#CBD5E0")),
        ('PADDING', (0,0), (-1,-1), 5),
    ]))
    story.append(t_cmd4)
    story.append(Spacer(1, 4))

    story.append(Paragraph("9. Test the ISO in a Virtual Machine (VirtualBox & QEMU)", h1_style))
    story.append(Paragraph("Verify the operating system in emulation and hypervisors:", body_style))
    story.append(Paragraph("1. <b>QEMU Emulation:</b> Run <code>qemu-system-x86_64 -cdrom build/yashwanth_micro_os.iso -serial stdio</code>.", bullet_style))
    story.append(Paragraph("2. <b>VirtualBox 7.x Setup:</b> Create a 64-bit VM (512 MB RAM, 1 CPU), attach <code>yashwanth_micro_os.iso</code> to the virtual IDE controller.", bullet_style))
    story.append(Paragraph("3. <b>Verified Boot Output:</b> The Limine bootloader presents a 3-second selection countdown, transfers control to <code>yashwanth_micro_os</code>, initializes PIT 8254 timer interrupts, and runs 4 concurrent tasks to completion.", bullet_style))

    story.append(Paragraph("10. MicroOS Companion Desktop Environment", h1_style))
    story.append(Paragraph("Alongside the bare-metal kernel, an authentic Windows 98 / Retro Motif desktop environment was developed in <code>desktop_environment/index.html</code>:", body_style))
    story.append(Paragraph("• <b>Developer Terminal (/bin/sh):</b> Live command execution, POSIX session details (<code>who am i</code> → <code>yeshwanth</code>), process table (<code>ps</code>, <code>top</code>), inline calculator (<code>calc</code>), and documentation (<code>help wifi</code>, <code>help snap</code>).", bullet_style))
    story.append(Paragraph("• <b>Real Hardware Wi-Fi Controller:</b> Reads host network interface (Realtek 8822CE Wireless LAN 802.11ac) and displays connected status for <code>MOJO-JAN-B-F5</code> with dynamic network switching.", bullet_style))
    story.append(Paragraph("• <b>Snap Store & Games:</b> Ubuntu Snap software center plus 6 playable arcade games (Snake, Tetris, 2048, Minesweeper, Pong).", bullet_style))

    story.append(Paragraph("11. Use the Published Space on Hugging Face", h1_style))
    story.append(Paragraph("The educational companion scheduler simulator is packaged in <code>hf_space/</code> and deployed to Hugging Face Spaces:", body_style))
    story.append(Paragraph('<a href="https://huggingface.co/spaces/Yashbot07/yashwanth-micro-os-scheduler"><font color="#2B6CB0"><u>https://huggingface.co/spaces/Yashbot07/yashwanth-micro-os-scheduler</u></font></a>', link_style))
    story.append(Paragraph("The Space allows users to configure task burst distributions, adjust dynamic quantum scaling parameters, and compare Gantt charts, waiting times, and fairness metrics interactively in their web browser.", body_style))

    story.append(Paragraph("12. Troubleshooting", h1_style))
    story.append(Paragraph("• <b>VirtualBox VBoxSVC lock:</b> If the VM is locked with 0x80bb0007, terminate background <code>VBoxSVC.exe</code> and power on cleanly.", bullet_style))
    story.append(Paragraph("• <b>Rust compilation errors:</b> Ensure the nightly compiler is active: <code>rustup default nightly</code>.", bullet_style))
    story.append(PageBreak())

    # =========================================================================
    # PAGE 5: CA REPORTS & SUMMARY
    # =========================================================================
    story.append(Paragraph("13. Multi-Core Scheduling Continuous Assessment (CA) Reports", h1_style))
    story.append(Paragraph("All academic investigations and lab reports have been organized in the <code>reports/</code> directory on GitHub:", body_style))

    ca_table_data = [
        [Paragraph("<b>Report File</b>", body_style), Paragraph("<b>Assessment</b>", body_style), Paragraph("<b>Key Investigation Topic</b>", body_style)],
        [Paragraph("<b>CA2_Multi_Core_Scheduling.pdf</b>", code_style), Paragraph("CA-2 Investigation", body_style), Paragraph("Multi-core execution performance: CPU affinity vs workload distribution.", body_style)],
        [Paragraph("<b>CA3_Multi_Core_Scheduling.pdf</b>", code_style), Paragraph("CA-3 Debugging", body_style), Paragraph("Root-cause diagnosis of static block partitioning defect & dynamic queue repair.", body_style)],
        [Paragraph("<b>CA5_Multi_Core_Scheduling.pdf</b>", code_style), Paragraph("CA-5 Systems Eng.", body_style), Paragraph("Linux scheduler telemetry, /proc context switches, and CPU affinity controls.", body_style)],
        [Paragraph("<b>OS_Lab_Report_Yashwanth_K.pdf</b>", code_style), Paragraph("OS Lab Report", body_style), Paragraph("Comprehensive laboratory report and theoretical OS derivations.", body_style)],
    ]
    t_ca = Table(ca_table_data, colWidths=[150, 95, 265])
    t_ca.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,0), colors.HexColor("#0F294A")),
        ('TEXTCOLOR', (0,0), (-1,0), colors.white),
        ('ALIGN', (0,0), (-1,-1), 'LEFT'),
        ('VALIGN', (0,0), (-1,-1), 'MIDDLE'),
        ('BOTTOMPADDING', (0,0), (-1,-1), 4),
        ('TOPPADDING', (0,0), (-1,-1), 4),
        ('LEFTPADDING', (0,0), (-1,-1), 6),
        ('RIGHTPADDING', (0,0), (-1,-1), 6),
        ('GRID', (0,0), (-1,-1), 0.5, colors.HexColor("#CBD5E0")),
        ('ROWBACKGROUNDS', (0,1), (-1,-1), [colors.white, colors.HexColor("#F7FAFC")]),
    ]))
    # Quick fix for table header text color
    for i in range(3):
        t_ca._cellvalues[0][i].style.textColor = colors.white
    story.append(t_ca)
    story.append(Spacer(1, 6))

    story.append(Paragraph("14. Summary", h1_style))
    story.append(Paragraph("The Yashwanth Micro-OS project bridges low-level bare-metal kernel engineering with empirical systems research. It delivers an x86_64 Long Mode operating system running a Dynamic Quantum Round-Robin scheduler, complete automated verification suites, a retro companion desktop environment, and empirical Linux multi-core performance investigations.", body_style))
    story.append(Paragraph("<b>Project Owner:</b> YASHWANTH K (USN: 25BTRCL293 | CSE-AIML-E)", body_style))

    story.append(Paragraph("<b>Reference files checked:</b>", body_style))
    story.append(Paragraph("• Kernel source: <code>src/main.rs</code>, <code>src/scheduler/round_robin.rs</code>, <code>src/task/task.rs</code>", bullet_style))
    story.append(Paragraph("• Boot & Linker: <code>limine.conf</code>, <code>linker.ld</code>, <code>build/yashwanth_micro_os.iso</code>", bullet_style))
    story.append(Paragraph("• Desktop & GUI: <code>desktop_environment/index.html</code> (Wi-Fi, Terminal, Snap Store)", bullet_style))
    story.append(Paragraph("• Simulator Package: <code>hf_space/app.py</code>, <code>hf_space/README.md</code>", bullet_style))

    story.append(Paragraph("15. Project Links", h1_style))
    story.append(Paragraph("GitHub Source Code & Reports Repository:", body_style))
    story.append(Paragraph('<a href="https://github.com/yash28062007-max/yashwanth-micro-os-scheduler"><font color="#2B6CB0"><u>https://github.com/yash28062007-max/yashwanth-micro-os-scheduler</u></font></a>', link_style))
    story.append(Paragraph("GitHub Academic Reports Folder:", body_style))
    story.append(Paragraph('<a href="https://github.com/yash28062007-max/yashwanth-micro-os-scheduler/tree/main/reports"><font color="#2B6CB0"><u>https://github.com/yash28062007-max/yashwanth-micro-os-scheduler/tree/main/reports</u></font></a>', link_style))
    story.append(Paragraph("Hugging Face Interactive Simulator Space:", body_style))
    story.append(Paragraph('<a href="https://huggingface.co/spaces/Yashbot07/yashwanth-micro-os-scheduler"><font color="#2B6CB0"><u>https://huggingface.co/spaces/Yashbot07/yashwanth-micro-os-scheduler</u></font></a>', link_style))

    story.append(Spacer(1, 10))
    story.append(Paragraph("<i>Note: This guide is generated based on verified laboratory benchmarks, compiled ISO builds, and Git commits as of November 2026.</i>", ParagraphStyle('Note', parent=body_style, textColor=colors.HexColor("#718096"), fontSize=7.5)))

    # Build document
    doc.build(story, canvasmaker=NumberedCanvas)
    print(f"PDF successfully built: {filename}")

if __name__ == "__main__":
    out_pdf = r"C:\Users\yash2\OneDrive\Desktop\Yashwanth_Micro_OS_Project_Guide.pdf"
    build_pdf(out_pdf)
