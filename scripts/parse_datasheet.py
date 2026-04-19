#!/usr/bin/env python3
"""
Datasheet PDF Parser

Parses hardware datasheet PDFs into structured Markdown chapters, extracts images,
and generates an index.yaml for downstream consumption by spec authoring commands.

Uses pymupdf4llm for Markdown extraction (including tables as GFM Markdown).

Usage:
    uv run .claude/scripts/parse_datasheet.py <pdf_path>
    uv run .claude/scripts/parse_datasheet.py <pdf_path> --output-dir ./parsed/
    uv run .claude/scripts/parse_datasheet.py <pdf_path> --table-strategy lines_strict

Exit codes:
    0 — All phases completed successfully
    1 — Partial success (some phases had warnings/failures, but output was produced)
    2 — Fatal error (no usable output produced)
"""

# /// script
# dependencies = [
#   "pymupdf4llm>=1.27.2",
#   "pyyaml>=6.0",
# ]
# requires-python = ">=3.12"
# ///

import argparse
import os
import re
import shutil
import sys
from pathlib import Path

try:
    import pymupdf
    import pymupdf4llm
except ImportError:
    print(
        "ERROR: pymupdf4llm is required. Install with: uv add pymupdf4llm",
        file=sys.stderr,
    )
    sys.exit(2)

try:
    import yaml
except ImportError:
    print(
        "ERROR: PyYAML is required. Install with: uv add pyyaml",
        file=sys.stderr,
    )
    sys.exit(2)


# ---------------------------------------------------------------------------
# Result collector (mirrors ValidationResult from validate_spec.py)
# ---------------------------------------------------------------------------

class ParseResult:
    """Collects PASS / FAIL results across parsing phases."""

    def __init__(self):
        self.results: list[tuple[str, bool, str]] = []  # (phase, passed, detail)

    def passed(self, phase: str, detail: str = "") -> None:
        self.results.append((phase, True, detail))

    def failed(self, phase: str, reason: str) -> None:
        self.results.append((phase, False, reason))

    @property
    def all_passed(self) -> bool:
        return all(ok for _, ok, _ in self.results)

    @property
    def any_passed(self) -> bool:
        return any(ok for _, ok, _ in self.results)

    def print_report(self) -> None:
        for phase, ok, detail in self.results:
            tag = "[PASS]" if ok else "[FAIL]"
            suffix = f": {detail}" if detail else ""
            print(f"{tag} {phase}{suffix}")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def slugify(title: str) -> str:
    """Lowercase, strip special chars, replace spaces with underscores, truncate to 60 chars."""
    slug = title.lower().strip()
    slug = re.sub(r"[^\w\s-]", "", slug)
    slug = re.sub(r"[\s-]+", "_", slug)
    slug = slug.strip("_")
    return slug[:60]


# ---------------------------------------------------------------------------
# ToC detection — 3-tier fallback
# ---------------------------------------------------------------------------

def detect_toc_tier(doc: pymupdf.Document) -> tuple[int, list]:
    """
    Try to detect table of contents using 3 tiers:
      1. Native ToC from PDF metadata
      2. Font-size heuristics via pymupdf4llm
      3. Fixed-page split (every 10 pages)

    Returns (tier_number, toc_entries) where toc_entries is a list of
    [level, title, page_number] for tiers 1-2, or chapter dicts for tier 3.
    """
    # Tier 1: Native ToC
    toc = doc.get_toc()
    if toc:
        # Filter to level-1 entries as chapter boundaries
        level1 = [entry for entry in toc if entry[0] == 1]
        if len(level1) >= 2:
            return 1, toc

    # Tier 2: Font-size heuristics
    try:
        hdr_info = pymupdf4llm.IdentifyHeaders(doc)
        if hasattr(hdr_info, "header_id") and hdr_info.header_id:
            # header_id maps font characteristics to markdown heading levels
            # We need to extract chapter boundaries from this
            # IdentifyHeaders returns a dict-like mapping font info → heading level
            # We'll use this in group_by_font_heuristics
            return 2, hdr_info
    except Exception:
        pass

    # Tier 3: Fixed-page split
    return 3, []


def group_pages_into_chapters(toc: list, page_count: int) -> list[dict]:
    """
    Group pages into chapters using native ToC entries (tier 1).

    Each chapter dict has: title, start_page (0-indexed), end_page (exclusive).
    """
    # Extract level-1 entries as chapter boundaries
    level1 = [entry for entry in toc if entry[0] == 1]

    if not level1:
        # Fallback: use all entries
        level1 = toc

    chapters = []
    for i, entry in enumerate(level1):
        _level, title, page_num = entry
        start_page = page_num - 1  # ToC pages are 1-indexed
        start_page = max(0, min(start_page, page_count - 1))

        if i + 1 < len(level1):
            end_page = level1[i + 1][2] - 1  # next chapter's start
            end_page = max(start_page + 1, min(end_page, page_count))
        else:
            end_page = page_count

        chapters.append({
            "title": title.strip(),
            "start_page": start_page,
            "end_page": end_page,
        })

    return chapters


def group_by_font_heuristics(doc: pymupdf.Document, chunks: list[dict]) -> list[dict]:
    """
    Group chunks into chapters using font-size heuristics (tier 2).

    Falls through to tier 3 if fewer than 3 chapters detected for 50+ page docs.
    """
    page_count = doc.page_count

    # Scan all text blocks for the largest font sizes to identify chapter headings
    # We'll look at the first line of each page for potential chapter headings
    font_sizes: dict[float, list[tuple[int, str]]] = {}

    for page_num in range(page_count):
        page = doc[page_num]
        blocks = page.get_text("dict")["blocks"]
        for block in blocks:
            if block.get("type") != 0:  # text block
                continue
            for line in block.get("lines", []):
                for span in line.get("spans", []):
                    size = round(span.get("size", 0), 1)
                    text = span.get("text", "").strip()
                    if text and size > 0:
                        font_sizes.setdefault(size, []).append((page_num, text))
            break  # only first line of each block
        break  # only first block

    if not font_sizes:
        return group_by_fixed_pages(page_count)

    # Largest font size entries are likely chapter headings
    sorted_sizes = sorted(font_sizes.keys(), reverse=True)
    # Try the largest font size first
    for candidate_size in sorted_sizes[:3]:
        entries = font_sizes[candidate_size]
        # Filter to entries with meaningful text (not just numbers or short strings)
        meaningful = [(pg, txt) for pg, txt in entries if len(txt) > 2]
        if len(meaningful) >= 2:
            # Build chapters from these headings
            chapters = []
            for i, (pg, txt) in enumerate(meaningful):
                end_pg = meaningful[i + 1][0] if i + 1 < len(meaningful) else page_count
                chapters.append({
                    "title": txt.strip(),
                    "start_page": pg,
                    "end_page": end_pg,
                })

            # Tier 2 falls through to tier 3 if < 3 chapters for 50+ page document
            if len(chapters) < 3 and page_count >= 50:
                continue
            return chapters

    # Fallback to tier 3
    return group_by_fixed_pages(page_count)


def group_by_fixed_pages(page_count: int, chunk_size: int = 10) -> list[dict]:
    """
    Group pages into fixed-size chapters (tier 3 fallback).
    """
    chapters = []
    for start in range(0, page_count, chunk_size):
        end = min(start + chunk_size, page_count)
        chapters.append({
            "title": f"Pages {start + 1}-{end}",
            "start_page": start,
            "end_page": end,
        })
    return chapters


# ---------------------------------------------------------------------------
# Chapter writing
# ---------------------------------------------------------------------------

def assign_chunks_to_chapters(chunks: list[dict], chapters: list[dict]) -> dict[int, list[dict]]:
    """Map each chunk to its chapter index based on page numbers."""
    chapter_chunks: dict[int, list[dict]] = {i: [] for i in range(len(chapters))}

    for chunk in chunks:
        chunk_page = chunk.get("metadata", {}).get("page", 0)
        assigned = False
        for i, ch in enumerate(chapters):
            if ch["start_page"] <= chunk_page < ch["end_page"]:
                chapter_chunks[i].append(chunk)
                assigned = True
                break
        if not assigned and chapters:
            # Assign to last chapter
            chapter_chunks[len(chapters) - 1].append(chunk)

    return chapter_chunks


def write_chapter_files(
    chapters: list[dict],
    chunks: list[dict],
    chapters_dir: Path,
) -> int:
    """
    Write chapter Markdown files. Returns number of files written.

    Files are named: 00_table_of_contents.md, 01_<slug>.md, etc.
    """
    chapters_dir.mkdir(parents=True, exist_ok=True)
    chapter_chunks = assign_chunks_to_chapters(chunks, chapters)
    files_written = 0

    for i, chapter in enumerate(chapters):
        slug = slugify(chapter["title"])
        filename = f"{i:02d}_{slug}.md"
        filepath = chapters_dir / filename

        # Collect markdown text for this chapter
        chapter_md_parts = []
        chapter_md_parts.append(f"# {chapter['title']}\n")
        chapter_md_parts.append(
            f"*Pages {chapter['start_page'] + 1}–{chapter['end_page']}*\n"
        )

        for chunk in chapter_chunks.get(i, []):
            text = chunk.get("text", "")
            if text.strip():
                chapter_md_parts.append(text)

        filepath.write_text("\n".join(chapter_md_parts), encoding="utf-8")
        chapter["filename"] = filename
        files_written += 1

    return files_written


# ---------------------------------------------------------------------------
# Image extraction
# ---------------------------------------------------------------------------

def collect_images(
    chunks: list[dict],
    images_dir: Path,
    image_size_limit: float,
) -> list[dict]:
    """
    Collect images written by pymupdf4llm, rename to page_NNN_img_NNN.png convention,
    and filter by size relative to page.

    Returns list of image metadata dicts.
    """
    images_dir.mkdir(parents=True, exist_ok=True)
    image_records: list[dict] = []
    img_counter: dict[int, int] = {}  # page_num -> counter

    for chunk in chunks:
        if not chunk.get("images"):
            continue

        page_num = chunk.get("metadata", {}).get("page", 0)

        for img_info in chunk["images"]:
            # pymupdf4llm may store images as file paths or inline
            # Check if there's a file path reference
            img_path = None
            if isinstance(img_info, dict):
                img_path = img_info.get("path") or img_info.get("filename")
                img_bbox = img_info.get("bbox", [0, 0, 0, 0])
            elif isinstance(img_info, str):
                img_path = img_info
                img_bbox = [0, 0, 0, 0]
            else:
                continue

            if not img_path or not Path(img_path).exists():
                continue

            # Check image size relative to page
            if len(img_bbox) >= 4:
                img_width = abs(img_bbox[2] - img_bbox[0])
                img_height = abs(img_bbox[3] - img_bbox[1])
                img_area = img_width * img_height
                # Approximate page area (A4-ish: 595 x 842 points)
                page_area = 595 * 842
                if page_area > 0 and (img_area / page_area) < image_size_limit:
                    continue

            # Rename to convention
            counter = img_counter.get(page_num, 0) + 1
            img_counter[page_num] = counter
            new_name = f"page_{page_num + 1:03d}_img_{counter:03d}.png"
            dest_path = images_dir / new_name

            try:
                shutil.copy2(img_path, dest_path)
                image_records.append({
                    "filename": new_name,
                    "page": page_num + 1,
                    "original": Path(img_path).name,
                })
            except Exception:
                pass

    return image_records


# ---------------------------------------------------------------------------
# Index generation
# ---------------------------------------------------------------------------

def write_index_yaml(
    chapters: list[dict],
    images: list[dict],
    toc_tier: int,
    pdf_filename: str,
    index_path: Path,
) -> None:
    """Write index.yaml summarizing parsed output."""
    # Build chapter index
    chapter_index = []
    for ch in chapters:
        entry = {
            "title": ch["title"],
            "filename": ch.get("filename", ""),
            "pages": f"{ch['start_page'] + 1}-{ch['end_page']}",
        }
        chapter_index.append(entry)

    # Build image index grouped by page
    image_index = []
    for img in images:
        image_index.append({
            "filename": img["filename"],
            "page": img["page"],
        })

    index_data = {
        "source_pdf": pdf_filename,
        "toc_detection_tier": toc_tier,
        "toc_tier_description": {
            1: "native_toc",
            2: "font_size_heuristics",
            3: "fixed_page_split",
        }.get(toc_tier, "unknown"),
        "chapters": chapter_index,
        "total_chapters": len(chapter_index),
        "images": image_index,
        "total_images": len(image_index),
    }

    index_path.parent.mkdir(parents=True, exist_ok=True)
    with open(index_path, "w", encoding="utf-8") as f:
        yaml.dump(index_data, f, default_flow_style=False, sort_keys=False, allow_unicode=True)


# ---------------------------------------------------------------------------
# Orchestrator
# ---------------------------------------------------------------------------

def extract_chapters(
    pdf_path: Path,
    output_dir: Path,
    table_strategy: str,
    image_size_limit: float,
) -> ParseResult:
    """
    Main orchestrator: open PDF, detect ToC, extract markdown, split into chapters,
    write files, extract images, and generate index.
    """
    result = ParseResult()

    # Phase: pdf_open
    try:
        doc = pymupdf.open(str(pdf_path))
        page_count = doc.page_count
        result.passed("pdf_open", f"{page_count} pages")
    except Exception as e:
        result.failed("pdf_open", str(e))
        return result

    # Phase: toc_detection
    try:
        toc_tier, toc_data = detect_toc_tier(doc)
        tier_names = {1: "native ToC", 2: "font-size heuristics", 3: "fixed-page split"}
        result.passed("toc_detection", f"tier {toc_tier} ({tier_names.get(toc_tier, 'unknown')})")
    except Exception as e:
        result.failed("toc_detection", str(e))
        toc_tier = 3
        toc_data = []

    # Phase: markdown_extraction
    try:
        # Set up a temporary image directory for pymupdf4llm
        temp_image_dir = output_dir / "_temp_images"
        temp_image_dir.mkdir(parents=True, exist_ok=True)

        chunks = pymupdf4llm.to_markdown(
            doc,
            page_chunks=True,
            write_images=True,
            image_path=str(temp_image_dir),
            table_strategy=table_strategy,
        )

        # pymupdf4llm returns either a string or list of dicts depending on page_chunks
        if isinstance(chunks, str):
            # Single string — wrap in a chunk-like structure
            chunks = [{"text": chunks, "metadata": {"page": 0}, "images": []}]
        elif isinstance(chunks, list):
            # Normalize: ensure each chunk is a dict with expected keys
            normalized = []
            for i, chunk in enumerate(chunks):
                if isinstance(chunk, str):
                    normalized.append({"text": chunk, "metadata": {"page": i}, "images": []})
                elif isinstance(chunk, dict):
                    if "metadata" not in chunk:
                        chunk["metadata"] = {"page": i}
                    elif "page" not in chunk["metadata"]:
                        chunk["metadata"]["page"] = i
                    if "images" not in chunk:
                        chunk["images"] = []
                    normalized.append(chunk)
            chunks = normalized

        total_text_len = sum(len(c.get("text", "")) for c in chunks)
        result.passed("markdown_extraction", f"{len(chunks)} page chunks, {total_text_len} chars")
    except Exception as e:
        result.failed("markdown_extraction", str(e))
        # Clean up temp dir
        if temp_image_dir.exists():
            shutil.rmtree(temp_image_dir, ignore_errors=True)
        doc.close()
        return result

    # Phase: chapter_splitting
    try:
        if toc_tier == 1:
            chapters = group_pages_into_chapters(toc_data, page_count)
        elif toc_tier == 2:
            chapters = group_by_font_heuristics(doc, chunks)
        else:
            chapters = group_by_fixed_pages(page_count)

        if not chapters:
            chapters = group_by_fixed_pages(page_count)

        result.passed("chapter_splitting", f"{len(chapters)} chapters")
    except Exception as e:
        result.failed("chapter_splitting", str(e))
        chapters = group_by_fixed_pages(page_count)

    # Phase: chapter_writing
    chapters_dir = output_dir / "chapters"
    try:
        files_written = write_chapter_files(chapters, chunks, chapters_dir)
        result.passed("chapter_writing", f"{files_written} files written")
    except Exception as e:
        result.failed("chapter_writing", str(e))

    # Phase: image_extraction
    images_dir = output_dir / "images"
    image_records: list[dict] = []
    # Maps temp image path fragments to final relative paths for rewriting chapter markdown
    image_path_rewrites: dict[str, str] = {}
    try:
        # Collect images from temp dir that pymupdf4llm created
        if temp_image_dir.exists():
            temp_images = list(temp_image_dir.glob("*.png")) + list(temp_image_dir.glob("*.jpg"))
            if temp_images:
                images_dir.mkdir(parents=True, exist_ok=True)
                img_counter: dict[int, int] = {}

                # Build a mapping from image filenames to page numbers
                # by scanning chunk text for image references
                img_to_page: dict[str, int] = {}
                for chunk in chunks:
                    page_num = chunk.get("metadata", {}).get("page", 0)
                    text = chunk.get("text", "")
                    for img_file in temp_images:
                        if img_file.name in text:
                            img_to_page[img_file.name] = page_num

                for img_file in sorted(temp_images):
                    # Determine page for this image
                    page_num = img_to_page.get(img_file.name, 0)

                    # Check image file size as a rough proxy for significance
                    file_size = img_file.stat().st_size
                    if file_size < 1024:  # skip tiny images (< 1KB)
                        continue

                    counter = img_counter.get(page_num, 0) + 1
                    img_counter[page_num] = counter
                    new_name = f"page_{page_num + 1:03d}_img_{counter:03d}.png"
                    dest_path = images_dir / new_name

                    shutil.copy2(img_file, dest_path)
                    image_records.append({
                        "filename": new_name,
                        "page": page_num + 1,
                        "original": img_file.name,
                    })
                    # Record rewrite: map original filename to final relative path
                    image_path_rewrites[img_file.name] = f"../images/{new_name}"

            result.passed("image_extraction", f"{len(image_records)} images collected")

            # Clean up temp images
            shutil.rmtree(temp_image_dir, ignore_errors=True)
        else:
            result.passed("image_extraction", "no images found")
    except Exception as e:
        result.failed("image_extraction", str(e))
        if temp_image_dir.exists():
            shutil.rmtree(temp_image_dir, ignore_errors=True)

    # Phase: rewrite image paths in chapter files
    if image_path_rewrites and chapters_dir.exists():
        for md_file in sorted(chapters_dir.glob("*.md")):
            try:
                content = md_file.read_text(encoding="utf-8")
                for orig_name, new_rel_path in image_path_rewrites.items():
                    # Replace any path containing the original temp filename
                    # Handles both relative and absolute paths pymupdf4llm may have written
                    content = re.sub(
                        r"!\[([^\]]*)\]\([^)]*" + re.escape(orig_name) + r"\)",
                        f"![\\1]({new_rel_path})",
                        content,
                    )
                md_file.write_text(content, encoding="utf-8")
            except Exception:
                pass  # Best-effort rewrite

    # Phase: index_generation
    index_path = output_dir / "index.yaml"
    try:
        write_index_yaml(chapters, image_records, toc_tier, pdf_path.name, index_path)
        result.passed("index_generation", str(index_path))
    except Exception as e:
        result.failed("index_generation", str(e))

    doc.close()
    return result


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Parse a hardware datasheet PDF into structured Markdown chapters.",
    )
    parser.add_argument(
        "pdf_path",
        type=Path,
        help="Path to the PDF file to parse",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=None,
        help="Output directory (default: <pdf_parent>/parsed/)",
    )
    parser.add_argument(
        "--image-size-limit",
        type=float,
        default=0.05,
        help="Skip images smaller than this fraction of page area (default: 0.05 = 5%%)",
    )
    parser.add_argument(
        "--table-strategy",
        type=str,
        default="lines_strict",
        choices=["lines", "lines_strict", "text", "explicit"],
        help="Table detection strategy for pymupdf4llm (default: lines_strict)",
    )
    args = parser.parse_args()

    # Validate input
    if not args.pdf_path.exists():
        print(f"ERROR: PDF file not found: {args.pdf_path}", file=sys.stderr)
        sys.exit(2)

    if not args.pdf_path.suffix.lower() == ".pdf":
        print(f"ERROR: File does not have .pdf extension: {args.pdf_path}", file=sys.stderr)
        sys.exit(2)

    # Determine output directory
    output_dir = args.output_dir
    if output_dir is None:
        output_dir = args.pdf_path.parent / "parsed"

    print(f"Parsing: {args.pdf_path}")
    print(f"Output:  {output_dir}")
    print()

    result = extract_chapters(
        pdf_path=args.pdf_path,
        output_dir=output_dir,
        table_strategy=args.table_strategy,
        image_size_limit=args.image_size_limit,
    )

    print()
    result.print_report()

    if result.all_passed:
        print("\nAll phases completed successfully.")
        sys.exit(0)
    elif result.any_passed:
        print("\nPartial success — some phases had issues.")
        sys.exit(1)
    else:
        print("\nFatal — no usable output produced.")
        sys.exit(2)


if __name__ == "__main__":
    main()
