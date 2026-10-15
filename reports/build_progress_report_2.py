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
    section("1", "Work Since Report 1"),
    para("Report 1 covered the WETH workload, the initial Rust code and the checks on RPC providers. Since then, the work has moved to collecting a set of real proofs, checking them locally and retrieving them through PIR. The <link href='https://github.com/Rohanarora17/pir' color='#000000'><u>repository</u></link> now contains the code, retained inputs and experiment results. Helios has also been selected for the light client part."),
    para("The main progress is that a proof record can now pass through the local PIR code and then through the Ethereum proof verifier. The state root used in this test still comes from the provider. Connecting it to live Ethereum consensus is work that remains."),
    section("2", "Collecting and Checking Real Proofs"),
    para("The new corpus contains 32 addresses at Sepolia block <b>11919648</b>, using the same WETH9 contract as before. Each request uses the exact block hash through EIP-1898. The corpus was reproduced using a second provider. One later request failed because a provider no longer had the old state. That failure was retained, and the collector did not silently move to another block."),
    para("Two Rust libraries, <font name='Courier'>alloy-trie</font> and <font name='Courier'>eth_trie</font>, were tested on these proofs. Both checked all 32 records and passed 313 rejection checks per candidate. The current verifier uses <b>eth_trie 0.6.1</b>. It can recover the account data from a proof, which lets the code obtain the storage root without trusting the account fields supplied separately by the RPC provider."),
    data_table([
        ["Check", "Result"],
        ["Real proof corpus", "32 records collected at one fixed block hash"],
        ["Balance results", "24 nonzero inclusions and 8 proven absences under the supplied root"],
        ["Verifier checks", "Account and storage verification, expected key and snapshot checks, proof size limits and explicit errors"],
        ["Rust workspace tests", "27 passing tests, including incorrect values, malformed nodes and missing proof paths"],
    ], [43 * mm, 125 * mm]),
    Spacer(1, 5),
    para("The checking order is important. First, the account proof gives the WETH storage root. The storage proof is then checked against that root for the requested key. If a required node is missing, the result is an error. It must not be treated as a zero balance. The eight zero cases above have valid absence proofs under the supplied root."),
    para("The verifier is kept separate from PIR and the light client. It only receives the expected request details and proof data. This means the retrieval method can change without rewriting Ethereum proof verification. It also makes it easier to test which check rejects a bad response."),
    section("3", "Choosing Helios for the Light Client"),
    para("Helios has been selected to check where the execution state root comes from. So far, the test uses its consensus code with a retained synthetic bootstrap and five signed Deneb updates. The adapter returns the finalised execution block number, block hash and state root. Ten Rust tests and four source integrity tests check this part of the experiment."),
    para("One useful case separates the optimistic head from the finalised head. The optimistic head moves to slot 96, while the finalised head stays at slot 72. The adapter continues using slot 72 until a verified update advances finality. Waiting longer does not cause it to accept the optimistic head as finalised."),
    para("This is still a fixture test with a 32 member test committee. It does not authenticate the WETH corpus on Sepolia. The remaining work includes live sync, a trusted recent checkpoint, support for the required fork and restart recovery. How much of Helios's existing sync code to reuse is also still open."),
    PageBreak(),
    section("4", "How the Proof Records Are Stored"),
    para("The PIR tests use fixed size records, while Ethereum proofs have different sizes. A binary encoder and decoder were added to try out padded proof records. All 32 records were encoded, decoded and verified again. The decoder also rejects oversized records and malformed frames."),
    data_table([
        ["Format", "Result on the current corpus"],
        ["Compact JSON", "4,599 to 5,545 bytes per record"],
        ["Binary before padding", "2,254 to 2,731 bytes per record"],
        ["3,072 byte records", "All 32 fit, with 18.05% of database bytes used for padding"],
        ["4,096 byte records", "All 32 fit, with 38.54% padding"],
    ], [43 * mm, 125 * mm]),
    Spacer(1, 5),
    para("A 3 KiB record is the smallest size tested that holds this whole corpus. It is not a general limit for Ethereum proofs. At 2,560 bytes, ten records do not fit. A larger corpus or another snapshot may therefore need a different size. The code rejects such a record rather than removing proof nodes to make it fit."),
    para("There is also repeated data in the proofs. The WETH account proof is common to all requests, so it is stored once in public metadata. The storage proofs have 192 node references but only 142 distinct nodes. Removing duplicates could save 35.84% of the raw storage proof bytes, before indexing and padding. Whether this helps PIR is still a separate question, since retrieving several nodes may require more private queries than retrieving one complete proof."),
    section("5", "PIR Experiments"),
    para("The local PIR tests use a fixed version of IKPIR. They compare direct index retrieval with full keyword lookup, using both SimplePIR and FrodoPIR. Each combination was tested with 3,072 and 4,096 byte records. All eight configurations recovered and verified the 32 records, giving <b>256 successful local PIR reads</b>."),
    data_table([
        ["Engine", "Retrieval mode", "Setup minimum (bytes)", "Query + reply (bytes)"],
        ["SimplePIR", "Direct index", "15,667,284", "12,504"],
        ["FrodoPIR", "Direct index", "19,243,072", "12,504"],
        ["SimplePIR", "Keyword", "125,664,173", "98,720"],
        ["FrodoPIR", "Keyword", "154,345,093", "98,720"],
    ], [29 * mm, 36 * mm, 55 * mm, 48 * mm]),
    Spacer(1, 4),
    para("These figures use 3,072 byte records. Query and reply sizes include the message framing. The setup sizes are minimum estimates from the setup data, not actual network downloads. They also exclude the common manifest and index map. The client needs setup information, including a hint used to decode replies, so counting only query and reply bytes would leave out a significant cost."),
    para("Direct indexing costs less than keyword lookup in this test. The client finds the record index using a public map, whereas keyword lookup retrieves buckets from multiple segments. Direct indexing is therefore the next baseline to investigate. This result alone does not settle the final backend choice."),
    para("PIR hides which record is requested, but it does not make the returned bytes trustworthy. The client must still derive the expected storage key, check the snapshot and verify the proof. A failed private request must not trigger an ordinary RPC lookup for the same address."),
    para("The full padded generation, metadata and index map together occupy 109,390 bytes. That is a very small dataset for judging PIR performance. These runs show that the pieces work together. They do not show how useful PIR will be at larger scales. They also run locally, so network delay and concurrent clients have not yet been measured."),
    PageBreak(),
    section("6", "What the Project Is Trying to Find Out"),
    para("The useful question now is <b>when private, verifiable Ethereum reads become practical</b>. Getting a PIR query to work is necessary, but it is only the starting point. The results should help explain which workloads justify the extra setup and server work, and which costs make the approach less useful."),
    para("The current 32 record corpus is mainly a correctness test. Its comparison with downloading the full dataset cannot answer the scale question. At the same time, increasing the number of records is not enough to claim that PIR is better. The comparison needs to include how often the client reads, how much it can store and how often the snapshot changes."),
    para("I want the final evaluation to give a concrete answer for a stated workload, rather than just show that one query succeeded. For example, it should explain how the costs change between a client making one read and a client making many reads before the next snapshot."),
    section("7", "Plan for the Next Experiments"),
    para("The existing real proofs will stay in the regression tests. The scale experiments will increase the number of records and total database size across several orders of magnitude. The sizes need to be chosen after estimating memory use. Security parameters will stay fixed, rather than being weakened to make a larger test run."),
    data_table([
        ["What will vary", "What needs to be measured"],
        ["Proof representation", "Complete proof records versus shared trie nodes, including padding, index maps and the number of private queries"],
        ["Client usage", "First setup, repeated reads and moving to a new snapshot, including hint costs and client storage"],
        ["Server resources", "Preprocessing time, query time, memory use and latency distributions, including sizes that cannot be run"],
        ["Comparison methods", "Downloading and caching the generation once, and ordinary RPC as a revealing reference"],
    ], [40 * mm, 128 * mm]),
    Spacer(1, 5),
    para("Synthetic data can help measure backend scaling before a larger real corpus is available. Simply repeating these 32 records would not tell us how real proof sizes, shared nodes or updates behave. Those results will be labelled separately. Network measurements will also be kept separate from local compute timings."),
    para("The node based approach needs a privacy check as well as a size comparison. If the number or sequence of node requests reveals part of the selected path, smaller storage is not enough. Each comparison must use the same requested data and proof requirements."),
    section("8", "Work Remaining"),
    para("The next step is the scale and representation comparison. After that, the record layout and PIR baseline can be reviewed before building snapshot publication and the complete client. Helios still needs live sync and recovery tests. A new corpus must then be tied to a root accepted by the light client, or an authenticated history path is needed for an older block."),
    para("The project is still in Phase 1. The verifier and Helios choices are settled. The final PIR backend, record size, Helios integration method and repository licence are not. Network serving and checks that all files belong to the same published generation also remain to be implemented."),
    subsection("References used"),
    literature_item(1, "eth_trie 0.6.1", "Account and storage proof verification in the Rust adapter.", "https://docs.rs/eth_trie/0.6.1/eth_trie/"),
    literature_item(2, "IKPIR", "Implementation used for the direct and keyword PIR experiments.", "https://github.com/orochi-network/IKPIR"),
    literature_item(3, "Helios", "Consensus code used for the signed update and finalised root tests.", "https://github.com/a16z/helios"),
    literature_item(4, "SimplePIR and DoublePIR", "Background on setup and communication costs, USENIX Security 2023.", "https://www.usenix.org/conference/usenixsecurity23/presentation/henzinger"),
    literature_item(5, "EIP-1186", "The account and storage proof response format.", "https://eips.ethereum.org/EIPS/eip-1186"),
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
