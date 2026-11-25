#!/usr/bin/env python3
"""Compare `stress --suite` results across machines.

    python3 tools/compare-bench.py bench/*.jsonl

Prints one table per metric: a row per scenario, a column per machine
label. If a file holds several runs for a label, the latest one wins.
"""

import json
import sys
from collections import OrderedDict

METRICS = [("fps", "avg FPS (higher is better)", "{:.1f}"), ("p95_ms", "p95 frame ms (lower is better)", "{:.1f}"),
           ("on_screen", "widgets on screen", "{:d}")]


def main(paths):
    if not paths:
        sys.exit(__doc__)
    rows = []
    for p in paths:
        with open(p, encoding="utf8") as f:
            rows += [json.loads(line) for line in f if line.strip()]
    labels = list(OrderedDict.fromkeys(r["label"] for r in rows))
    scenarios = list(OrderedDict.fromkeys(r["scenario"] for r in rows))
    latest = {(r["label"], r["scenario"]): r for r in rows}

    print("machines:")
    for lab in labels:
        r = next(r for r in rows if r["label"] == lab)
        print(f"  {lab}: {r['os']}/{r['arch']}, {r['cpus']} cpus, scale {r['scale']}, viewport {r['viewport'][0]:.0f}x{r['viewport'][1]:.0f}")
    print("  (FPS is capped by each display's refresh rate)\n")

    w = max(14, *(len(s) for s in scenarios))
    for key, title, fmt in METRICS:
        print(title)
        print("  " + "scenario".ljust(w) + "".join(lab[:16].rjust(18) for lab in labels))
        for sc in scenarios:
            cells = []
            for lab in labels:
                r = latest.get((lab, sc))
                cells.append((fmt.format(r[key]) if r else "-").rjust(18))
            print("  " + sc.ljust(w) + "".join(cells))
        print()


if __name__ == "__main__":
    main(sys.argv[1:])
