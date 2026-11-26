import os
import sys
from reportlab.lib.pagesizes import letter
from reportlab.lib import colors
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import inch
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, PageBreak
)
from reportlab.pdfgen import canvas
from pypdf import PdfReader

class ExactNumberedCanvas(canvas.Canvas):
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
        self.setFont("Helvetica", 8.5)
        self.setFillColor(colors.HexColor("#718096"))
        
        # Header (pages 2 to total) - No horizontal line, matching reference screenshot exactly
        if self._pageNumber > 1:
            self.drawString(54, 11 * inch - 36, "YASHWANTH MICRO-OS | YASHWANTH K")
            self.drawRightString(8.5 * inch - 54, 11 * inch - 36, f"Page {self._pageNumber}")

        # Footer (all pages) - Thin rule with text below, matching reference screenshot exactly
        self.setStrokeColor(colors.HexColor("#CBD5E0"))
        self.setLineWidth(0.5)
        self.line(54, 45, 8.5 * inch - 54, 45)
        self.drawString(54, 32, "YASHWANTH MICRO-OS | YASHWANTH K")
        self.drawRightString(8.5 * inch - 54, 32, f"Page {self._pageNumber}")
        self.restoreState()

def make_callout(text, width=504):
    p = Paragraph(f"<b>Important:</b> {text}", ParagraphStyle(
        'CalloutText',
        fontName='Helvetica',
        fontSize=8.5,
        leading=12,
        textColor=colors.HexColor("#2D3748")
    ))
    t = Table([[p]], colWidths=[width])
    t.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#FFF5EB")),
        ('BOX', (0,0), (-1,-1), 0.75, colors.HexColor("#FBD38D")),
        ('TOPPADDING', (0,0), (-1,-1), 6),
        ('BOTTOMPADDING', (0,0), (-1,-1), 6),
        ('LEFTPADDING', (0,0), (-1,-1), 8),
        ('RIGHTPADDING', (0,0), (-1,-1), 8),
    ]))
    return t

def make_code_box(code_text, width=504):
    p = Paragraph(code_text, ParagraphStyle(
        'CodeBlock',
        fontName='Courier',
        fontSize=8,
        leading=11,
        textColor=colors.HexColor("#1A202C")
    ))
    t = Table([[p]], colWidths=[width])
    t.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,-1), colors.HexColor("#F7FAFC")),
        ('BOX', (0,0), (-1,-1), 0.5, colors.HexColor("#E2E8F0")),
        ('TOPPADDING', (0,0), (-1,-1), 5),
        ('BOTTOMPADDING', (0,0), (-1,-1), 5),
        ('LEFTPADDING', (0,0), (-1,-1), 8),
        ('RIGHTPADDING', (0,0), (-1,-1), 8),
    ]))
    return t

def generate_pdf(output_path):
    doc = SimpleDocTemplate(
        output_path,
        pagesize=letter,
        leftMargin=54,
        rightMargin=54,
        topMargin=50,
        bottomMargin=54
    )

    styles = getSampleStyleSheet()

    # Typography matching reference PDF
    title_style = ParagraphStyle(
        'CoverTitle',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=24,
        leading=28,
        textColor=colors.HexColor("#0B2545"),
        alignment=1, # Center
        spaceAfter=10
    )

    subtitle_style = ParagraphStyle(
        'CoverSubtitle',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=12,
        leading=16,
        textColor=colors.HexColor("#4A5568"),
        alignment=1,
        spaceAfter=36
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
        fontSize=20,
        leading=24,
        textColor=colors.HexColor("#0B2545"),
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
        spaceAfter=40
    )

    meta_style = ParagraphStyle(
        'CoverMeta',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=10,
        leading=16,
        textColor=colors.HexColor("#2D3748"),
        alignment=1
    )

    h1_style = ParagraphStyle(
        'Heading1',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=13,
        leading=16,
        textColor=colors.HexColor("#0B2545"),
        spaceBefore=8,
        spaceAfter=4
    )

    h2_blue = ParagraphStyle(
        'Heading2Blue',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=10.5,
        leading=14,
        textColor=colors.HexColor("#1B4D89"),
        spaceBefore=6,
        spaceAfter=3
    )

    body_style = ParagraphStyle(
        'Body',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=8.5,
        leading=11.5,
        textColor=colors.HexColor("#2D3748"),
        spaceAfter=4
    )

    bullet_style = ParagraphStyle(
        'Bullet',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=8.5,
        leading=11.5,
        textColor=colors.HexColor("#2D3748"),
        leftIndent=10,
        spaceAfter=2
    )

    link_style = ParagraphStyle(
        'Link',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=8.5,
        leading=11.5,
        textColor=colors.HexColor("#2B6CB0"),
        spaceAfter=3
    )

    story = []

    # =========================================================================
    # PAGE 1: COVER
    # =========================================================================
    story.append(Spacer(1, 150))
    story.append(Paragraph("YASHWANTH MICRO-OS", title_style))
    story.append(Paragraph("Source Code to Bootable ISO – Project Guide", subtitle_style))
    
    story.append(Paragraph("Prepared for", prep_style))
    story.append(Paragraph("YASHWANTH K", name_style))
    story.append(Paragraph("Repository links, build overview, ISO generation workflow, verification, and boot testing.", desc_style))
    
    story.append(Spacer(1, 40))
    meta_text = (
        "<b>Project:</b> Yashwanth Micro-OS (x86_64 Dynamic Quantum Kernel)<br/>"
        "<b>Source:</b> GitHub<br/>"
        "<b>Published ISO:</b> Hugging Face"
    )
    story.append(Paragraph(meta_text, meta_style))
    story.append(PageBreak())

    # =========================================================================
    # PAGE 2: PROJECT LINKS, OVERVIEW & REQUIREMENTS
    # =========================================================================
    story.append(Paragraph("1. Project Links", h1_style))
    story.append(Paragraph("Use the following links to access the source code and the already-published ISO image:", body_style))
    
    story.append(Paragraph("<b>GitHub – Source code:</b>", body_style))
    story.append(Paragraph('<a href="https://github.com/yash28062007-max/yashwanth-micro-os-scheduler"><font color="#2B6CB0"><u>https://github.com/yash28062007-max/yashwanth-micro-os-scheduler</u></font></a>', link_style))
    
    story.append(Paragraph("<b>Hugging Face – Published ISO files:</b>", body_style))
    story.append(Paragraph('<a href="https://huggingface.co/Yashbot07/yashwanth-microos-iso/tree/main"><font color="#2B6CB0"><u>https://huggingface.co/Yashbot07/yashwanth-microos-iso/tree/main</u></font></a>', link_style))
    
    story.append(Paragraph("<b>Hugging Face – Interactive Scheduler Simulator Space:</b>", body_style))
    story.append(Paragraph('<a href="https://huggingface.co/spaces/Yashbot07/yashwanth-micro-os-scheduler"><font color="#2B6CB0"><u>https://huggingface.co/spaces/Yashbot07/yashwanth-micro-os-scheduler</u></font></a>', link_style))
    
    story.append(Paragraph(
        "The GitHub README identifies the project as a bare-metal Rust operating system for x86_64 computers, intended "
        "to support both UEFI and legacy BIOS boot. The Hugging Face repository currently lists a file named "
        "<b>yashwanth_micro_os.iso</b> (about 2.5 MB). The Space provides the zero-install interactive scheduler simulator.",
        body_style
    ))

    story.append(Paragraph("2. What “Source Code to ISO” Means", h1_style))
    story.append(Paragraph(
        "The source repository contains scripts and project files used to prepare the bare-metal kernel, arrange "
        "Limine bootloader files, compile the Rust <code>no_std</code> kernel targeting <code>x86_64-unknown-none</code>, "
        "and package everything into an ISO image. The ISO is the bootable image that can be tested in a virtual "
        "machine (VirtualBox, QEMU) or written to a USB drive.",
        body_style
    ))

    story.append(make_callout(
        "the repository's <b>build_iso.sh</b> compiles the bare-metal x86_64 Rust kernel and integrates Limine bootloader "
        "stages. It expects a nightly Rust toolchain with <code>rust-src</code> and <code>llvm-tools</code> installed. "
        "Check and adapt environment paths for your Linux VM or WSL environment before running it."
    ))

    story.append(Paragraph("3. Requirements", h1_style))
    story.append(Paragraph(
        "Prepare a Linux environment (Ubuntu/Debian VM or WSL2) with sufficient free disk space. The build script "
        "uses low-level compilation and packaging tools.",
        body_style
    ))
    story.append(Paragraph("Main tools referenced by the project build script:", body_style))
    story.append(Paragraph("• <b>Git</b> – download the source repository.", bullet_style))
    story.append(Paragraph("• <b>Rust Nightly</b> – compile <code>no_std</code> kernel with <code>-Zbuild-std=core,alloc</code>.", bullet_style))
    story.append(Paragraph("• <b>xorriso & mtools</b> – assemble the hybrid BIOS + UEFI ISO image.", bullet_style))
    story.append(Paragraph("• <b>Limine Bootloader</b> – provide EFI and BIOS bootloader stages.", bullet_style))
    story.append(Paragraph("• <b>md5sum and sha256sum</b> – generate checksum records.", bullet_style))
    story.append(Paragraph("• <b>QEMU / VirtualBox</b> – hypervisor testing and verification.", bullet_style))

    story.append(Paragraph("Install common packages on Debian/Ubuntu:", h2_blue))
    story.append(make_code_box(
        "sudo apt update<br/>"
        "sudo apt install -y git build-essential xorriso mtools qemu-system-x86 curl<br/>"
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly"
    ))
    story.append(Paragraph(
        "Package names and toolchain paths can vary by Linux distribution. If the build reports a missing component, confirm the package and path.",
        ParagraphStyle('Note1', parent=body_style, fontSize=7.5, leading=10, textColor=colors.HexColor("#718096"))
    ))

    story.append(Paragraph("4. Download the Source Code", h1_style))
    story.append(Paragraph("Open a terminal in the Linux VM and clone the repository:", body_style))
    story.append(make_code_box(
        "git clone https://github.com/yash28062007-max/yashwanth-micro-os-scheduler.git<br/>"
        "cd yashwanth-micro-os-scheduler<br/>"
        "ls -la"
    ))
    story.append(Paragraph("Read the project instructions and inspect the build-related files:", body_style))
    story.append(make_code_box(
        "less README.md<br/>"
        "less limine.conf<br/>"
        "cargo --version"
    ))
    story.append(Paragraph(
        "The README explains the kernel architecture, Dynamic Quantum Round-Robin scheduler, and how to boot the ISO in QEMU.",
        body_style
    ))
    story.append(PageBreak())

    # =========================================================================
    # PAGE 3: KERNEL ARCHITECTURE, WORKSPACE & BUILD
    # =========================================================================
    story.append(Paragraph("5. Understand the Build Script Before Running It", h1_style))
    story.append(Paragraph("The current project uses these expected locations:", body_style))
    story.append(make_code_box(
        "KERNEL_SRC=src/main.rs<br/>"
        "SCHEDULER=src/scheduler/round_robin.rs<br/>"
        "TASK_MGMT=src/task/task.rs<br/>"
        "LIMINE_CFG=limine.conf<br/>"
        "LINKER_SCRIPT=linker.ld<br/>"
        "OUT_ISO=build/yashwanth_micro_os.iso"
    ))
    story.append(Paragraph(
        "This means the build needs the Rust source tree, Limine configuration, and bootloader binary assets under <code>limine/</code>. "
        "A normal compiler without the bare-metal target specification cannot assemble the kernel.",
        body_style
    ))
    
    story.append(Paragraph("The script’s main stages are:", h2_blue))
    stages = [
        "1. Compile the bare-metal Rust kernel with <code>x86_64-unknown-none</code> target in release mode.",
        "2. Link the 64-bit ELF executable using the linker script at high-canonical addresses.",
        "3. Create the ISO staging filesystem directory at <code>build/iso_root/</code>.",
        "4. Copy the compiled kernel binary and Limine configuration into the boot directory.",
        "5. Stage Limine BIOS (<code>limine-bios-cd.bin</code>) and UEFI (<code>limine-uefi-cd.bin</code>) boot assets.",
        "6. Use <code>xorriso</code> to assemble a hybrid BIOS + UEFI ISO and calculate a SHA-256 checksum.",
        "7. Deploy Limine MBR boot records using <code>limine bios-install</code> to enable legacy boot."
    ]
    for s in stages:
        story.append(Paragraph(s, bullet_style))

    story.append(Paragraph("6. Prepare the Build Workspace", h1_style))
    story.append(Paragraph(
        "Before running the script, confirm that the expected source folders exist and contain the required files:",
        body_style
    ))
    story.append(make_code_box(
        "ls -ld src/ src/scheduler src/task src/memory src/timer<br/>"
        "ls -l limine.conf linker.ld Cargo.toml<br/>"
        "ls -l limine/limine-bios.sys limine/limine-uefi-cd.bin"
    ))
    story.append(Paragraph(
        "Also confirm that the output directory <code>build/</code> exists and is writable. The script's output path "
        "generates the bootable image directly in the local workspace.",
        body_style
    ))

    story.append(Paragraph("7. Run the ISO Build", h1_style))
    story.append(Paragraph(
        "Only after confirming every input path, reviewing the compiler configuration, and preparing the toolchain, "
        "compile and package the operating system:",
        body_style
    ))
    story.append(make_code_box(
        "cargo +nightly build --release --target x86_64-unknown-none -Zbuild-std=core,alloc<br/>"
        "chmod +x scripts/build_iso.sh<br/>"
        "./scripts/build_iso.sh"
    ))
    story.append(make_callout(
        "The script compiles the bare-metal kernel, creates staging directories, and calls <code>xorriso</code>. "
        "Its build stage ensures Limine boot headers and El Torito records are properly mapped. Do not terminate the "
        "process while xorriso creates the hybrid partition table."
    ))
    story.append(Paragraph(
        "If the process completes successfully, the script prints the output ISO path and appends its SHA-256 checksum "
        "to the checksum record. The output location is <code>build/yashwanth_micro_os.iso</code>.",
        body_style
    ))
    story.append(PageBreak())

    # =========================================================================
    # PAGE 4: VERIFICATION, VM TESTING & REPO
    # =========================================================================
    story.append(Paragraph("8. Verify the ISO", h1_style))
    story.append(Paragraph("Check that the ISO exists and has a sensible file size:", body_style))
    story.append(make_code_box(
        "ls -lh build/yashwanth_micro_os.iso<br/>"
        "file build/yashwanth_micro_os.iso<br/>"
        "sha256sum build/yashwanth_micro_os.iso"
    ))
    story.append(Paragraph(
        "Compare the checksum with the generated checksum record. A checksum helps detect accidental file corruption; "
        "it does not by itself prove that the ISO boots correctly.",
        body_style
    ))

    story.append(Paragraph("9. Test the ISO in a Virtual Machine", h1_style))
    story.append(Paragraph("Use a virtual machine such as VirtualBox or QEMU to test the image before trying it on physical hardware:", body_style))
    vm_steps = [
        "1. Create a new Linux virtual machine with an x86_64 architecture (or run with QEMU).",
        "2. Attach the generated ISO (<code>yashwanth_micro_os.iso</code>) as the virtual optical disk.",
        "3. Start the VM and check whether the Limine boot menu appears.",
        "4. Test the normal live boot option and observe kernel startup logs on the linear framebuffer.",
        "5. If available, repeat with UEFI enabled and with legacy BIOS mode.",
        "6. Check task execution, round-robin time slicing, and dynamic quantum adjustment in serial logs.",
        "7. Record any boot errors and fix them before sharing the ISO."
    ]
    for step in vm_steps:
        story.append(Paragraph(step, bullet_style))

    story.append(Paragraph("10. Use the Published ISO from Hugging Face", h1_style))
    story.append(Paragraph(
        "If your goal is only to test the already-built image, you do not need to rebuild it. Open the Hugging Face repository, "
        "select <b>yashwanth_micro_os.iso</b>, and download it. Then attach that ISO to a virtual machine for testing.",
        body_style
    ))
    story.append(Paragraph('<a href="https://huggingface.co/Yashbot07/yashwanth-microos-iso/tree/main"><font color="#2B6CB0"><u>Open the Hugging Face ISO repository</u></font></a>', link_style))
    story.append(Paragraph('<a href="https://huggingface.co/spaces/Yashbot07/yashwanth-micro-os-scheduler"><font color="#2B6CB0"><u>Open the Hugging Face Interactive Scheduler Simulator</u></font></a>', link_style))

    story.append(Paragraph("11. Create a Bootable USB (Optional)", h1_style))
    story.append(Paragraph(
        "The project README recommends using Rufus on Windows and selecting “Write in DD Image mode” if Rufus identifies "
        "the ISO as an ISOHybrid image. balenaEtcher is another option. Writing an ISO erases the selected USB drive, so "
        "carefully verify the target device and get permission before proceeding.",
        body_style
    ))

    story.append(Paragraph("12. Troubleshooting", h1_style))
    troubles = [
        "• <b>Missing nightly toolchain:</b> install nightly Rust with <code>rustup default nightly</code> and add <code>rust-src</code>.",
        "• <b>Missing boot files:</b> confirm the <code>limine/</code> directory contains the required BIOS and UEFI CD binary assets.",
        "• <b>Command not found:</b> install the required package (for example, <code>xorriso</code> or <code>mtools</code>).",
        "• <b>Output path error:</b> verify the <code>build/</code> directory exists and is writable in your workspace.",
        "• <b>ISO builds but does not boot:</b> verify Limine bootloader configuration in <code>limine.conf</code> and test BIOS/UEFI separately."
    ]
    for t in troubles:
        story.append(Paragraph(t, bullet_style))
    story.append(PageBreak())

    # =========================================================================
    # PAGE 5: SUMMARY & LINKS
    # =========================================================================
    story.append(Paragraph("13. Summary", h1_style))
    story.append(Paragraph(
        "The Yashwanth Micro-OS build process prepares a bare-metal x86_64 Rust kernel, configures hardware interrupts "
        "with Intel 8254 PIT and dual 8259 PIC, executes a Dynamic Quantum Round-Robin scheduler, and packages the result "
        "as a bootable ISO using Limine and xorriso. The final image should then be checked and tested in a virtual machine.",
        body_style
    ))
    story.append(Paragraph("Project owner: <b>YASHWANTH K</b>", body_style))

    story.append(Spacer(1, 6))
    story.append(Paragraph("Reference files checked", h2_blue))
    ref_files = [
        "• GitHub repository README.md",
        "• Kernel source <code>src/main.rs</code> and <code>src/scheduler/round_robin.rs</code>",
        "• Hugging Face repository listing for <code>yashwanth_micro_os.iso</code>",
        "• Hugging Face Space interactive scheduler simulator (<code>app.py</code>)",
        "• Academic reports: CA-2, CA-3, CA-5, and comprehensive OS Lab Report"
    ]
    for rf in ref_files:
        story.append(Paragraph(rf, bullet_style))

    story.append(Spacer(1, 6))
    story.append(Paragraph("Links", h2_blue))
    story.append(Paragraph('<a href="https://github.com/yash28062007-max/yashwanth-micro-os-scheduler"><font color="#2B6CB0"><u>GitHub source repository</u></font></a>', link_style))
    story.append(Paragraph('<a href="https://github.com/yash28062007-max/yashwanth-micro-os-scheduler/tree/main/reports"><font color="#2B6CB0"><u>GitHub academic reports folder</u></font></a>', link_style))
    story.append(Paragraph('<a href="https://huggingface.co/Yashbot07/yashwanth-microos-iso/tree/main"><font color="#2B6CB0"><u>Hugging Face ISO repository</u></font></a>', link_style))
    story.append(Paragraph('<a href="https://huggingface.co/spaces/Yashbot07/yashwanth-micro-os-scheduler"><font color="#2B6CB0"><u>Hugging Face Interactive Simulator Space</u></font></a>', link_style))

    story.append(Spacer(1, 40))
    story.append(Paragraph(
        "Note: This guide is based on the repository contents visible at the time it was prepared. "
        "Build scripts and file paths may change when the repository is updated.",
        ParagraphStyle('NoteEnd', parent=body_style, fontSize=7.5, leading=11, textColor=colors.HexColor("#718096"))
    ))

    # Build the PDF
    doc.build(story, canvasmaker=ExactNumberedCanvas)
    print(f"Generated PDF successfully at {output_path}")

if __name__ == "__main__":
    desktop_pdf = r"C:\Users\yash2\OneDrive\Desktop\Yashwanth_Micro_OS_Project_Guide.pdf"
    generate_pdf(desktop_pdf)
    reader = PdfReader(desktop_pdf)
    print(f"Verified Page Count: {len(reader.pages)}")
    assert len(reader.pages) == 5, f"Expected 5 pages, got {len(reader.pages)}"
