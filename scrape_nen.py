"""
NEN Connect GQL Standard - Split into per-chapter markdown files.
Uses proper titles from the standard's TOC.
"""

import re
from pathlib import Path

SOURCE = Path(r"h:\gql-wire-protocol\.claude\reference\iso-iec-39075-gql-standard.md")
OUT_DIR = Path(r"h:\gql-wire-protocol\.claude\reference\iso-iec-39075")

# Titles from the ISO/IEC 39075 TOC
TITLES = {
    "sec_titlepage": "Title Page",
    "sec_foreword": "Foreword",
    "sec_intro": "Introduction",
    "sec_1": "1 - Scope",
    "sec_2": "2 - Normative References",
    "sec_3": "3 - Terms and Definitions",
    "sec_4": "4 - Concepts",
    "sec_5": "5 - Notation and Conventions",
    "sec_6": "6 - GQL-program",
    "sec_7": "7 - Session Management",
    "sec_8": "8 - Transaction Management",
    "sec_9": "9 - Procedure Specification",
    "sec_10": "10 - Variable Definitions",
    "sec_11": "11 - Object Expressions",
    "sec_12": "12 - Catalog-modifying Statements",
    "sec_13": "13 - Data-modifying Statements",
    "sec_14": "14 - Query Statements",
    "sec_15": "15 - Procedure Calling",
    "sec_15.2": "15.2 - Inline Procedure Call",
    "sec_15.3": "15.3 - Named Procedure Call",
    "sec_16": "16 - Common Elements",
    "sec_16.2": "16.2 - Use Graph Clause",
    "sec_16.3": "16.3 - Graph Pattern Binding Table",
    "sec_16.4": "16.4 - Graph Pattern",
    "sec_16.5": "16.5 - Insert Graph Pattern",
    "sec_16.6": "16.6 - Path Pattern Prefix",
    "sec_16.7": "16.7 - Path Pattern Expression",
    "sec_16.8": "16.8 - Label Expression",
    "sec_16.9": "16.9 - Path Variable Reference",
    "sec_16.10": "16.10 - Element Variable Reference",
    "sec_16.11": "16.11 - Graph Pattern Quantifier",
    "sec_16.12": "16.12 - Simplified Path Pattern Expression",
    "sec_16.13": "16.13 - Where Clause",
    "sec_16.14": "16.14 - Yield Clause",
    "sec_16.15": "16.15 - Group By Clause",
    "sec_16.16": "16.16 - Order By Clause",
    "sec_16.17": "16.17 - Sort Specification List",
    "sec_16.18": "16.18 - Limit Clause",
    "sec_16.19": "16.19 - Offset Clause",
    "sec_17": "17 - Object References",
    "sec_18": "18 - Type Elements",
    "sec_19": "19 - Predicates",
    "sec_20": "20 - Value Expressions and Specifications",
    "sec_21": "21 - Lexical Elements",
    "sec_22": "22 - Additional Common Rules",
    "sec_23": "23 - GQLSTATUS and Diagnostic Records",
    "sec_24": "24 - Conformance",
    "sec_24.4": "24.4 - Requirements for GQL-programs",
    "sec_A": "Annex A - SQL Alignment",
    "sec_B": "Annex B - Implementation-defined Elements",
    "sec_C": "Annex C - Implementation-dependent Elements",
    "sec_D": "Annex D - Deprecated Features",
    "sec_E": "Annex E - Optional Features",
    "sec_bibl": "Bibliography",
}


def main():
    text = SOURCE.read_text(encoding="utf-8")

    parts = re.split(
        r"\n={60}\nChapter: (sec_[\w.]+)\n={60}\n",
        text,
    )

    OUT_DIR.mkdir(parents=True, exist_ok=True)

    index_lines = [
        "# ISO/IEC 39075 - GQL Standard",
        "",
        "Extracted from NEN Connect (NEN-ISO/IEC 39075:2024)",
        "",
        "## Chapters",
        "",
    ]

    count = 0
    for i in range(1, len(parts), 2):
        sec_id = parts[i]
        content = parts[i + 1].strip()
        title = TITLES.get(sec_id, sec_id)
        slug = sec_id.replace(".", "-")
        filename = f"{slug}.md"

        md = f"# {title}\n\n{content}\n"
        (OUT_DIR / filename).write_text(md, encoding="utf-8")

        index_lines.append(f"- [{title}]({filename})")
        count += 1
        print(f"  {filename:<24} {len(content):>8,} chars  {title}")

    index_lines.append("")
    (OUT_DIR / "README.md").write_text(
        "\n".join(index_lines), encoding="utf-8"
    )
    print(f"\nDone: {count} chapter files + README.md in {OUT_DIR}")


if __name__ == "__main__":
    main()
