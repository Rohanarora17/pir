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
    section("1", "Progress Since the First Report"),
    para("Since Report 1, the <link href='https://github.com/Rohanarora17/pir' color='#000000'><u>project</u></link> has progressed from workload and provider checks to a retained proof corpus, native proof verification, candidate binary records and local CPU PIR retrieval. Helios has been selected as the consensus light client. Its root interface has been tested using synthetic consensus fixtures."),
    callout("Current milestone", "The real proof records can be retrieved through PIR and verified locally under the supplied execution state root. Live consensus authentication of that root and a complete network service remain pending."),
    Spacer(1, 7),
    section("2", "Real Corpus and Native Proof Verification"),
    para("A corpus of 32 declared addresses was collected at Sepolia execution block <b>11919648</b> for the same WETH9 contract used in Report 1. Every proof request used the exact block hash through EIP-1898. A second provider reproduced the normalised corpus. A separate historical retention failure was preserved without changing the selected block or falling back to another selector."),
    para("The Rust libraries <font name='Courier'>alloy-trie</font> and <font name='Courier'>eth_trie</font> were compared on the retained proofs. Both verified all records under the supplied root and passed 313 rejection checks per candidate. <b>eth_trie 0.6.1</b> was selected because it recovers authenticated account and storage values, fitting the proof based manifest without relying on provider supplied account field hints."),
    data_table([
        ["Item", "Verified local result"],
        ["Corpus", "32 records at one fixed block hash, with the shared WETH account proof"],
        ["Balance outcomes", "24 nonzero inclusions and 8 proven absences under the supplied root"],
        ["Reusable verifier", "Separate account and storage stages, independent key and snapshot binding, byte limits and typed failures"],
        ["Workspace validation", "27 passing Rust tests, including malformed proofs, false values and missing path rejection"],
    ], [43 * mm, 125 * mm]),
    Spacer(1, 5),
    para("The verifier recovers the account storage root before checking the requested storage key. Missing paths, malformed nodes and false values fail explicitly. A zero balance requires a valid absence proof. These results remain conditional on the supplied root, whose consensus provenance is unverified."),
    subsection("Integration decision"),
    para("The verifier consumes project owned types and performs no RPC or consensus synchronisation. PIR transports the record bytes, while a separate light client will supply the expected root. This keeps proof correctness, selection privacy and root authentication independently testable. The selected library's internal types do not enter the protocol crate."),
    section("3", "Helios Selection and Root Interface"),
    para("<b>Helios has been selected</b> for independent execution root authentication. A separate probe verifies a trusted bootstrap and five signed updates from retained synthetic Deneb fixtures. It exports the accepted finalised execution block number, block hash and state root. Ten Rust boundary tests and four source integrity tests cover corrupt updates, incorrect network or checkpoint, stale contexts and provider root substitution."),
    para("The probe distinguishes optimistic progress from finality. In one transition, the optimistic head reaches slot 96 while the finalised head remains at slot 72. The adapter continues to export the execution context from slot 72 until a verified finality update advances it. A timeout never forces an optimistic head into finality."),
    para("These fixtures use a 32 member test committee and do not authenticate the real WETH corpus. Live Sepolia sync, checkpoint provenance, current fork support and restart recovery remain pending. The choice between core only embedding and reuse of fuller Helios synchronisation machinery is still open."),
    PageBreak(),
    section("4", "Proof Record Layout and Storage Measurements"),
    para("A bounded experimental binary codec was built to measure complete storage proof records without JSON hex expansion. Every real row round trips through encoding, padding, decoding and native verification. Oversized rows and malformed frames are rejected explicitly."),
    data_table([
        ["Representation", "Measured result on the 32 records"],
        ["Compact JSON", "4,599 to 5,545 bytes per record"],
        ["Binary before padding", "2,254 to 2,731 bytes per record"],
        ["3,072 byte rows", "All records fit, with 18.05% of database bytes used as padding"],
        ["4,096 byte rows", "All records fit, with 38.54% padding"],
    ], [43 * mm, 125 * mm]),
    Spacer(1, 5),
    para("Three KiB is the smallest tested capacity that fits this corpus, not a general Ethereum proof bound. At 2,560 bytes, ten records are rejected as oversized. Final geometry must account for larger corpora, future proof sizes and backend alignment rather than discarding proof nodes to meet a row limit."),
    para("The common account proof is stored once in public metadata. Storage proofs contain 192 node references but only 142 distinct nodes. Deduplication could remove 35.84% of raw storage proof bytes before private indexing and padding costs. This is a storage observation, not a demonstrated retrieval improvement. Privately obtaining several nodes may cost more than retrieving one complete proof bundle."),
    section("5", "Local Private Retrieval Experiments"),
    para("The pinned IKPIR implementation was integrated through an isolated adapter. Direct index PIR and full keyword lookup were tested with SimplePIR and FrodoPIR, at both 3,072 and 4,096 byte record sizes. All eight configurations recovered and verified all 32 records, giving <b>256 successful local PIR reads</b>. Source archives, licences, parameters and raw results are retained."),
    data_table([
        ["Engine", "Retrieval mode", "Setup minimum (bytes)", "Query + reply (bytes)"],
        ["SimplePIR", "Direct index", "15,667,284", "12,504"],
        ["FrodoPIR", "Direct index", "19,243,072", "12,504"],
        ["SimplePIR", "Keyword", "125,664,173", "98,720"],
        ["FrodoPIR", "Keyword", "154,345,093", "98,720"],
    ], [29 * mm, 36 * mm, 55 * mm, 48 * mm]),
    Spacer(1, 4),
    para("The table uses 3,072 byte rows. Query and reply figures include the tested message framing. Setup figures are canonical minimum estimates, not measured network transfers, and exclude the common manifest and index map. The exchange runs within a local process. It does not establish network latency, concurrent throughput or a cryptographic security audit."),
    subsection("Interpretation of the results"),
    para("Direct indexing has lower setup and response costs in this geometry and is the proposed next baseline. It resolves the logical index locally from a public map. Full keyword lookup returns buckets across multiple segments, increasing the amount retrieved per balance. This comparison concerns the tested table layout and does not establish a universally better PIR engine."),
    para("The complete padded generation plus metadata and index map is only 109,390 bytes. The current corpus therefore establishes functional integration, not PIR's practical advantage. Whole generation download remains a reference measurement, but the tiny corpus comparison does not settle the architecture or predict the outcome at meaningful scales."),
    callout("Evidence boundary", "Native proof recovery and PIR retrieval have been composed locally under one supplied root. Live consensus authentication, network transport, private trie node traversal and realistic scale evaluation have not yet been demonstrated."),
    PageBreak(),
    section("6", "Research Contribution and Evaluation Question"),
    para("The intended contribution is an evidence based account of <b>when private, verifiable Ethereum reads are practical</b>. A working PIR query establishes compatibility, but the stronger result is an operating envelope that connects corpus size, proof representation, client resources, read frequency and snapshot freshness to measured costs."),
    para("The main question is: for which workloads does selective private retrieval provide a useful tradeoff compared with downloading and caching the declared dataset? The study must also identify when setup, proof duplication, server work or generation refresh dominates. A larger database is a test condition, not evidence by itself that PIR will be preferable."),
    section("7", "Planned Scale and Workload Study"),
    para("The 32 real records will remain the regression corpus. Separate scale tests will increase logical record count and database bytes across multiple orders of magnitude. Concrete sizes will be selected after estimating memory costs, with security parameters held fixed. Synthetic workloads will be labelled separately from larger real Ethereum corpora."),
    data_table([
        ["Dimension", "Planned comparison and evidence"],
        ["Representation", "Complete proof bundles versus deduplicated trie nodes, including padding, index maps and the number of private retrieval rounds"],
        ["Client lifecycle", "Cold setup, repeated reads on one generation and transitions to a new generation, including hints, public metadata and client storage"],
        ["System resources", "Server preprocessing, query computation, memory and latency distributions, with resource limits and failed sizes reported"],
        ["Reference paths", "Whole generation download counted once when cached, plus ordinary revealing RPC as a correctness and cost reference"],
    ], [40 * mm, 128 * mm]),
    Spacer(1, 5),
    para("Repeated copies of the current records can isolate backend scaling, but cannot establish realistic trie node sharing, proof diversity or update behaviour. Those properties require a declared real corpus. Network measurements will include transport, or will be clearly labelled as a model. Local computation timings will not be presented as complete request latency."),
    para("The comparisons will use matched logical queries, proof requirements and snapshot identities. Any private node traversal must hide selection through its access pattern, including retrieval count and stopping behaviour. A smaller database alone is insufficient if it requires more private rounds or reveals part of the requested path."),
    section("8", "Pending Integration and Next Milestone"),
    bullet("Run the scale and representation study, then review the final record layout and PIR baseline."),
    bullet("Validate live Helios synchronisation and recovery, then bind a new corpus to an accepted root or establish an authenticated history path for an older block."),
    bullet("Implement authenticated generation commitments, atomic publication and network serving before demonstrating a complete private and consensus verified read."),
    Spacer(1, 4),
    para("The project remains in Phase 1. Verifier and Helios selection are accepted. Final PIR adoption, record geometry, the Helios integration mode and repository licence remain open. No practical crossover or production privacy claim is asserted before the planned measurements and integration checks."),
    subsection("Literature and implementation references"),
    literature_item(1, "eth_trie 0.6.1", "Value recovery and proof verification interfaces used in the Rust adapter.", "https://docs.rs/eth_trie/0.6.1/eth_trie/"),
    literature_item(2, "IKPIR", "Pinned CPU implementation used for direct and keyword retrieval experiments.", "https://github.com/orochi-network/IKPIR"),
    literature_item(3, "Helios", "Pinned consensus core used for signed update and finalised root interface tests.", "https://github.com/a16z/helios"),
    literature_item(4, "SimplePIR and DoublePIR", "Client preprocessing and communication tradeoffs, USENIX Security 2023.", "https://www.usenix.org/conference/usenixsecurity23/presentation/henzinger"),
    literature_item(5, "EIP-1186", "Native account and storage proof fields used by the collector and verifier.", "https://eips.ethereum.org/EIPS/eip-1186"),
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
