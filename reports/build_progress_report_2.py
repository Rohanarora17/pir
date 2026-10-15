#!/usr/bin/env python3
"""Rebuild the final report with ReportLab, independently of the working directory."""
import argparse
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_LEFT
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import mm
from reportlab.platypus import PageBreak, Paragraph, SimpleDocTemplate, Spacer, Table, TableStyle


INK = colors.black
RULE = colors.HexColor("#777777")
GRID = colors.HexColor("#999999")
LIGHT = colors.HexColor("#E8E8E8")
PALE = colors.HexColor("#F4F4F4")
WHITE = colors.white


styles = getSampleStyleSheet()
styles.add(ParagraphStyle(
    name="ReportTitle", parent=styles["Title"], fontName="Times-Bold",
    fontSize=16.5, leading=19.5, textColor=INK, alignment=TA_CENTER,
    spaceAfter=7,
))
styles.add(ParagraphStyle(
    name="ReportSubtitle", parent=styles["Normal"], fontName="Times-Roman",
    fontSize=10.5, leading=13.5, textColor=INK, alignment=TA_CENTER,
    spaceAfter=8,
))
styles.add(ParagraphStyle(
    name="Section", parent=styles["Heading1"], fontName="Times-Bold",
    fontSize=12.6, leading=15.2, textColor=INK, spaceBefore=4,
    spaceAfter=5, keepWithNext=True,
))
styles.add(ParagraphStyle(
    name="Subsection", parent=styles["Heading2"], fontName="Times-Bold",
    fontSize=10.7, leading=13.2, textColor=INK, spaceBefore=6,
    spaceAfter=2.5, keepWithNext=True,
))
styles.add(ParagraphStyle(
    name="Body", parent=styles["BodyText"], fontName="Times-Roman",
    fontSize=9.8, leading=12.6, textColor=INK, spaceAfter=5,
))
styles.add(ParagraphStyle(
    name="Small", parent=styles["BodyText"], fontName="Times-Roman",
    fontSize=9, leading=11.2, textColor=INK, alignment=TA_CENTER,
    spaceAfter=3,
))
styles.add(ParagraphStyle(
    name="ReportBullet", parent=styles["BodyText"], fontName="Times-Roman",
    fontSize=9.5, leading=12, leftIndent=13, firstLineIndent=-8,
    bulletIndent=2, textColor=INK, spaceAfter=2.5,
))
styles.add(ParagraphStyle(
    name="Block", parent=styles["BodyText"], fontName="Times-Roman",
    fontSize=9.4, leading=12, textColor=INK,
))
styles.add(ParagraphStyle(
    name="TableHead", parent=styles["BodyText"], fontName="Times-Bold",
    fontSize=8.5, leading=10.5, textColor=INK, alignment=TA_LEFT,
))
styles.add(ParagraphStyle(
    name="TableCell", parent=styles["BodyText"], fontName="Times-Roman",
    fontSize=8.45, leading=10.5, textColor=INK,
))
styles.add(ParagraphStyle(
    name="Lit", parent=styles["BodyText"], fontName="Times-Roman",
    fontSize=8.8, leading=11.1, leftIndent=16, firstLineIndent=-14,
    textColor=INK, spaceAfter=3,
))


def para(text, style="Body"):
    return Paragraph(text, styles[style])


def section(number, title):
    return Paragraph(f"{number}. {title}", styles["Section"])


def subsection(title):
    return Paragraph(title, styles["Subsection"])


def bullet(text):
    return Paragraph(f"<bullet>&bull;</bullet>{text}", styles["ReportBullet"])


def callout(title, body):
    table = Table(
        [[Paragraph(f"<b>{title}.</b> {body}", styles["Block"])]],
        colWidths=[168 * mm], hAlign="LEFT",
    )
    table.setStyle(TableStyle([
        ("LINEBEFORE", (0, 0), (0, -1), 1, INK),
        ("LEFTPADDING", (0, 0), (-1, -1), 8),
        ("RIGHTPADDING", (0, 0), (-1, -1), 3),
        ("TOPPADDING", (0, 0), (-1, -1), 3.5),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 3.5),
    ]))
    return table


def data_table(rows, widths):
    converted = []
    for row_number, row in enumerate(rows):
        style = styles["TableHead"] if row_number == 0 else styles["TableCell"]
        converted.append([Paragraph(str(cell), style) for cell in row])
    table = Table(converted, colWidths=widths, repeatRows=1, hAlign="LEFT")
    table.setStyle(TableStyle([
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("GRID", (0, 0), (-1, -1), 0.35, GRID),
        ("BACKGROUND", (0, 0), (-1, 0), LIGHT),
        ("LEFTPADDING", (0, 0), (-1, -1), 6),
        ("RIGHTPADDING", (0, 0), (-1, -1), 6),
        ("TOPPADDING", (0, 0), (-1, -1), 4),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 4),
    ]))
    return table










def literature_item(number, title, body, url):
    return Paragraph(
        f"<b>{number}. {title}.</b> {body} "
        f"<link href='{url}' color='#000000'><u>Source</u></link>",
        styles["Lit"],
    )


def footer(canvas, document):
    canvas.saveState()
    width, _ = A4
    canvas.setFont("Times-Roman", 8)
    canvas.setFillColor(INK)
    canvas.drawString(21 * mm, 10 * mm, "Rohan Arora | CS26S022")
    canvas.drawRightString(width - 21 * mm, 10 * mm, str(document.page))
    canvas.restoreState()





parser = argparse.ArgumentParser(description="Build the three-page CS6666 Progress Report 2 PDF.")
parser.add_argument("--output", type=Path, default=Path(__file__).resolve().with_name("progress-report-2.pdf"))
args = parser.parse_args()
OUTPUT = args.output.expanduser().resolve()

story = [
    para("Progress Report 2", "ReportTitle"),
    para("Private and Verifiable Ethereum State Reads", "ReportSubtitle"),
    para("CS6666 | Blockchain and Distributed Systems", "Small"),
    para("<b>Rohan Arora</b> &nbsp;&nbsp;|&nbsp;&nbsp; Roll No. <b>CS26S022</b>", "Small"),
    Spacer(1, 7),
    section("1", "Progress Since Report 1"),
    para("The project studies private and verifiable access to Ethereum contract state, using WETH balances at a fixed Sepolia snapshot. Report 1 covered the architecture, Rust foundation, workload selection and RPC provider study. The work since then has focused on real proof collection, native proof verification, record encoding and local PIR experiments."),
    para("<b>Work completed so far.</b> A dataset of 32 real balance proofs has been collected and verified. These records have also been retrieved through existing PIR implementations and checked again by the verifier. Helios has been selected for the light client component, and an initial connection has been tested using synthetic data. Code, retained inputs and experiment results are available in the <link href='https://github.com/Rohanarora17/pir' color='#000000'><u>repository</u></link>."),
    section("2", "Real Proof Collection and Verification"),
    para("The new dataset contains 32 addresses at Sepolia block <b>11919648</b>, using the same WETH9 contract and balance mapping as Report 1. Every proof request specifies the exact block hash. A second provider reproduced the dataset. A later failure to obtain the historical state was retained as a collection failure, without switching to another block."),
    subsection("Verifier comparison and selection"),
    para("Two Rust libraries, <font name='Courier'>alloy-trie</font> and <font name='Courier'>eth_trie</font>, were compared on the real proofs and rejection cases. Both checked all 32 records and passed 313 rejection checks per candidate. <b>eth_trie 0.6.1</b> was selected because it can recover the checked account data directly from the proof. This fits the current record format and avoids relying on account fields supplied separately by the RPC provider."),
    data_table([
        ["Part", "Work completed"],
        ["Proof dataset", "32 records at one fixed block hash, reproduced through a second provider"],
        ["Balance verification", "24 nonzero balances and 8 proven storage absences under the supplied root"],
        ["Reusable Rust verifier", "Account and storage proof checks, requested key and snapshot checks, size limits and explicit errors"],
        ["Workspace validation", "27 passing Rust tests covering valid reads and rejection behaviour"],
    ], [43 * mm, 125 * mm]),
    Spacer(1, 5),
    subsection("Checks enforced by the verifier"),
    para("The account proof is checked against the expected state root, and the recovered WETH storage root is used to check the requested balance. The returned value must match the value established by the proof. An absent storage entry is accepted as zero only with a valid absence proof. Missing nodes, malformed data and a wrong requested key produce errors."),
    para("The verifier is a separate component from PIR and the light client. This keeps proof checking independent of the retrieval method and leaves root authentication to the consensus component. The present real-data results remain conditional on the provider-supplied root."),
    section("3", "Decisions Carried Forward"),
    bullet("Continue exact block-hash collection and reject unavailable historical state without a fallback."),
    bullet("Use eth_trie behind the project verifier interface, with request checks and bounded proof inputs."),
    bullet("Verify every record recovered through PIR before accepting its balance."),
    bullet("Keep live consensus authentication separate from provider agreement and local proof verification."),
    PageBreak(),
    section("4", "Proof Record Encoding"),
    para("The original proof responses have different sizes, while the tested PIR arrangement uses fixed size records. A binary encoder and decoder were added to compare candidate record sizes. The common WETH account proof is stored once in the snapshot information, with each record carrying its balance and storage proof. All 32 records were encoded, decoded and verified again."),
    data_table([
        ["Representation", "Result on the current dataset"],
        ["Compact JSON", "4,599 to 5,545 bytes per record"],
        ["Binary before padding", "2,254 to 2,731 bytes per record"],
        ["3,072 byte records", "All records fit, with 18.05% of database bytes used for padding"],
        ["4,096 byte records", "All records fit, with 38.54% padding"],
    ], [43 * mm, 125 * mm]),
    Spacer(1, 5),
    para("The 3,072 byte format is the smallest tested size that holds this dataset. It is not a general limit for Ethereum proofs. Oversized records are rejected rather than shortened. Repeated parts of the storage proofs could also be stored once, with a possible 35.84% saving in raw proof bytes here. That saving still needs to be compared with indexing, padding and any extra private requests."),
    section("5", "Local PIR Experiments"),
    para("The experiments use a fixed version of IKPIR and its SimplePIR and FrodoPIR implementations. Two retrieval methods were compared: requesting a record by its position and looking it up by a key. In the first method, the client finds the position locally from a public list and uses PIR to hide which position it requests."),
    para("Both methods and implementations were tested with 3,072 and 4,096 byte records. All eight configurations recovered and verified the full dataset, giving <b>256 successful local PIR reads</b>. These are repeated checks of the same 32 records, not a larger dataset."),
    data_table([
        ["Observation", "Result and implication"],
        ["Private retrieval", "All 32 records recovered correctly in each of eight configurations"],
        ["Lookup by position", "Lower setup and reply sizes than key lookup in this experiment"],
        ["Per-read communication", "12,504 bytes by position versus 98,720 bytes by key for 3,072 byte records"],
        ["Initial setup", "Significant additional client information is required before reads, and must be counted separately"],
    ], [43 * mm, 125 * mm]),
    Spacer(1, 5),
    subsection("What the results establish"),
    para("The local runs show that private retrieval, record decoding and native proof verification work together. They do not constitute a network service or a privacy audit. Lookup by position is the next baseline to investigate, while the final PIR implementation remains open."),
    para("Setup cost is important in interpreting the communication figures. For 3,072 byte records, the smaller SimplePIR setup estimate is about 15.7 MB, compared with 109,390 bytes for the complete dataset and shared information. Setup figures are minimum estimates, not measured network downloads. The experiment therefore does not show a bandwidth advantage for PIR on this small dataset."),
    para("A failed private read must return an error. It must not trigger an ordinary RPC request for the same address, since that would reveal the selection. Network delay, concurrent clients and the complete snapshot publication path have not yet been measured or implemented."),
    PageBreak(),
    section("6", "Light Client Progress"),
    para("Helios has been selected as the Ethereum light client. Its role is to supply an independently checked block identity and state root to the proof verifier. An initial Rust connection has been tested using saved synthetic consensus data, including checks that reject altered updates and an incorrect expected root."),
    para("The tests also keep a recently observed block separate from a finalised block. The code does not accept a newer root as finalised merely because time has passed. These are local feasibility tests. They do <b>not</b> authenticate the real Sepolia balance dataset or establish live synchronisation."),
    para("The remaining work includes a trusted starting checkpoint, live Sepolia synchronisation, support for the required network version and recovery after restarting. Helios is selected, but the precise integration method is still open."),
    section("7", "Evaluation Focus and Next Work"),
    para("The next question is <b>when private, verifiable reads become practical</b>. The 32-record dataset is useful for correctness checks, but is too small to judge the practical value of PIR. Increasing the dataset alone is also not enough to claim an advantage. The comparison must account for read frequency, client storage and snapshot updates."),
    para("Downloading and caching a complete snapshot once also hides later selections. It should therefore be charged once per snapshot, rather than again for every read. The planned evaluation will compare that option with PIR, including initial setup, repeated reads and the cost of moving to a new snapshot."),
    data_table([
        ["Next task", "Purpose"],
        ["Larger scale experiments", "Measure setup, query cost and memory at increasing dataset sizes without weakening cryptographic settings"],
        ["Proof storage comparison", "Compare complete proof records with shared proof parts, including all retrieval costs"],
        ["Real and artificial datasets", "Separate backend scaling results from claims about real Ethereum proof sizes and updates"],
        ["Live composed read", "Collect fresh proofs for a block checked by Helios and test retrieval, proof verification and root authentication together"],
    ], [43 * mm, 125 * mm]),
    Spacer(1, 5),
    para("<b>Current position.</b> The project remains in Phase 1. The verifier and Helios choices are settled. The final PIR implementation, record size, Helios integration method and repository licence remain open. Larger-scale evaluation, network serving and checks that all published files belong to the same snapshot are pending."),
    subsection("Literature and Implementations Reviewed"),
    literature_item(1, "eth_trie 0.6.1", "Used for the account and storage proof verifier.", "https://docs.rs/eth_trie/0.6.1/eth_trie/"),
    literature_item(2, "IKPIR", "Used for the position and key lookup experiments with SimplePIR and FrodoPIR.", "https://github.com/orochi-network/IKPIR"),
    literature_item(3, "Helios", "Selected for the consensus component and tested with synthetic inputs.", "https://github.com/a16z/helios"),
    literature_item(4, "SimplePIR and DoublePIR", "Background for setup and communication cost comparisons, USENIX Security 2023.", "https://www.usenix.org/conference/usenixsecurity23/presentation/henzinger"),
    literature_item(5, "EIP-1186", "Reference for account and storage proof response fields.", "https://eips.ethereum.org/EIPS/eip-1186"),
]

OUTPUT.parent.mkdir(parents=True, exist_ok=True)
document = SimpleDocTemplate(
    str(OUTPUT), pagesize=A4,
    leftMargin=21 * mm, rightMargin=21 * mm,
    topMargin=18 * mm, bottomMargin=20 * mm,
    title="Private and Verifiable Ethereum State Reads | Progress Report 2",
    author="Rohan Arora (CS26S022)",
    subject="CS6666 Blockchain and Distributed Systems Project Progress Report 2",
)
document.build(story, onFirstPage=footer, onLaterPages=footer)
print(OUTPUT.resolve())
