# Project progress reports

[Progress Report 2](progress-report-2.pdf) is the final three-page A4 update, prepared on
15 October 2026 for Rohan Arora (CS26S022). It records local proof/PIR results, the synthetic
Helios feasibility boundary, and the planned scale/workload study. It does not claim live
consensus authentication or completed scale evaluation.

The PDF matches the current local deliverable. Its wording was revised at Rohan's request to
use a plainer student progress-report voice while retaining the measured results and limitations. Temporary renders, draft PDFs and audit files
are deliberately excluded. The earlier Bitcoin assignment is outside this Git repository.

## Rebuild

The source uses ReportLab and standard PDF fonts. Install the pinned dependency in a virtual
environment, then choose a fresh output path to avoid changing the published PDF:

```sh
python3 -m venv /tmp/pvrpc-report-venv
/tmp/pvrpc-report-venv/bin/python -m pip install -r reports/requirements.txt
/tmp/pvrpc-report-venv/bin/python reports/build_progress_report_2.py --output /tmp/pvrpc-progress-report-2.pdf
```

The builder also works from another working directory. Omitting `--output` rebuilds the PDF
beside the script. Creation timestamps may differ from the retained PDF. Review all three pages
after content or dependency changes. The reviewed report has six embedded reference links and
uses the same academic style as Part 1, expanded to three pages at Rohan's request.
