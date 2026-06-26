#!/usr/bin/env python3
"""Generate small deterministic PDF fixtures for Phase 0 smoke tests."""

from __future__ import annotations

import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "fixtures" / "generated"


class Pdf:
    def __init__(self) -> None:
        self.objects: list[bytes] = []

    def add(self, body: str | bytes) -> int:
        if isinstance(body, str):
            body = body.encode("ascii")
        self.objects.append(body)
        return len(self.objects)

    def render(
        self,
        root: int,
        offset_drift: dict[int, int] | None = None,
        trailer_entries: str | bytes = b"",
        header_version: str = "1.4",
    ) -> bytes:
        offset_drift = offset_drift or {}
        if isinstance(trailer_entries, str):
            trailer_entries = trailer_entries.encode("ascii")
        out = bytearray(f"%PDF-{header_version}\n".encode("ascii") + b"%\xe2\xe3\xcf\xd3\n")
        offsets = [0]
        for idx, body in enumerate(self.objects, start=1):
            offsets.append(len(out))
            out.extend(f"{idx} 0 obj\n".encode("ascii"))
            out.extend(body)
            out.extend(b"\nendobj\n")

        xref_offset = len(out)
        out.extend(f"xref\n0 {len(self.objects) + 1}\n".encode("ascii"))
        out.extend(b"0000000000 65535 f \n")
        for object_number, offset in enumerate(offsets[1:], start=1):
            adjusted = offset + offset_drift.get(object_number, 0)
            out.extend(f"{adjusted:010d} 00000 n \n".encode("ascii"))
        out.extend(
            (
                f"trailer\n<< /Size {len(self.objects) + 1} /Root {root} 0 R "
            ).encode("ascii")
        )
        out.extend(trailer_entries)
        out.extend(f">>\nstartxref\n{xref_offset}\n%%EOF\n".encode("ascii"))
        return bytes(out)


def page_pdf(media_box: str, content: str | bytes) -> bytes:
    pdf = Pdf()
    content_bytes = content.encode("ascii") if isinstance(content, str) else content
    contents = pdf.add(
        f"<< /Length {len(content_bytes)} >>\nstream\n".encode("ascii")
        + content_bytes
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R "
        f"/MediaBox {media_box} /Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def rotated_office_export_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.95 0.95 0.95 rg 0 0 160 100 re f "
        b"0.1 0.3 0.7 rg 12 62 136 22 re f "
        b"0 0 0 RG 1 w 12 20 136 64 re S 12 42 m 148 42 l S Q "
        b"BT /F1 12 Tf 18 76 Td (Rotated office export) Tj "
        b"0 -24 Td (Amount: 120.00) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 100] /Rotate 90 "
        f"/Resources << /Font << /F1 4 0 R >> >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def cropped_scan_page_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.86 g 0 0 180 180 re f "
        b"0.65 g 30 20 120 120 re f "
        b"0.2 g 40 40 100 16 re f 45 70 90 12 re f 40 96 100 16 re f Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 180 180] "
        f"/CropBox [30 20 150 140] /Resources << >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def user_unit_page_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.9 0.95 1 rg 0 0 80 60 re f "
        b"0.1 0.45 0.2 rg 10 12 60 24 re f Q "
        b"BT /F1 8 Tf 12 45 Td (UserUnit 2.0) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 80 60] /UserUnit 2 "
        f"/Resources << /Font << /F1 4 0 R >> >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def pdf20_basic_office_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.94 0.96 0.99 rg 0 0 220 140 re f "
        b"0.12 0.24 0.42 rg 20 90 180 26 re f "
        b"0 0 0 RG 0.8 w 20 40 180 50 re S 20 65 m 200 65 l S Q "
        b"BT /F1 10 Tf 28 102 Td (PDF 2.0 office baseline) Tj "
        b"0 -34 Td (Standard page content only) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 220 140] "
        f"/Resources << /Font << /F1 4 0 R >> >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Version /2.0 /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog, header_version="2.0")


def pdf20_associated_files_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.96 0.94 0.90 rg 0 0 240 140 re f "
        b"0.1 0.45 0.36 rg 24 36 192 64 re f Q "
        b"BT /F1 10 Tf 32 72 Td (PDF 2.0 associated file) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 240 140] "
        f"/Resources << /Font << /F1 4 0 R >> >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    embedded_payload = b"pdf20-associated-file,metadata-only\n"
    embedded = pdf.add(
        b"<< /Type /EmbeddedFile /Subtype /text#2Fplain /Length "
        + str(len(embedded_payload)).encode("ascii")
        + b" >>\nstream\n"
        + embedded_payload
        + b"endstream"
    )
    filespec = pdf.add(
        f"<< /Type /Filespec /F (payload.txt) /UF (payload.txt) "
        f"/AFRelationship /Data /EF << /F {embedded} 0 R >> >>"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Version /2.0 /Pages {pages} 0 R "
        f"/AF [{filespec} 0 R] "
        f"/Names << /EmbeddedFiles << /Names [(payload.txt) {filespec} 0 R] >> >> >>"
    )
    assert font == 4
    return pdf.render(catalog, header_version="2.0")


def pdf20_black_point_compensation_pdf() -> bytes:
    pdf = Pdf()
    content = b"/GSbp gs 0.2 0.4 0.6 rg 20 20 120 70 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 180 120] "
        "/Resources << /ExtGState << /GSbp << /Type /ExtGState "
        f"/UseBlackPtComp true >> >> >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Version /2.0 /Pages {pages} 0 R >>")
    return pdf.render(catalog, header_version="2.0")


def metadata_outline_page_labels_pdf() -> bytes:
    pdf = Pdf()
    content = b"0.12 0.18 0.28 rg 20 20 160 60 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 200 120] "
        f"/Resources << >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    metadata = b"<x:xmpmeta><dc:title>Metadata Fixture</dc:title></x:xmpmeta>"
    metadata_object = pdf.add(
        b"<< /Type /Metadata /Subtype /XML /Length "
        + str(len(metadata)).encode("ascii")
        + b" >>\nstream\n"
        + metadata
        + b"\nendstream"
    )
    info = pdf.add(
        "<< /Title (Metadata Fixture) /Author (pdfrust) "
        "/Creator (fixture generator) /Producer (pdfrust) >>"
    )
    outline_one = pdf.add(
        f"<< /Title (Chapter One) /Parent 8 0 R /Dest [{page} 0 R /Fit] /Next 7 0 R >>"
    )
    outline_two = pdf.add(
        f"<< /Title (Appendix) /Parent 8 0 R /Dest [{page} 0 R /Fit] >>"
    )
    outlines = pdf.add(
        f"<< /Type /Outlines /First {outline_one} 0 R /Last {outline_two} 0 R /Count 2 >>"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /Metadata {metadata_object} 0 R "
        f"/Outlines {outlines} 0 R "
        f"/PageLabels << /Nums [0 << /P (A-) /S /D /St 1 >>] >> "
        f"/Names << /Dests << /Names [(chapter-one) [{page} 0 R /Fit]] >> >> "
        "/MarkInfo << /Marked true >> "
        "/StructTreeRoot << /Type /StructTreeRoot /K [] >> >>"
    )
    return pdf.render(catalog, trailer_entries=f"/Info {info} 0 R ")


def tagged_accessibility_metadata_pdf() -> bytes:
    pdf = Pdf()
    content = b"/Figure << /MCID 0 >> BDC 0.1 0.3 0.7 rg 20 40 120 40 re f EMC"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 220 140] "
        f"/Resources << >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    structure = pdf.add(
        f"<< /Type /StructElem /S /Document /P 5 0 R /Pg {page} 0 R "
        f"/K [<< /Type /MCR /Pg {page} 0 R /MCID 0 >>] >>"
    )
    struct_tree = pdf.add(
        f"<< /Type /StructTreeRoot /K [{structure} 0 R] "
        "/RoleMap << /Document /Document >> >>"
    )
    info = pdf.add("<< /Title (Tagged Accessibility Fixture) /Author (pdfrust) >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /Lang (en-US) "
        "/MarkInfo << /Marked true >> "
        f"/StructTreeRoot {struct_tree} 0 R >>"
    )
    return pdf.render(catalog, trailer_entries=f"/Info {info} 0 R ")


def tagged_report_visual_integrity_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"/H1 << /MCID 0 >> BDC "
        b"q 0.12 0.18 0.28 rg 0 224 360 36 re f Q "
        b"BT /F1 12 Tf 24 238 Td (Tagged Quarterly Report) Tj ET EMC "
        b"/Table << /MCID 1 >> BDC "
        b"q 0.97 0.98 0.99 rg 0 0 360 224 re f Q "
        b"q 0.18 0.22 0.30 RG 0.7 w 24 54 312 132 re S "
        b"24 154 m 336 154 l S 24 122 m 336 122 l S 24 90 m 336 90 l S "
        b"132 54 m 132 186 l S 236 54 m 236 186 l S Q "
        b"q 0.18 0.48 0.72 rg 260 74 18 64 re f 290 74 18 84 re f Q "
        b"BT /F1 8 Tf 32 168 Td (Metric) Tj 108 0 Td (Value) Tj 104 0 Td (Status) Tj "
        b"-212 -32 Td (Revenue) Tj 108 0 Td (128) Tj 104 0 Td (Green) Tj "
        b"-212 -32 Td (Orders) Tj 108 0 Td (912) Tj 104 0 Td (Blue) Tj "
        b"-212 -32 Td (Queue) Tj 108 0 Td (17) Tj 104 0 Td (Amber) Tj ET EMC"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 360 260] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    struct_tree = pdf.add(
        f"<< /Type /StructTreeRoot /K [<< /S /Document /Pg {page} 0 R /K ["
        f"<< /S /H1 /Pg {page} 0 R /K << /Type /MCR /Pg {page} 0 R /MCID 0 >> >> "
        f"<< /S /Table /Pg {page} 0 R /K << /Type /MCR /Pg {page} 0 R /MCID 1 >> >>"
        "] >>] /RoleMap << /H1 /H /Table /Table /Document /Document >> >>"
    )
    info = pdf.add("<< /Title (Tagged Report Visual Integrity) /Author (pdfrust) >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /Lang (en-US) "
        "/MarkInfo << /Marked true >> "
        f"/StructTreeRoot {struct_tree} 0 R >>"
    )
    assert font == 4
    return pdf.render(catalog, trailer_entries=f"/Info {info} 0 R ")


def tagged_form_visual_integrity_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"/Form << /MCID 0 >> BDC "
        b"q 1 1 1 rg 0 0 300 200 re f Q "
        b"q 0.92 0.96 1 rg 0 164 300 36 re f Q "
        b"q 0.18 0.22 0.30 RG 0.7 w 24 44 252 108 re S "
        b"24 124 m 276 124 l S 24 96 m 276 96 l S 24 68 m 276 68 l S "
        b"120 44 m 120 152 l S Q "
        b"BT /F1 11 Tf 24 178 Td (Tagged Intake Form) Tj "
        b"/F1 8 Tf 32 136 Td (Field) Tj 96 0 Td (Value) Tj "
        b"-96 -28 Td (Name) Tj 96 0 Td (Example User) Tj "
        b"-96 -28 Td (Consent) Tj 96 0 Td (Accepted) Tj "
        b"-96 -28 Td (Date) Tj 96 0 Td (2026-06-26) Tj ET EMC"
    )
    checked_appearance = (
        b"1 1 1 rg 0 0 16 16 re f "
        b"0 0 0 RG 0.8 w 0.5 0.5 15 15 re S "
        b"0 0 0 RG 1.2 w 4 8 m 7 4 l S 7 4 m 13 13 l S"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 300 200] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R /Annots [6 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    appearance = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 16 16] /Length "
        + str(len(checked_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + checked_appearance
        + b"\nendstream"
    )
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Btn /T (TaggedConsent) /V /Yes /AS /Yes "
        "/Rect [32 74 48 90] "
        f"/AP << /N << /Yes {appearance} 0 R >> >> >>"
    )
    struct_tree = pdf.add(
        f"<< /Type /StructTreeRoot /K [<< /S /Document /Pg {page} 0 R /K ["
        f"<< /S /Form /Pg {page} 0 R /K << /Type /MCR /Pg {page} 0 R /MCID 0 >> >>"
        "] >>] /RoleMap << /Form /P /Document /Document >> >>"
    )
    info = pdf.add("<< /Title (Tagged Form Visual Integrity) /Author (pdfrust) >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /Lang (en-US) "
        f"/AcroForm << /Fields [{field} 0 R] >> "
        "/MarkInfo << /Marked true >> "
        f"/StructTreeRoot {struct_tree} 0 R >>"
    )
    assert font == 4
    assert appearance == 5
    assert field == 6
    return pdf.render(catalog, trailer_entries=f"/Info {info} 0 R ")


def tagged_office_alt_text_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 1 1 1 rg 0 0 420 280 re f Q "
        b"/Figure << /MCID 0 >> BDC "
        b"q 0.93 0.96 0.99 rg 28 74 176 138 re f Q "
        b"q 0.18 0.22 0.30 RG 0.7 w 28 74 176 138 re S "
        b"28 108 m 204 108 l S 28 142 m 204 142 l S 28 176 m 204 176 l S Q "
        b"q 0.18 0.48 0.72 rg 54 74 22 72 re f 94 74 22 96 re f 134 74 22 54 re f Q "
        b"q 0.85 0.36 0.14 RG 1.4 w 54 146 m 94 170 l 134 128 l 174 188 l S Q EMC "
        b"/P << /MCID 1 >> BDC "
        b"q 0.18 0.22 0.30 RG 0.7 w 236 76 150 136 re S "
        b"236 180 m 386 180 l S 236 148 m 386 148 l S 236 116 m 386 116 l S "
        b"306 76 m 306 212 l S Q "
        b"BT /F1 12 Tf 28 244 Td (Tagged Office Export) Tj "
        b"/F1 8 Tf 46 224 Td (Figure has alt text in structure metadata.) Tj "
        b"198 -28 Td (Item) Tj 70 0 Td (Status) Tj "
        b"-70 -32 Td (Plan) Tj 70 0 Td (Green) Tj "
        b"-70 -32 Td (Risk) Tj 70 0 Td (Amber) Tj "
        b"-70 -32 Td (Launch) Tj 70 0 Td (Blue) Tj ET EMC"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 420 280] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    struct_tree = pdf.add(
        f"<< /Type /StructTreeRoot /K [<< /S /Document /Pg {page} 0 R /K ["
        f"<< /S /Figure /Alt (Quarterly chart panel) /Pg {page} 0 R "
        f"/K << /Type /MCR /Pg {page} 0 R /MCID 0 >> >> "
        f"<< /S /P /Pg {page} 0 R /K << /Type /MCR /Pg {page} 0 R /MCID 1 >> >>"
        "] >>] /RoleMap << /Document /Document /Figure /Figure /P /P >> >>"
    )
    info = pdf.add("<< /Title (Tagged Office Alt Text Fixture) /Author (pdfrust) >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /Lang (en-US) "
        "/MarkInfo << /Marked true >> "
        f"/StructTreeRoot {struct_tree} 0 R >>"
    )
    assert font == 4
    return pdf.render(catalog, trailer_entries=f"/Info {info} 0 R ")


def tagged_structure_heavy_report_pdf() -> bytes:
    pdf = Pdf()
    ops: list[str] = [
        "q 0.98 0.99 1 rg 0 0 360 300 re f Q",
        "q 0.12 0.18 0.28 rg 0 262 360 38 re f Q",
        "BT /F1 12 Tf 24 276 Td (Structure Heavy Tagged Report) Tj ET",
    ]
    kids: list[str] = []
    for idx in range(64):
        x = 24 + (idx % 8) * 38
        y = 230 - (idx // 8) * 24
        ops.append(
            f"/P << /MCID {idx} >> BDC "
            f"q 0.18 0.22 0.30 RG 0.4 w {x} {y} 28 12 re S Q "
            f"BT /F1 5 Tf {x + 3} {y + 4} Td ({idx:02d}) Tj ET EMC"
        )
        kids.append(
            f"<< /S /P /Pg 2 0 R /K << /Type /MCR /Pg 2 0 R /MCID {idx} >> >>"
        )
    content = " ".join(ops).encode("ascii")
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 360 300] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    struct_tree = pdf.add(
        f"<< /Type /StructTreeRoot /K [<< /S /Document /Pg {page} 0 R /K ["
        + " ".join(kids)
        + "] >>] /RoleMap << /Document /Document /P /P >> >>"
    )
    info = pdf.add("<< /Title (Tagged Structure Heavy Report) /Author (pdfrust) >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /Lang (en-US) "
        "/MarkInfo << /Marked true >> "
        f"/StructTreeRoot {struct_tree} 0 R >>"
    )
    assert font == 4
    return pdf.render(catalog, trailer_entries=f"/Info {info} 0 R ")


def malformed_tagged_structure_pdf() -> bytes:
    pdf = Pdf()
    content = b"0.2 0.2 0.2 rg 20 20 80 40 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 80] "
        f"/Resources << >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R "
        "/MarkInfo << /Marked true >> "
        "/StructTreeRoot << /Type /StructTreeRoot /K << /S /Document /K true >> >> >>"
    )
    return pdf.render(catalog)


def malformed_xref_offset_drift_pdf() -> bytes:
    pdf = Pdf()
    content = b"q 0.9 0 0 rg 10 10 60 40 re f Q"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 80] "
        f"/Resources << >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog, offset_drift={contents: 1})


def form_xobject_pdf() -> bytes:
    pdf = Pdf()
    content = b"q 2 0 0 2 10 20 cm /Fm1 Do Q"
    form = b"0.2 0.7 0.3 rg 0 0 40 40 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /XObject << /Fm1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    form_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 40 40] "
        b"/Matrix [1 0 0 1 5 6] /Length "
        + str(len(form)).encode("ascii")
        + b" >>\nstream\n"
        + form
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert form_object == 4
    return pdf.render(catalog)


def image_xobject_pdf() -> bytes:
    pdf = Pdf()
    content = b"q 64 0 0 64 28 28 cm /Im1 Do Q"
    image = bytes(
        [
            255,
            0,
            0,
            0,
            255,
            0,
            0,
            0,
            255,
            255,
            255,
            0,
        ]
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def cmyk_image_pdf() -> bytes:
    pdf = Pdf()
    content = b"q 80 0 0 80 20 20 cm /Im1 Do Q"
    image = bytes(
        [
            0,
            255,
            255,
            0,
            255,
            0,
            255,
            0,
            255,
            255,
            0,
            0,
            0,
            0,
            0,
            255,
        ]
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 "
        b"/ColorSpace /DeviceCMYK /BitsPerComponent 8 /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def icc_based_image_pdf(
    color_space_name: bytes,
    components: int,
    image: bytes,
    profile: bytes,
) -> bytes:
    pdf = Pdf()
    content = b"q 80 0 0 80 20 20 cm /Im1 Do Q"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 "
        b"/ColorSpace [/ICCBased 6 0 R] /BitsPerComponent 8 /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    profile_object = pdf.add(
        b"<< /N "
        + str(components).encode("ascii")
        + b" /Alternate /"
        + color_space_name
        + b" /Length "
        + str(len(profile)).encode("ascii")
        + b" >>\nstream\n"
        + profile
        + b"\nendstream"
    )
    assert image_object == 4
    assert profile_object == 6
    return pdf.render(catalog)


def icc_rgb_image_pdf() -> bytes:
    image = bytes(
        [
            255,
            0,
            0,
            0,
            255,
            0,
            0,
            0,
            255,
            255,
            255,
            0,
        ]
    )
    return icc_based_image_pdf(
        b"DeviceRGB",
        3,
        image,
        b"pdfrust synthetic icc rgb profile",
    )


def icc_gray_image_pdf() -> bytes:
    image = bytes([0, 85, 170, 255])
    return icc_based_image_pdf(
        b"DeviceGray",
        1,
        image,
        b"pdfrust synthetic icc gray profile",
    )


def icc_cmyk_image_pdf() -> bytes:
    image = bytes(
        [
            0,
            255,
            255,
            0,
            255,
            0,
            255,
            0,
            255,
            255,
            0,
            0,
            0,
            0,
            0,
            255,
        ]
    )
    return icc_based_image_pdf(
        b"DeviceCMYK",
        4,
        image,
        b"pdfrust synthetic icc cmyk profile",
    )


def output_intent_rgb_pdf() -> bytes:
    pdf = Pdf()
    content = b"0.1 0.45 0.85 rg 20 20 80 50 re f"
    profile = b"pdfrust synthetic profile placeholder"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 90] "
        f"/Resources << >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    profile_object = pdf.add(
        b"<< /N 3 /Length "
        + str(len(profile)).encode("ascii")
        + b" >>\nstream\n"
        + profile
        + b"\nendstream"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /OutputIntents [<< /Type /OutputIntent "
        f"/S /GTS_PDFA1 /OutputConditionIdentifier (sRGB synthetic) /DestOutputProfile {profile_object} 0 R >>] >>"
    )
    assert profile_object == 4
    return pdf.render(catalog)


def indexed_image_pdf() -> bytes:
    pdf = Pdf()
    content = b"q 80 0 0 80 20 20 cm /Im1 Do Q"
    image = bytes([0, 1, 2, 3])
    palette = b"ff000000ff000000ffffffffff"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 "
        b"/ColorSpace [/Indexed /DeviceRGB 3 <"
        + palette
        + b">] /BitsPerComponent 8 /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def tiny_jpeg_fixture_bytes() -> bytes:
    return bytes.fromhex(
        "ffd8ffe000104a46494600010100000100010000"
        "ffdb0043000302020302020303030304030304050805050404050a070706080c0a0c0c0b0a0b0b0d0e12100d0e110e0b0b1016101113141515150c0f171816141812141514"
        "ffdb00430103040405040509050509140d0b0d1414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414"
        "ffc00011080004000403011100021101031101"
        "ffc4001f0000010501010101010100000000000000000102030405060708090a0b"
        "ffc400b5100002010303020403050504040000017d01020300041105122131410613516107227114328191a1082342b1c11552d1f02433627282090a161718191a25262728292a3435363738393a434445464748494a535455565758595a636465666768696a737475767778797a838485868788898a92939495969798999aa2a3a4a5a6a7a8a9aab2b3b4b5b6b7b8b9bac2c3c4c5c6c7c8c9cad2d3d4d5d6d7d8d9dae1e2e3e4e5e6e7e8e9eaf1f2f3f4f5f6f7f8f9fa"
        "ffc4001f0100030101010101010101010000000000000102030405060708090a0b"
        "ffc400b51100020102040403040705040400010277000102031104052131061241510761711322328108144291a1b1c109233352f0156272d10a162434e125f11718191a262728292a35363738393a434445464748494a535455565758595a636465666768696a737475767778797a82838485868788898a92939495969798999aa2a3a4a5a6a7a8a9aab2b3b4b5b6b7b8b9bac2c3c4c5c6c7c8c9cad2d3d4d5d6d7d8d9dae2e3e4e5e6e7e8e9eaf2f3f4f5f6f7f8f9fa"
        "ffda000c03010002110311003f00f9d2bf0c3fd533ffd9"
    )


def dct_image_pdf() -> bytes:
    pdf = Pdf()
    content = b"q 80 0 0 80 20 20 cm /Im1 Do Q"
    image = tiny_jpeg_fixture_bytes()
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 4 /Height 4 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def predictor_image_pdf() -> bytes:
    pdf = Pdf()
    content = b"q 80 0 0 80 20 20 cm /Im1 Do Q"
    row0 = bytes([255, 0, 0, 0, 255, 0])
    row1 = bytes([0, 0, 255, 255, 255, 0])
    encoded_row1 = bytes((current - previous) % 256 for current, previous in zip(row1, row0))
    image = zlib.compress(b"\x00" + row0 + b"\x02" + encoded_row1)
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode "
        b"/DecodeParms << /Predictor 15 /Colors 3 /Columns 2 /BitsPerComponent 8 >> /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def soft_mask_image_pdf() -> bytes:
    pdf = Pdf()
    content = b"q 80 0 0 80 20 20 cm /Im1 Do Q"
    image = bytes(
        [
            255,
            0,
            0,
            0,
            255,
            0,
            0,
            0,
            255,
            255,
            255,
            0,
        ]
    )
    mask = bytes([0, 128, 255, 64])
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /SMask 6 0 R /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    mask_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 "
        b"/ColorSpace /DeviceGray /BitsPerComponent 8 /Length "
        + str(len(mask)).encode("ascii")
        + b" >>\nstream\n"
        + mask
        + b"\nendstream"
    )
    assert image_object == 4
    assert mask_object == 6
    return pdf.render(catalog)


def pack_one_bit_rows(rows: list[str]) -> bytes:
    width = len(rows[0])
    row_bytes = (width + 7) // 8
    packed = bytearray()
    for row in rows:
        assert len(row) == width
        value = 0
        used = 0
        for bit in row:
            value = (value << 1) | (1 if bit == "1" else 0)
            used += 1
            if used == 8:
                packed.append(value)
                value = 0
                used = 0
        if used:
            packed.append(value << (8 - used))
        while len(packed) % row_bytes != 0:
            packed.append(0)
    return bytes(packed)


def expand_one_bit_rows(rows: list[str], x_factor: int, y_factor: int) -> list[str]:
    expanded = []
    for row in rows:
        wide = "".join(bit * x_factor for bit in row)
        expanded.extend([wide] * y_factor)
    return expanded


def image_mask_pdf(
    media_box: str,
    content: bytes,
    rows: list[str],
    *,
    extra: bytes = b"",
    compressed: bool = False,
) -> bytes:
    pdf = Pdf()
    mask = pack_one_bit_rows(rows)
    image_data = zlib.compress(mask) if compressed else mask
    width = len(rows[0])
    height = len(rows)
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R "
        f"/MediaBox {media_box} /Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    filter_entry = b"/Filter /FlateDecode " if compressed else b""
    image_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width {width} /Height {height} "
            "/ImageMask true /BitsPerComponent 1 "
        ).encode("ascii")
        + extra
        + filter_entry
        + f"/Length {len(image_data)} >>\nstream\n".encode("ascii")
        + image_data
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def image_mask_signature_pdf() -> bytes:
    rows = expand_one_bit_rows(
        [
            "000000000000000000000000",
            "001110001100011100011000",
            "010001010010100010100100",
            "010001100000100010100100",
            "001110011100111100111000",
            "000001000010100000100100",
            "111110111100100000100100",
            "000000000000000000000000",
        ],
        4,
        3,
    )
    content = (
        b"0.98 0.97 0.94 rg 0 0 120 80 re f "
        b"0.75 0.72 0.66 RG 1 w 12 28 m 108 28 l S "
        b"q 0.02 0.15 0.30 rg 96 0 0 24 12 32 cm /Im1 Do Q"
    )
    return image_mask_pdf("[0 0 120 80]", content, rows)


def image_mask_monochrome_icon_pdf() -> bytes:
    rows = expand_one_bit_rows(
        [
            "00000000",
            "00111100",
            "01000010",
            "01011010",
            "01011010",
            "01000010",
            "00111100",
            "00000000",
        ],
        4,
        4,
    )
    content = (
        b"0.94 0.96 0.98 rg 0 0 120 120 re f "
        b"q 0.04 0.45 0.22 rg 32 0 0 32 44 44 cm /Im1 Do Q"
    )
    return image_mask_pdf("[0 0 120 120]", content, rows)


def image_mask_logo_pdf() -> bytes:
    rows = expand_one_bit_rows(
        [
            "0000000000000000",
            "0011110000111100",
            "0110011001100110",
            "1100001111000011",
            "1100001111000011",
            "0110011001100110",
            "0011110000111100",
            "0001100000011000",
            "0001100000011000",
            "0011110000111100",
            "0110011001100110",
            "0000000000000000",
        ],
        4,
        4,
    )
    content = (
        b"0.12 0.13 0.15 rg 0 0 150 100 re f "
        b"q 0.95 0.76 0.18 rg 64 0 0 48 43 26 cm /Im1 Do Q"
    )
    return image_mask_pdf("[0 0 150 100]", content, rows, compressed=True)


def image_mask_inverted_icon_pdf() -> bytes:
    rows = [
        "000000000000000000000000",
        "001110001100011100011000",
        "010001010010100010100100",
        "010001100000100010100100",
        "001110011100111100111000",
        "000001000010100000100100",
        "111110111100100000100100",
        "000000000000000000000000",
    ]
    content = (
        b"0.94 0.96 0.98 rg 0 0 120 120 re f "
        b"q 0.04 0.45 0.22 rg 64 0 0 64 28 28 cm /Im1 Do Q"
    )
    return image_mask_pdf("[0 0 120 120]", content, rows, extra=b"/Decode [1 0] ")


def unsupported_image_codec_pdf(filter_name: str) -> bytes:
    pdf = Pdf()
    content = b"q 40 0 0 40 40 40 cm /Im1 Do Q"
    image = b"\x00"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 "
            f"/ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /{filter_name} "
            f"/Length {len(image)} >>\nstream\n"
        ).encode("ascii")
        + image
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def scanned_page_pdf() -> bytes:
    pdf = Pdf()
    width = 64
    height = 80
    image = bytes((x * 3 + y * 2) % 256 for y in range(height) for x in range(width))
    content = b"q 160 0 0 200 0 0 cm /Im1 Do Q"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 200] "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width {width} /Height {height} "
            f"/ColorSpace /DeviceGray /BitsPerComponent 8 /Length {len(image)} >>\n"
            "stream\n"
        ).encode("ascii")
        + image
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def mobile_rotated_camera_scan_pdf() -> bytes:
    pdf = Pdf()
    width = 160
    height = 240
    image = bytes(212 + ((x * 5 + y * 3) % 36) for y in range(height) for x in range(width))
    compressed = zlib.compress(image)
    content = (
        b"q 240 0 0 320 0 0 cm /Im1 Do Q "
        b"q 0.05 0.05 0.05 RG 1 w 26 34 m 214 34 l S 26 286 m 214 286 l S Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 240 320] /Rotate 90 "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width {width} /Height {height} "
            f"/ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode "
            f"/Length {len(compressed)} >>\nstream\n"
        ).encode("ascii")
        + compressed
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def mobile_cropped_photo_scan_pdf() -> bytes:
    pdf = Pdf()
    width = 260
    height = 340
    image = bytes(
        238 - min(50, abs(x - width // 2) // 3 + abs(y - height // 2) // 5)
        for y in range(height)
        for x in range(width)
    )
    compressed = zlib.compress(image)
    content = (
        b"q 260 0 0 340 0 0 cm /Im1 Do Q "
        b"q 0.12 0.12 0.12 RG 0.8 w 30 42 180 232 re S Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 260 340] "
        "/CropBox [20 30 220 290] "
        "/Resources << /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width {width} /Height {height} "
            f"/ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode "
            f"/Length {len(compressed)} >>\nstream\n"
        ).encode("ascii")
        + compressed
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def mobile_ocr_overlay_scan_pdf() -> bytes:
    pdf = Pdf()
    width = 110
    height = 150
    image = bytes([230] * width * height)
    compressed = zlib.compress(image)
    content = (
        b"q 220 0 0 300 0 0 cm /Im1 Do Q "
        b"BT /F1 12 Tf 3 Tr 32 214 Td (Invisible mobile OCR line one) Tj "
        b"0 -28 Td (Invisible mobile OCR line two) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 220 300] "
        "/Resources << /Font << /F1 5 0 R >> /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width {width} /Height {height} "
            f"/ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode "
            f"/Length {len(compressed)} >>\nstream\n"
        ).encode("ascii")
        + compressed
        + b"\nendstream"
    )
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    assert font == 5
    return pdf.render(catalog)


def mobile_mixed_compression_scan_pdf() -> bytes:
    pdf = Pdf()
    gray_width = 320
    gray_height = 220
    gray = bytes(226 + ((x + y) % 24) for y in range(gray_height) for x in range(gray_width))
    gray_compressed = zlib.compress(gray)
    jpeg = tiny_jpeg_fixture_bytes()
    content = (
        b"q 260 0 0 180 0 0 cm /Gray Do Q "
        b"q 54 0 0 54 188 96 cm /Jpeg Do Q "
        b"q 0.10 0.10 0.10 RG 0.8 w 24 24 212 132 re S Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 260 180] "
        "/Resources << /XObject << /Gray 4 0 R /Jpeg 5 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    gray_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width {gray_width} /Height {gray_height} "
            f"/ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode "
            f"/Length {len(gray_compressed)} >>\nstream\n"
        ).encode("ascii")
        + gray_compressed
        + b"\nendstream"
    )
    jpeg_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 4 /Height 4 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length "
        + str(len(jpeg)).encode("ascii")
        + b" >>\nstream\n"
        + jpeg
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert gray_object == 4
    assert jpeg_object == 5
    return pdf.render(catalog)


def scanner_skewed_mailroom_page_pdf() -> bytes:
    pdf = Pdf()
    width = 180
    height = 240
    image = bytes(
        222 - min(54, abs(x - width // 2) // 4 + (y % 23))
        for y in range(height)
        for x in range(width)
    )
    compressed = zlib.compress(image)
    content = (
        b"q 220 12 -8 300 24 18 cm /Scan Do Q "
        b"q 0.12 0.12 0.12 RG 0.7 w 48 76 m 206 92 l 198 270 l 42 252 l h S Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 260 340] "
        "/Resources << /XObject << /Scan 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width {width} /Height {height} "
            f"/ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode "
            f"/Length {len(compressed)} >>\nstream\n"
        ).encode("ascii")
        + compressed
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def scanner_large_image_budget_pdf() -> bytes:
    pdf = Pdf()
    width = 640
    height = 880
    image = bytes(214 + ((x // 16 + y // 24) % 28) for y in range(height) for x in range(width))
    compressed = zlib.compress(image)
    content = (
        b"q 320 0 0 440 0 0 cm /Scan Do Q "
        b"q 0.08 0.08 0.08 RG 1 w 38 62 244 318 re S 54 374 m 266 374 l S Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 320 440] "
        "/Resources << /XObject << /Scan 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width {width} /Height {height} "
            f"/ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode "
            f"/Length {len(compressed)} >>\nstream\n"
        ).encode("ascii")
        + compressed
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    return pdf.render(catalog)


def scanner_ocr_form_overlay_pdf() -> bytes:
    pdf = Pdf()
    width = 150
    height = 210
    image = bytes(226 - min(38, (x % 37) // 2 + (y % 29) // 3) for y in range(height) for x in range(width))
    compressed = zlib.compress(image)
    content = (
        b"q 240 0 0 336 0 0 cm /Scan Do Q "
        b"q 0 0 0 RG 0.8 w 34 72 172 204 re S 34 230 m 206 230 l 34 184 m 206 184 l S "
        b"48 140 10 10 re S 52 144 m 58 152 l S 58 152 m 68 132 l S "
        b"116 104 m 196 104 l S Q "
        b"BT /F1 10 Tf 3 Tr 42 238 Td (Invisible OCR account form) Tj "
        b"0 -46 Td (Invisible checked consent text) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 240 336] "
        "/Resources << /Font << /F1 5 0 R >> /XObject << /Scan 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width {width} /Height {height} "
            f"/ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode "
            f"/Length {len(compressed)} >>\nstream\n"
        ).encode("ascii")
        + compressed
        + b"\nendstream"
    )
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    assert font == 5
    return pdf.render(catalog)


def ocr_invisible_text_layer_pdf() -> bytes:
    return page_pdf(
        "[0 0 220 160]",
        (
            "0.94 0.94 0.9 rg 0 0 220 160 re f "
            "0.82 0.82 0.78 rg 18 28 184 18 re f "
            "0.78 0.78 0.74 rg 18 58 150 12 re f "
            "0.76 0.76 0.72 rg 18 84 176 16 re f "
            "BT /F1 12 Tf 3 Tr 20 32 Td (Invisible OCR text line one) Tj "
            "0 28 Td (Invisible OCR text line two) Tj ET"
        ),
    )


def mixed_text_image_pdf() -> bytes:
    pdf = Pdf()
    image = bytes(
        [
            235,
            245,
            255,
            235,
            245,
            255,
            180,
            210,
            245,
            180,
            210,
            245,
            180,
            210,
            245,
            235,
            245,
            255,
            235,
            245,
            255,
            180,
            210,
            245,
            180,
            210,
            245,
            235,
            245,
            255,
            235,
            245,
            255,
            180,
            210,
            245,
            180,
            210,
            245,
            235,
            245,
            255,
            235,
            245,
            255,
            180,
            210,
            245,
        ]
    )
    content = (
        b"q 0.96 0.96 0.96 rg 0 0 220 160 re f Q "
        b"q 150 0 0 84 35 36 cm /Im1 Do Q "
        b"q 0.1 0.35 0.7 RG 2 w 35 36 150 84 re S Q "
        b"q 0.9 0.2 0.1 rg 148 52 24 24 re f Q "
        b"BT /F1 18 Tf 42 116 Td (Quarterly handout) Tj "
        b"/F1 12 Tf 0 -82 Td (image + vector + text) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 220 160] "
        "/Resources << /Font << /F1 5 0 R >> /XObject << /Im1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 4 /Height 4 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    assert font == 5
    return pdf.render(catalog)


def transparency_group_pdf() -> bytes:
    pdf = Pdf()
    content = b"q 1 0 0 1 10 10 cm /Fm1 Do Q"
    form = b"1 0 0 rg 0 0 20 20 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /XObject << /Fm1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    form_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] "
        b"/Group << /S /Transparency /I true >> /Length "
        + str(len(form)).encode("ascii")
        + b" >>\nstream\n"
        + form
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert form_object == 4
    return pdf.render(catalog)


def transparency_knockout_group_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"0.5 0.5 0.5 rg 0 0 120 120 re f "
        b"q 1 0 0 1 15 15 cm /Fm1 Do Q"
    )
    form = (
        b"q /GSHalf gs 1 0 0 rg 0 0 60 60 re f Q "
        b"q /GSHalf gs 0 0 1 rg 30 30 60 60 re f Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /ExtGState << /GSHalf << /ca 0.5 >> >> "
        "/XObject << /Fm1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    form_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 90 90] "
        b"/Group << /S /Transparency /I true /K true /CS /DeviceRGB >> /Length "
        + str(len(form)).encode("ascii")
        + b" >>\nstream\n"
        + form
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert form_object == 4
    return pdf.render(catalog)


def transparency_isolated_alpha_group_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"0.5 0.5 0.5 rg 0 0 120 120 re f "
        b"q 1 0 0 1 15 15 cm /Fm1 Do Q"
    )
    form = (
        b"q /GSHalf gs 1 0 0 rg 0 0 60 60 re f Q "
        b"q /GSHalf gs 0 0 1 rg 30 30 60 60 re f Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /ExtGState << /GSHalf << /ca 0.5 >> >> "
        "/XObject << /Fm1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    form_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 90 90] "
        b"/Group << /S /Transparency /I true /CS /DeviceRGB >> /Length "
        + str(len(form)).encode("ascii")
        + b" >>\nstream\n"
        + form
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert form_object == 4
    return pdf.render(catalog)


def blend_modes_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"0.5 0.5 0.5 rg 0 0 120 120 re f "
        b"q /GS1 gs 1 0 0 rg 10 10 40 40 re f Q "
        b"q /GS2 gs 0 0 1 rg 70 10 40 40 re f Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /ExtGState << "
        "/GS1 << /BM /Multiply >> "
        "/GS2 << /BM /Screen >> "
        ">> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def blend_mode_array_fallback_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"0.5 0.5 0.5 rg 0 0 120 120 re f "
        b"q /GS1 gs 0 0 0 rg 20 20 60 60 re f Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /ExtGState << "
        "/GS1 << /BM [/UnsupportedMode /Multiply] >> "
        ">> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def unsupported_blend_mode_pdf() -> bytes:
    pdf = Pdf()
    content = b"0.5 0.5 0.5 rg 0 0 120 120 re f q /GS1 gs 1 0 0 rg 20 20 60 60 re f Q"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /ExtGState << /GS1 << /BM /Overlay >> >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def extgstate_luminosity_soft_mask_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"0.5 0.5 0.5 rg 0 0 120 120 re f "
        b"q /GSMask gs 1 0 0 rg 20 20 80 80 re f Q"
    )
    mask = b"0.9 g 0 0 80 80 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /ExtGState << "
        "/GSMask << /SMask << /S /Luminosity /G 4 0 R >> >> "
        ">> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    mask_form = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 80 80] /Length "
        + str(len(mask)).encode("ascii")
        + b" >>\nstream\n"
        + mask
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert mask_form == 4
    return pdf.render(catalog)


def transparency_alpha_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"0.5 0.5 0.5 rg 0 0 120 120 re f "
        b"q /GSFill gs 1 0 0 rg 10 10 40 40 re f Q "
        b"q /GSStroke gs 0 0 1 RG 8 w 70 10 40 40 re S Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /ExtGState << "
        "/GSFill << /ca 0.5 >> "
        "/GSStroke << /CA 0.5 >> "
        ">> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def axial_gradient_pdf() -> bytes:
    pdf = Pdf()
    content = b"/Sh1 sh"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /Shading << /Sh1 << "
        "/ShadingType 2 /ColorSpace /DeviceRGB /Coords [0 0 120 0] "
        "/Function << /FunctionType 2 /Domain [0 1] /C0 [1 0 0] /C1 [0 0 1] /N 1 >> "
        "/Extend [true true] "
        ">> >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def radial_gradient_pdf() -> bytes:
    pdf = Pdf()
    content = b"/Sh1 sh"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /Shading << /Sh1 << "
        "/ShadingType 3 /ColorSpace /DeviceRGB /Coords [60 60 0 60 60 60] "
        "/Function << /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [0 0 1] /N 1 >> "
        "/Extend [true true] "
        ">> >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def mesh_shading_unsupported_pdf() -> bytes:
    pdf = Pdf()
    content = b"/Sh1 sh"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /Shading << /Sh1 << "
        "/ShadingType 4 /ColorSpace /DeviceRGB /BitsPerCoordinate 8 "
        "/BitsPerComponent 8 /BitsPerFlag 2 "
        "/Decode [0 120 0 120 0 1 0 1 0 1] "
        "/Length 0 >> >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def type4_mesh_shading_pdf() -> bytes:
    def pack_bits(records: list[list[tuple[int, int]]]) -> bytes:
        bits = ""
        for record in records:
            for value, width in record:
                bits += format(value, f"0{width}b")
        padding = (-len(bits)) % 8
        bits += "0" * padding
        return int(bits, 2).to_bytes(len(bits) // 8, "big")

    vertices = [
        # flag, x, y, r, g, b
        [(0, 8), (0, 8), (0, 8), (255, 8), (0, 8), (0, 8)],
        [(0, 8), (255, 8), (0, 8), (0, 8), (255, 8), (0, 8)],
        [(0, 8), (0, 8), (255, 8), (0, 8), (0, 8), (255, 8)],
    ]
    mesh = pack_bits(vertices)
    pdf = Pdf()
    content = b"/Sh1 sh"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    shading = pdf.add(
        b"<< /Type /Shading /ShadingType 4 /ColorSpace /DeviceRGB /BitsPerCoordinate 8 "
        b"/BitsPerComponent 8 /BitsPerFlag 8 "
        b"/Decode [0 120 0 120 0 1 0 1 0 1] /Length "
        + str(len(mesh)).encode("ascii")
        + b" >>\nstream\n"
        + mesh
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 4 0 R /MediaBox [0 0 120 120] "
        f"/Resources << /Shading << /Sh1 {shading} 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def separation_spot_color_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q /CS1 cs 1 scn 16 18 88 42 re f "
        b"/CS1 cs 0.45 scn 16 72 88 24 re f "
        b"0 0 0 RG 1 w 16 18 88 78 re S Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /ColorSpace << /CS1 "
        "[/Separation /BrandOrange /DeviceCMYK "
        "<< /FunctionType 2 /Domain [0 1] /C0 [0 0 0 0] /C1 [0 0.65 1 0] /N 1 >>] "
        ">> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def overprint_spot_approximation_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"0.95 0.95 0.95 rg 0 0 120 120 re f "
        b"q /GSOP gs /CS1 cs 0.85 scn 16 22 88 44 re f "
        b"0 0 0 RG 2 w 16 22 88 44 re S Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << "
        "/ExtGState << /GSOP << /OP true /op true /OPM 1 >> >> "
        "/ColorSpace << /CS1 "
        "[/Separation /OverprintOrange /DeviceRGB "
        "<< /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [1 0.4 0.15] /N 1 >>] "
        ">> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def devicen_spot_color_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q /CS2 cs 0.25 1 scn 14 18 92 32 re f "
        b"/CS2 CS 1 0.35 SCN 6 w 18 76 m 102 76 l S "
        b"/CS2 cs 0.8 0.2 scn 24 88 72 16 re f Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /ColorSpace << /CS2 "
        "[/DeviceN [/SpotOrange /SpotBlue] /DeviceRGB "
        "<< /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [0.2 0.4 0.9] /N 1 >>] "
        ">> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    return pdf.render(catalog)


def tiling_pattern_pdf() -> bytes:
    pdf = Pdf()
    content = b"/Pattern cs /P1 scn 0 0 120 120 re f"
    pattern = b"1 0 0 rg 0 0 10 20 re f 0 0 1 rg 10 0 10 20 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /Pattern << /P1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    pattern_object = pdf.add(
        b"<< /Type /Pattern /PatternType 1 /PaintType 1 /TilingType 1 "
        b"/BBox [0 0 20 20] /XStep 20 /YStep 20 /Length "
        + str(len(pattern)).encode("ascii")
        + b" >>\nstream\n"
        + pattern
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert pattern_object == 4
    return pdf.render(catalog)


def uncolored_tiling_pattern_pdf() -> bytes:
    pdf = Pdf()
    content = b"/CS1 cs 0.2 0.7 0.3 /P1 scn 0 0 120 120 re f"
    pattern = b"0 0 12 12 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        "/Resources << /ColorSpace << /CS1 [/Pattern /DeviceRGB] >> "
        "/Pattern << /P1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    pattern_object = pdf.add(
        b"<< /Type /Pattern /PatternType 1 /PaintType 2 /TilingType 1 "
        b"/BBox [0 0 12 12] /XStep 24 /YStep 24 /Length "
        + str(len(pattern)).encode("ascii")
        + b" >>\nstream\n"
        + pattern
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert pattern_object == 4
    return pdf.render(catalog)


def dashed_stroke_pdf() -> bytes:
    return page_pdf(
        "[0 0 120 120]",
        "0 0 0 RG 4 w [10 10] 0 d 10 60 m 110 60 l S",
    )


def line_caps_pdf() -> bytes:
    return page_pdf(
        "[0 0 120 120]",
        "0 0 0 RG 4 w "
        "0 J 20 90 m 100 90 l S "
        "1 J 20 60 m 100 60 l S "
        "2 J 20 30 m 100 30 l S",
    )


def line_joins_pdf() -> bytes:
    return page_pdf(
        "[0 0 120 120]",
        "0 0 0 RG 8 w "
        "2 j 20 30 m 50 30 l 50 60 l S "
        "1 j 20 75 m 50 75 l 50 105 l S "
        "0 j 80 30 m 110 30 l 110 60 l S",
    )


def clipped_paths_pdf() -> bytes:
    return page_pdf(
        "[0 0 120 120]",
        "4 4 112 112 re 40 40 40 40 re W* n "
        "0 0 0 rg 0 0 120 120 re f",
    )


def vector_stress_pdf() -> bytes:
    ops: list[str] = [
        "q",
        "8 8 144 104 re W n",
        "16 16 128 88 re W n",
        "0.96 0.96 0.96 rg 0 0 160 120 re f",
        "0.82 0.82 0.82 RG 0.5 w",
    ]
    for x in range(20, 145, 10):
        ops.append(f"{x} 18 m {x} 104 l S")
    for y in range(20, 105, 10):
        ops.append(f"18 {y} m 144 {y} l S")
    bar_colors = [
        (0.1, 0.45, 0.82),
        (0.08, 0.62, 0.42),
        (0.88, 0.42, 0.1),
        (0.58, 0.28, 0.72),
    ]
    for index, height in enumerate([24, 52, 36, 70, 44, 62, 30, 78, 48, 58]):
        x = 22 + index * 12
        r, g, b = bar_colors[index % len(bar_colors)]
        ops.append(f"{r} {g} {b} rg {x} 20 8 {height} re f")
        ops.append(f"0.12 0.12 0.12 RG 0.8 w {x} 20 8 {height} re S")
    ops.extend(
        [
            "0.95 0.12 0.18 RG 2 w 1 J 1 j",
            "20 52 m 32 92 44 12 56 52 c 68 92 80 12 92 52 c "
            "104 92 116 12 128 52 c 136 78 140 36 144 62 c S",
            "0.05 0.05 0.05 rg",
        ]
    )
    for x, y in [(30, 86), (54, 42), (78, 68), (102, 34), (126, 76)]:
        ops.append(f"{x} {y} 4 4 re f")
    ops.append("Q")
    return page_pdf("[0 0 160 120]", " ".join(ops))


def annotation_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    appearance = b"0 0 0 rg 0 0 20 10 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        f"/Contents {contents} 0 R /Annots [5 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 10] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Stamp /Rect [20 80 60 100] /AS /On "
        f"/AP << /N << /On {appearance_object} 0 R >> >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert annotation == 5
    return pdf.render(catalog)


def annotation_missing_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b"0 0 0 rg 10 10 20 20 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        f"/Contents {contents} 0 R /Annots [4 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    annotation = pdf.add("<< /Type /Annot /Subtype /Stamp /Rect [60 60 90 90] >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert annotation == 4
    return pdf.render(catalog)


def link_annotation_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    appearance = b"0 0 1 RG 3 w 1.5 1.5 37 17 re S"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        f"/Contents {contents} 0 R /Annots [5 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 40 20] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Link /Rect [70 20 110 40] "
        "/Border [0 0 0] /A << /S /URI /URI (https://example.invalid/) >> "
        f"/AP << /N {appearance_object} 0 R >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert annotation == 5
    return pdf.render(catalog)


def highlight_annotation_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    appearance = b"1 1 0 rg 0 0 50 12 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        f"/Contents {contents} 0 R /Annots [5 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 50 12] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Highlight /Rect [20 60 70 72] "
        f"/QuadPoints [20 72 70 72 20 60 70 60] /AP << /N {appearance_object} 0 R >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert annotation == 5
    return pdf.render(catalog)


def highlight_annotation_without_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b"0 0 0 rg 10 10 20 20 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        f"/Contents {contents} 0 R /Annots [4 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Highlight /Rect [20 60 100 75] "
        "/QuadPoints [20 75 100 75 20 60 100 60] /C [1 1 0] >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert annotation == 4
    return pdf.render(catalog)


def markup_annotations_without_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        f"/Contents {contents} 0 R /Annots [4 0 R 5 0 R 6 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    underline = pdf.add(
        "<< /Type /Annot /Subtype /Underline /Rect [15 80 65 92] "
        "/QuadPoints [15 92 65 92 15 80 65 80] /C [1 0 0] >>"
    )
    square = pdf.add(
        "<< /Type /Annot /Subtype /Square /Rect [15 20 45 50] /C [0 0.45 1] >>"
    )
    circle = pdf.add(
        "<< /Type /Annot /Subtype /Circle /Rect [70 20 110 60] /C [0 0.55 0] >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert underline == 4
    assert square == 5
    assert circle == 6
    return pdf.render(catalog)


def link_annotation_without_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b"0 0 0 rg 10 10 20 20 re f"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        f"/Contents {contents} 0 R /Annots [4 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Link /Rect [60 60 100 80] "
        "/Border [0 0 0] /A << /S /URI /URI (https://example.invalid/) >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert annotation == 4
    return pdf.render(catalog)


def text_note_annotation_without_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        f"/Contents {contents} 0 R /Annots [4 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Text /Rect [80 80 102 102] "
        "/C [1 1 0] /Contents (Review note) >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert annotation == 4
    return pdf.render(catalog)


def widget_annotation_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    appearance = b"0.9 0.9 0.9 rg 0 0 50 18 re f 0 0 0 RG 1 w 0.5 0.5 49 17 re S"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 120] "
        f"/Contents {contents} 0 R /Annots [5 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 50 18] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Tx /Rect [20 25 70 43] "
        f"/AP << /N {appearance_object} 0 R >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert annotation == 5
    return pdf.render(catalog)


def acroform_text_field_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    appearance = b"0.85 0.92 1 rg 0 0 60 20 re f 0 0 0 RG 1 w 0.5 0.5 59 19 re S"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 140 80] "
        f"/Contents {contents} 0 R /Annots [5 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 60 20] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Tx /T (Name) /V (Ada) "
        "/Rect [30 30 90 50] "
        f"/AP << /N {appearance_object} 0 R >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{field} 0 R] >> >>")
    assert field == 5
    return pdf.render(catalog)


def xfa_static_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 1 0 0 1 30 30 cm "
        b"0.85 0.92 1 rg 0 0 60 20 re f 0 0 0 RG 1 w 0.5 0.5 59 19 re S "
        b"Q"
    )
    appearance = b"0.85 0.92 1 rg 0 0 60 20 re f 0 0 0 RG 1 w 0.5 0.5 59 19 re S"
    xfa_packet = (
        b"<xdp:xdp xmlns:xdp=\"http://ns.adobe.com/xdp/\">"
        b"<template><subform name=\"static\" layout=\"tb\"/></template>"
        b"</xdp:xdp>"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 140 80] "
        f"/Contents {contents} 0 R /Annots [5 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 60 20] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Tx /T (Name) /V (Ada) "
        "/Rect [30 30 90 50] "
        f"/AP << /N {appearance_object} 0 R >> >>"
    )
    xfa = pdf.add(
        f"<< /Length {len(xfa_packet)} >>\nstream\n".encode("ascii")
        + xfa_packet
        + b"\nendstream"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R "
        f"/AcroForm << /Fields [{field} 0 R] /XFA {xfa} 0 R >> >>"
    )
    assert field == 5
    return pdf.render(catalog)


def xfa_dynamic_no_static_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    xfa_packet = (
        b"<xdp:xdp xmlns:xdp=\"http://ns.adobe.com/xdp/\">"
        b"<template><subform name=\"dynamic\" layout=\"flowed\"/></template>"
        b"<datasets><data><value>runtime-only</value></data></datasets>"
        b"</xdp:xdp>"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 140 80] "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    xfa = pdf.add(
        f"<< /Length {len(xfa_packet)} >>\nstream\n".encode("ascii")
        + xfa_packet
        + b"\nendstream"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /XFA {xfa} 0 R >> >>"
    )
    return pdf.render(catalog)


def acroform_text_field_missing_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 140 80] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R /Annots [5 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Tx /T (Name) /V (Ada) "
        "/Rect [30 30 100 52] >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{field} 0 R] >> >>")
    assert font == 4
    assert field == 5
    return pdf.render(catalog)


def acroform_checkbox_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    yes_appearance = (
        b"1 1 1 rg 0 0 20 20 re f "
        b"0 0 0 RG 1 w 0.5 0.5 19 19 re S "
        b"0 0 0 rg 6 6 8 8 re f"
    )
    off_appearance = b"1 1 1 rg 0 0 20 20 re f 0 0 0 RG 1 w 0.5 0.5 19 19 re S"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 80 80] "
        f"/Contents {contents} 0 R /Annots [6 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    yes_appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Length "
        + str(len(yes_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + yes_appearance
        + b"\nendstream"
    )
    off_appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Length "
        + str(len(off_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + off_appearance
        + b"\nendstream"
    )
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Btn /T (Agree) /V /Yes /AS /Yes "
        "/Rect [20 30 40 50] "
        f"/AP << /N << /Yes {yes_appearance_object} 0 R /Off {off_appearance_object} 0 R >> >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{field} 0 R] >> >>")
    assert field == 6
    return pdf.render(catalog)


def acroform_checkbox_missing_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 80 80] "
        f"/Contents {contents} 0 R /Annots [4 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Btn /T (Agree) /V /Yes /AS /Yes "
        "/Rect [20 30 40 50] >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{field} 0 R] >> >>")
    assert field == 4
    return pdf.render(catalog)


def acroform_radio_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    selected_appearance = (
        b"1 1 1 rg 0 0 20 20 re f "
        b"0 0 0 RG 1 w 0.5 0.5 19 19 re S "
        b"0 0 0 rg 7 7 6 6 re f"
    )
    off_appearance = b"1 1 1 rg 0 0 20 20 re f 0 0 0 RG 1 w 0.5 0.5 19 19 re S"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 100 80] "
        f"/Contents {contents} 0 R /Annots [6 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    selected_appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Length "
        + str(len(selected_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + selected_appearance
        + b"\nendstream"
    )
    off_appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Length "
        + str(len(off_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + off_appearance
        + b"\nendstream"
    )
    selected_field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Btn /Ff 32768 /T (ChoiceA) /V /A /AS /A "
        "/Rect [20 42 40 62] "
        f"/AP << /N << /A {selected_appearance_object} 0 R /Off {off_appearance_object} 0 R >> >> >>"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{selected_field} 0 R] >> >>"
    )
    assert selected_field == 6
    return pdf.render(catalog)


def acroform_radio_missing_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 100 80] "
        f"/Contents {contents} 0 R /Annots [4 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Btn /Ff 32768 /T (ChoiceA) "
        "/V /A /AS /A /Rect [20 30 40 50] >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{field} 0 R] >> >>")
    assert field == 4
    return pdf.render(catalog)


def acroform_radio_off_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    selected_appearance = (
        b"1 1 1 rg 0 0 20 20 re f "
        b"0 0 0 RG 1 w 0.5 0.5 19 19 re S "
        b"0 0 0 rg 7 7 6 6 re f"
    )
    off_appearance = b"1 1 1 rg 0 0 20 20 re f 0 0 0 RG 1 w 0.5 0.5 19 19 re S"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 100 80] "
        f"/Contents {contents} 0 R /Annots [6 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    selected_appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Length "
        + str(len(selected_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + selected_appearance
        + b"\nendstream"
    )
    off_appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Length "
        + str(len(off_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + off_appearance
        + b"\nendstream"
    )
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Btn /Ff 32768 /T (ChoiceOff) /V /A /AS /Off "
        "/Rect [20 42 40 62] "
        f"/AP << /N << /A {selected_appearance_object} 0 R /Off {off_appearance_object} 0 R >> >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{field} 0 R] >> >>")
    assert field == 6
    return pdf.render(catalog)


def acroform_choice_missing_appearance_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 140 80] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R /Annots [5 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Ch /T (Plan) /V (Basic) "
        "/Rect [30 30 110 52] >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{field} 0 R] >> >>")
    assert font == 4
    assert field == 5
    return pdf.render(catalog)


def acroform_signature_placeholder_pdf() -> bytes:
    pdf = Pdf()
    content = b""
    appearance = (
        b"0.94 0.94 0.94 rg 0 0 100 30 re f "
        b"0 0 0 RG 1 w 0.5 0.5 99 29 re S "
        b"0.25 0.25 0.25 RG 2 w 8 8 m 92 22 l S"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 90] "
        f"/Contents {contents} 0 R /Annots [5 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 100 30] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Sig /T (Signature) "
        "/Rect [20 35 120 65] "
        f"/AP << /N {appearance_object} 0 R >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{field} 0 R] >> >>")
    assert field == 5
    return pdf.render(catalog)


def digital_signature_appearance_pdf() -> bytes:
    pdf = Pdf()
    appearance = (
        b"0.94 0.94 0.94 rg 0 0 100 30 re f "
        b"0 0 0 RG 1 w 0.5 0.5 99 29 re S "
        b"0.25 0.25 0.25 RG 2 w 8 8 m 92 22 l S"
    )
    content = b"q 1 0 0 1 20 35 cm " + appearance + b" Q"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 90] "
        f"/Contents {contents} 0 R /Annots [6 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 100 30] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    signature = pdf.add(
        "<< /Type /Sig /Filter /Adobe.PPKLite /SubFilter /adbe.pkcs7.detached "
        "/ByteRange [0 0 0 0] /Contents <00> /M (D:20260625120000Z) >>"
    )
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Sig /T (SignedByExample) "
        f"/V {signature} 0 R /Rect [20 35 120 65] "
        f"/AP << /N {appearance_object} 0 R >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{field} 0 R] >> >>")
    assert signature == 5
    assert field == 6
    return pdf.render(catalog)


def e_signature_contract_workflow_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 1 1 1 rg 0 0 360 260 re f Q "
        b"q 0.11 0.18 0.28 rg 0 224 360 36 re f Q "
        b"q 0.18 0.22 0.30 RG 0.7 w 24 46 312 150 re S "
        b"24 166 m 336 166 l S 24 136 m 336 136 l S 24 106 m 336 106 l S "
        b"24 76 m 336 76 l S 126 46 m 126 196 l S 232 46 m 232 196 l S Q "
        b"q 0 0 0 RG 0.8 w 42 52 m 116 52 l 146 52 m 216 52 l 252 52 m 318 52 l S Q "
        b"BT /F1 12 Tf 24 238 Td (Synthetic Services Agreement) Tj "
        b"/F1 8 Tf 30 180 Td (Signer) Tj 104 0 Td (Initials) Tj 106 0 Td (Date) Tj "
        b"-210 -30 Td (Example Signer) Tj 104 0 Td (ES) Tj 106 0 Td (2026-06-26) Tj "
        b"-210 -30 Td (Workflow ID) Tj 104 0 Td (ESIGN-0153) Tj 106 0 Td (Complete) Tj "
        b"-210 -30 Td (Consent) Tj 104 0 Td (Accepted) Tj 106 0 Td (Audit attached) Tj "
        b"-210 -82 Td (Signer initials) Tj 104 0 Td (Date field) Tj 106 0 Td (Signature field) Tj ET"
    )
    signature_appearance = (
        b"0.94 0.94 0.94 rg 0 0 82 24 re f "
        b"0 0 0 RG 0.8 w 0.5 0.5 81 23 re S "
        b"0.10 0.10 0.10 RG 1.6 w 8 8 m 22 16 l 38 6 l 56 18 l 74 11 l S"
    )
    stamp_appearance = (
        b"0.78 0.04 0.04 RG 1.2 w 0.5 0.5 58 20 re S "
        b"0.78 0.04 0.04 RG 0.8 w 6 10 m 52 10 l S"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 360 260] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R /Annots [7 0 R 9 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    signature = pdf.add(
        "<< /Type /Sig /Filter /Adobe.PPKLite /SubFilter /adbe.pkcs7.detached "
        "/ByteRange [0 0 0 0] /Contents <00> /M (D:20260626100000Z) >>"
    )
    signature_appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 82 24] /Length "
        + str(len(signature_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + signature_appearance
        + b"\nendstream"
    )
    signature_field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Sig /T (SignerSignature) "
        f"/V {signature} 0 R /Rect [252 40 334 64] "
        f"/AP << /N {signature_appearance_object} 0 R >> >>"
    )
    stamp_appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 58 20] /Length "
        + str(len(stamp_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + stamp_appearance
        + b"\nendstream"
    )
    stamp = pdf.add(
        "<< /Type /Annot /Subtype /Stamp /Name /Approved /Rect [268 174 326 194] "
        f"/AP << /N {stamp_appearance_object} 0 R >> >>"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{signature_field} 0 R] >> >>"
    )
    assert font == 4
    assert signature == 5
    assert signature_field == 7
    assert stamp == 9
    return pdf.render(catalog)


def e_signature_audit_certificate_pdf() -> bytes:
    return page_pdf(
        "[0 0 420 300]",
        (
            "q 0.98 0.99 1 rg 0 0 420 300 re f Q "
            "q 0.10 0.16 0.25 rg 0 258 420 42 re f Q "
            "q 0.18 0.22 0.28 RG 0.7 w 26 44 232 174 re S "
            "26 190 m 258 190 l S 26 162 m 258 162 l S 26 134 m 258 134 l S "
            "26 106 m 258 106 l S 118 44 m 118 218 l S 190 44 m 190 218 l S Q "
            "q 0.76 0.05 0.04 RG 1.6 w 286 176 84 42 re S 294 184 68 26 re S Q "
            "q 0 0 0 rg 298 58 4 4 re f 310 58 4 4 re f 322 58 4 4 re f "
            "298 70 4 4 re f 322 70 4 4 re f 334 70 4 4 re f "
            "298 82 4 4 re f 310 82 4 4 re f 334 82 4 4 re f Q "
            "BT /F1 13 Tf 24 274 Td (E-Signature Completion Certificate) Tj "
            "/F1 8 Tf 34 202 Td (Event) Tj 92 0 Td (Actor) Tj 72 0 Td (Time UTC) Tj "
            "-164 -28 Td (Viewed) Tj 92 0 Td (Signer) Tj 72 0 Td (10:01) Tj "
            "-164 -28 Td (Consented) Tj 92 0 Td (Signer) Tj 72 0 Td (10:02) Tj "
            "-164 -28 Td (Signed) Tj 92 0 Td (Signer) Tj 72 0 Td (10:04) Tj "
            "-164 -28 Td (Completed) Tj 92 0 Td (System) Tj 72 0 Td (10:05) Tj "
            "170 62 Td (COMPLETED) Tj "
            "-2 -118 Td (QR-style audit marker) Tj "
            "-262 -202 Td (Presence-only audit certificate fixture; no cryptographic validation.) Tj ET"
        ),
    )


def e_signature_incremental_revision_pdf() -> bytes:
    pdf = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
    offsets: dict[int, int] = {}

    def add_object(number: int, body: bytes) -> int:
        offset = len(pdf)
        offsets[number] = offset
        pdf.extend(f"{number} 0 obj\n".encode("ascii"))
        pdf.extend(body)
        pdf.extend(b"\nendobj\n")
        return offset

    base_content = (
        b"q 1 1 1 rg 0 0 300 180 re f Q "
        b"q 0.18 0.22 0.30 RG 0.8 w 24 42 252 94 re S Q "
        b"BT /F1 11 Tf 32 144 Td (Draft Agreement) Tj "
        b"/F1 8 Tf 32 116 Td (Status: pending signature) Tj "
        b"0 -28 Td (Revision 0) Tj ET"
    )
    add_object(1, b"<< /Type /Catalog /Pages 2 0 R >>")
    add_object(2, b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
    add_object(
        3,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 180] "
        b"/Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>",
    )
    add_object(4, b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    add_object(
        5,
        f"<< /Length {len(base_content)} >>\nstream\n".encode("ascii")
        + base_content
        + b"\nendstream",
    )
    first_xref = len(pdf)
    pdf.extend(b"xref\n0 6\n0000000000 65535 f \n")
    for number in range(1, 6):
        pdf.extend(f"{offsets[number]:010d} 00000 n \n".encode("ascii"))
    pdf.extend(
        f"trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{first_xref}\n%%EOF\n".encode(
            "ascii"
        )
    )

    updated_content = (
        b"q 1 1 1 rg 0 0 300 180 re f Q "
        b"q 0.18 0.22 0.30 RG 0.8 w 24 42 252 94 re S "
        b"24 106 m 276 106 l S Q "
        b"q 0.78 0.04 0.04 RG 1.2 w 214 118 44 18 re S Q "
        b"BT /F1 11 Tf 32 144 Td (Signed Agreement) Tj "
        b"/F1 8 Tf 32 118 Td (Status: completed by incremental update) Tj "
        b"0 -28 Td (Revision 1 signed at 10:06 UTC) Tj "
        b"0 -48 Td (Signature appearance) Tj ET"
    )
    signature_appearance = (
        b"0.94 0.94 0.94 rg 0 0 82 24 re f "
        b"0 0 0 RG 0.8 w 0.5 0.5 81 23 re S "
        b"0.10 0.10 0.10 RG 1.6 w 8 8 m 22 16 l 38 6 l 56 18 l 74 11 l S"
    )
    updated_page = add_object(
        3,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 180] "
        b"/Resources << /Font << /F1 4 0 R >> >> /Contents 8 0 R /Annots [9 0 R] >>",
    )
    updated_catalog = add_object(
        1,
        b"<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [9 0 R] >> >>",
    )
    signature = add_object(
        6,
        b"<< /Type /Sig /Filter /Adobe.PPKLite /SubFilter /adbe.pkcs7.detached "
        b"/ByteRange [0 0 0 0] /Contents <00> /M (D:20260626100600Z) >>",
    )
    appearance = add_object(
        7,
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 82 24] /Length "
        + str(len(signature_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + signature_appearance
        + b"\nendstream",
    )
    stream = add_object(
        8,
        f"<< /Length {len(updated_content)} >>\nstream\n".encode("ascii")
        + updated_content
        + b"\nendstream",
    )
    field = add_object(
        9,
        b"<< /Type /Annot /Subtype /Widget /FT /Sig /T (IncrementalSignature) "
        b"/V 6 0 R /Rect [190 44 272 68] /AP << /N 7 0 R >> >>",
    )
    second_xref = len(pdf)
    pdf.extend(
        (
            "xref\n1 1\n"
            f"{updated_catalog:010d} 00000 n \n"
            "3 1\n"
            f"{updated_page:010d} 00000 n \n"
            "6 4\n"
            f"{signature:010d} 00000 n \n"
            f"{appearance:010d} 00000 n \n"
            f"{stream:010d} 00000 n \n"
            f"{field:010d} 00000 n \n"
            f"trailer\n<< /Size 10 /Root 1 0 R /Prev {first_xref} >>\n"
            f"startxref\n{second_xref}\n%%EOF\n"
        ).encode("ascii")
    )
    return bytes(pdf)


def embedded_source_file_pdf() -> bytes:
    pdf = Pdf()
    content = b"0.92 0.96 1 rg 12 28 136 24 re f 0 0 0 RG 1 w 12 28 136 24 re S"
    embedded = b"fn main() { println!(\"attached source\"); }\n"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 90] "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    embedded_file = pdf.add(
        f"<< /Type /EmbeddedFile /Subtype /text#2Fplain /Length {len(embedded)} >>\nstream\n".encode("ascii")
        + embedded
        + b"endstream"
    )
    filespec = pdf.add(
        f"<< /Type /Filespec /F (main.rs) /UF (main.rs) /EF << /F {embedded_file} 0 R >> >>"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /Names << /EmbeddedFiles << /Names [(main.rs) {filespec} 0 R] >> >> >>"
    )
    return pdf.render(catalog)


def portfolio_embedded_files_pdf() -> bytes:
    pdf = Pdf()
    content = b"0.96 0.96 0.96 rg 0 0 160 90 re f 0.15 0.2 0.35 rg 20 32 120 26 re f"
    embedded = b"Portfolio attachment payload\n"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 90] "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    embedded_file = pdf.add(
        f"<< /Type /EmbeddedFile /Length {len(embedded)} >>\nstream\n".encode("ascii")
        + embedded
        + b"endstream"
    )
    filespec = pdf.add(
        f"<< /Type /Filespec /F (portfolio.txt) /UF (portfolio.txt) /EF << /F {embedded_file} 0 R >> >>"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R "
        f"/Names << /EmbeddedFiles << /Names [(portfolio.txt) {filespec} 0 R] >> >> "
        "/Collection << /Type /Collection /View /D >> >>"
    )
    return pdf.render(catalog)


def file_attachment_annotation_pdf() -> bytes:
    pdf = Pdf()
    attachment = b"attachment bytes\n"
    appearance = (
        b"1 0.92 0.45 rg 0 0 18 18 re f "
        b"0.15 0.15 0.15 RG 1 w 0.5 0.5 17 17 re S "
        b"0.15 0.15 0.15 rg 5 4 8 10 re f"
    )
    content = b"q 1 0 0 1 24 36 cm " + appearance + b" Q"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 90] "
        f"/Contents {contents} 0 R /Annots [7 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 18 18] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    embedded_file = pdf.add(
        f"<< /Type /EmbeddedFile /Length {len(attachment)} >>\nstream\n".encode("ascii")
        + attachment
        + b"endstream"
    )
    filespec = pdf.add(
        f"<< /Type /Filespec /F (note.txt) /UF (note.txt) /EF << /F {embedded_file} 0 R >> >>"
    )
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /FileAttachment /Name /PushPin "
        f"/Rect [24 36 42 54] /FS {filespec} 0 R /AP << /N {appearance_object} 0 R >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert appearance_object == 4
    assert embedded_file == 5
    assert filespec == 6
    assert annotation == 7
    return pdf.render(catalog)


def linearized_first_page_pdf(malformed_hint: bool = False) -> bytes:
    first_content = b"0.87 0.94 1 rg 0 0 160 90 re f 0.1 0.25 0.45 rg 16 28 80 28 re f"
    second_content = b"0.95 0.9 0.82 rg 0 0 160 90 re f 0.45 0.1 0.1 rg 60 22 72 42 re f"
    objects: list[bytes] = [
        b"<< /Linearized 1 /L 0000000000 /H [0 0] /O 4 /E 0000000000 /N 2 /T 0000000000 >>",
        b"<< /Type /Catalog /Pages 3 0 R >>",
        b"<< /Type /Pages /Kids [4 0 R 6 0 R] /Count 2 >>",
        b"<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 90] /Contents 5 0 R >>",
        f"<< /Length {len(first_content)} >>\nstream\n".encode("ascii")
        + first_content
        + b"\nendstream",
        b"<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 90] /Contents 7 0 R >>",
        f"<< /Length {len(second_content)} >>\nstream\n".encode("ascii")
        + second_content
        + b"\nendstream",
    ]
    out = bytearray(b"%PDF-1.5\n%\xe2\xe3\xcf\xd3\n")
    offsets = [0]
    first_page_end = 0
    for idx, body in enumerate(objects, start=1):
        if idx == 6:
            first_page_end = len(out)
        offsets.append(len(out))
        out.extend(f"{idx} 0 obj\n".encode("ascii"))
        out.extend(body)
        out.extend(b"\nendobj\n")

    xref_offset = len(out)
    out.extend(f"xref\n0 {len(objects) + 1}\n".encode("ascii"))
    out.extend(b"0000000000 65535 f \n")
    for offset in offsets[1:]:
        out.extend(f"{offset:010d} 00000 n \n".encode("ascii"))
    out.extend(
        f"trailer\n<< /Size {len(objects) + 1} /Root 2 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n".encode(
            "ascii"
        )
    )

    first_page_end_value = 12 if malformed_hint else first_page_end
    replacements = {
        b"/L 0000000000": f"/L {len(out):010d}".encode("ascii"),
        b"/E 0000000000": f"/E {first_page_end_value:010d}".encode("ascii"),
        b"/T 0000000000": f"/T {xref_offset:010d}".encode("ascii"),
    }
    result = bytes(out)
    for placeholder, value in replacements.items():
        result = result.replace(placeholder, value, 1)
    return result


def optional_content_layer_pdf(visible: bool) -> bytes:
    pdf = Pdf()
    content = (
        b"0 0.6 0 rg 10 10 40 40 re f "
        b"/OC /Layer BDC "
        b"0.9 0 0 rg 60 10 40 40 re f "
        b"EMC"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 120 80] "
        "/Resources << /Properties << /Layer 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    layer = pdf.add("<< /Type /OCG /Name (Fixture Layer) >>")
    off = "" if visible else "/OFF [4 0 R] "
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R "
        f"/OCProperties << /OCGs [{layer} 0 R] /D << /BaseState /ON {off}>> >> >>"
    )
    assert layer == 4
    return pdf.render(catalog)


def optional_content_ocmd_pdf() -> bytes:
    pdf = Pdf()
    content = b"/OC /Policy BDC 0.9 0 0 rg 20 20 40 40 re f EMC"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 100 80] "
        "/Resources << /Properties << /Policy 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    policy = pdf.add("<< /Type /OCMD /OCGs [5 0 R] /P /AllOn >>")
    group = pdf.add("<< /Type /OCG /Name (Unsupported Policy Layer) >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R "
        f"/OCProperties << /OCGs [{group} 0 R] /D << /BaseState /ON >> >> >>"
    )
    assert policy == 4
    return pdf.render(catalog)


def incremental_update_pdf() -> bytes:
    pdf = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
    offsets: list[int] = [0]

    def add_object(number: int, body: bytes) -> int:
        offset = len(pdf)
        offsets.append(offset)
        pdf.extend(f"{number} 0 obj\n".encode("ascii"))
        pdf.extend(body)
        pdf.extend(b"\nendobj\n")
        return offset

    base_content = b"0 0.6 0 rg 10 10 40 40 re f"
    updated_content = b"0.9 0 0 rg 10 10 40 40 re f"
    add_object(1, b"<< /Type /Catalog /Pages 2 0 R >>")
    add_object(2, b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
    add_object(
        3,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 120 80] /Contents 4 0 R >>",
    )
    add_object(
        4,
        f"<< /Length {len(base_content)} >>\nstream\n".encode("ascii")
        + base_content
        + b"\nendstream",
    )
    first_xref = len(pdf)
    pdf.extend(b"xref\n0 5\n0000000000 65535 f \n")
    for offset in offsets[1:]:
        pdf.extend(f"{offset:010d} 00000 n \n".encode("ascii"))
    pdf.extend(
        f"trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{first_xref}\n%%EOF\n".encode(
            "ascii"
        )
    )

    updated_page = add_object(
        3,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 120 80] /Contents 5 0 R >>",
    )
    updated_stream = add_object(
        5,
        f"<< /Length {len(updated_content)} >>\nstream\n".encode("ascii")
        + updated_content
        + b"\nendstream",
    )
    second_xref = len(pdf)
    pdf.extend(
        (
            "xref\n3 1\n"
            f"{updated_page:010d} 00000 n \n"
            "5 1\n"
            f"{updated_stream:010d} 00000 n \n"
            f"trailer\n<< /Size 6 /Root 1 0 R /Prev {first_xref} >>\n"
            f"startxref\n{second_xref}\n%%EOF\n"
        ).encode("ascii")
    )
    return bytes(pdf)


def incremental_deleted_object_pdf() -> bytes:
    pdf = bytearray(b"%PDF-1.7\n")
    offsets: dict[int, int] = {}

    def obj(number: int, body: bytes) -> None:
        offsets[number] = len(pdf)
        pdf.extend(f"{number} 0 obj\n".encode("ascii"))
        pdf.extend(body)
        pdf.extend(b"\nendobj\n")

    obj(1, b"<< /Type /Catalog /Pages 2 0 R >>")
    obj(2, b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
    obj(
        3,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 120 80] "
        b"/Contents 5 0 R /UnusedDeleted 4 0 R >>",
    )
    obj(4, b"<< /Deleted true >>")
    content = b"0 0.5 0 rg 20 20 80 40 re f"
    obj(
        5,
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream",
    )

    first_xref = len(pdf)
    pdf.extend(b"xref\n0 6\n0000000000 65535 f \n")
    for number in range(1, 6):
        pdf.extend(f"{offsets[number]:010} 00000 n \n".encode("ascii"))
    pdf.extend(
        f"trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{first_xref}\n%%EOF\n".encode(
            "ascii"
        )
    )

    second_xref = len(pdf)
    pdf.extend(
        (
            "xref\n4 1\n"
            "0000000000 00001 f \n"
            f"trailer\n<< /Size 6 /Root 1 0 R /Prev {first_xref} >>\n"
            f"startxref\n{second_xref}\n%%EOF\n"
        ).encode("ascii")
    )
    return bytes(pdf)


def hybrid_reference_pdf() -> bytes:
    pdf = bytearray(b"%PDF-1.5\n%\xe2\xe3\xcf\xd3\n")

    def add_object(number: int, body: bytes) -> int:
        offset = len(pdf)
        pdf.extend(f"{number} 0 obj\n".encode("ascii"))
        pdf.extend(body)
        pdf.extend(b"\nendobj\n")
        return offset

    content = b"0 0 0.9 rg 10 10 40 40 re f"
    object_1 = add_object(1, b"<< /Type /Catalog /Pages 2 0 R >>")
    object_2 = add_object(2, b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
    object_3 = add_object(
        3,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 120 80] /Contents 4 0 R >>",
    )
    object_4 = add_object(
        4,
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream",
    )
    xref_stream_offset = len(pdf)
    xref_data = bytearray()
    _push_xref_stream_entry(xref_data, 1, object_4, 0)
    compressed_xref = zlib.compress(bytes(xref_data))
    pdf.extend(
        f"5 0 obj\n<< /Type /XRef /Size 5 /W [1 4 2] /Index [4 1] /Length {len(compressed_xref)} /Filter /FlateDecode >>\nstream\n".encode(
            "ascii"
        )
    )
    pdf.extend(compressed_xref)
    pdf.extend(b"\nendstream\nendobj\n")
    classic_xref = len(pdf)
    pdf.extend(
        (
            "xref\n0 4\n"
            "0000000000 65535 f \n"
            f"{object_1:010d} 00000 n \n"
            f"{object_2:010d} 00000 n \n"
            f"{object_3:010d} 00000 n \n"
            f"trailer\n<< /Size 5 /Root 1 0 R /XRefStm {xref_stream_offset} >>\n"
            f"startxref\n{classic_xref}\n%%EOF\n"
        ).encode("ascii")
    )
    return bytes(pdf)


def encrypted_placeholder_pdf() -> bytes:
    pdf = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
    offsets: list[int] = [0]

    def add_object(number: int, body: bytes) -> None:
        offsets.append(len(pdf))
        pdf.extend(f"{number} 0 obj\n".encode("ascii"))
        pdf.extend(body)
        pdf.extend(b"\nendobj\n")

    content = b"0.9 0 0 rg 10 10 40 40 re f"
    add_object(1, b"<< /Type /Catalog /Pages 2 0 R >>")
    add_object(2, b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
    add_object(
        3,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 120 80] /Contents 4 0 R >>",
    )
    add_object(
        4,
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream",
    )
    add_object(5, b"<< /Filter /Standard /V 1 /R 2 /P -4 >>")
    xref_offset = len(pdf)
    pdf.extend(f"xref\n0 {len(offsets)}\n".encode("ascii"))
    pdf.extend(b"0000000000 65535 f \n")
    for offset in offsets[1:]:
        pdf.extend(f"{offset:010d} 00000 n \n".encode("ascii"))
    pdf.extend(
        (
            f"trailer\n<< /Size {len(offsets)} /Root 1 0 R /Encrypt 5 0 R >>\n"
            f"startxref\n{xref_offset}\n%%EOF\n"
        ).encode("ascii")
    )
    return bytes(pdf)


def _push_xref_stream_entry(output: bytearray, entry_type: int, field_2: int, field_3: int) -> None:
    output.append(entry_type)
    output.extend(field_2.to_bytes(4, "big"))
    output.extend(field_3.to_bytes(2, "big"))


def embedded_font_pdf() -> bytes:
    pdf = Pdf()
    content = b"BT /F1 18 Tf 20 60 Td (embedded font fixture) Tj ET"
    font_program = b"fake-truetype-font-program"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 180 100] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(
        "<< /Type /Font /Subtype /TrueType /BaseFont /EmbeddedFixture "
        "/FontDescriptor 6 0 R >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    descriptor = pdf.add(
        "<< /Type /FontDescriptor /FontName /EmbeddedFixture /FontFile2 7 0 R >>"
    )
    font_file = pdf.add(
        b"<< /Length "
        + str(len(font_program)).encode("ascii")
        + b" >>\nstream\n"
        + font_program
        + b"\nendstream"
    )
    assert font == 4
    assert descriptor == 6
    assert font_file == 7
    return pdf.render(catalog)


def tounicode_text_pdf() -> bytes:
    pdf = Pdf()
    content = b"BT /F1 24 Tf 30 60 Td <0102> Tj ET"
    cmap = (
        b"/CIDInit /ProcSet findresource begin\n"
        b"1 begincmap\n"
        b"2 beginbfchar\n"
        b"<01> <0041>\n"
        b"<02> <005a>\n"
        b"endbfchar\n"
        b"endcmap\n"
        b"CMapName currentdict /CMap defineresource pop\n"
        b"end"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 100] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(
        "<< /Type /Font /Subtype /Type1 /BaseFont /SubsetFont /ToUnicode 6 0 R >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    cmap_stream = pdf.add(
        b"<< /Length "
        + str(len(cmap)).encode("ascii")
        + b" >>\nstream\n"
        + cmap
        + b"\nendstream"
    )
    assert font == 4
    assert cmap_stream == 6
    return pdf.render(catalog)


def cid_font_text_pdf() -> bytes:
    pdf = Pdf()
    content = b"BT /F1 24 Tf 30 60 Td <00010002> Tj ET"
    cmap = (
        b"/CIDInit /ProcSet findresource begin\n"
        b"1 begincmap\n"
        b"2 beginbfchar\n"
        b"<0001> <0043>\n"
        b"<0002> <0049>\n"
        b"endbfchar\n"
        b"endcmap\n"
        b"CMapName currentdict /CMap defineresource pop\n"
        b"end"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 180 100] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(
        "<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+CIDFixture "
        "/Encoding /Identity-H /DescendantFonts [<< /Type /Font "
        "/Subtype /CIDFontType2 /BaseFont /ABCDEE+CIDFixture "
        "/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> "
        "/DW 600 >>] /ToUnicode 6 0 R >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    cmap_stream = pdf.add(
        b"<< /Length "
        + str(len(cmap)).encode("ascii")
        + b" >>\nstream\n"
        + cmap
        + b"\nendstream"
    )
    assert font == 4
    assert cmap_stream == 6
    return pdf.render(catalog)


def vertical_cjk_text_pdf() -> bytes:
    pdf = Pdf()
    content = b"BT /F1 24 Tf 90 80 Td <00010002> Tj ET"
    cmap = (
        b"/CIDInit /ProcSet findresource begin\n"
        b"1 begincmap\n"
        b"2 beginbfchar\n"
        b"<0001> <65e5>\n"
        b"<0002> <672c>\n"
        b"endbfchar\n"
        b"endcmap\n"
        b"CMapName currentdict /CMap defineresource pop\n"
        b"end"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 180 120] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(
        "<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+VerticalFixture "
        "/Encoding /Identity-V /DescendantFonts [<< /Type /Font "
        "/Subtype /CIDFontType2 /BaseFont /ABCDEE+VerticalFixture "
        "/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> "
        "/DW 1000 >>] /ToUnicode 6 0 R >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    cmap_stream = pdf.add(
        b"<< /Length "
        + str(len(cmap)).encode("ascii")
        + b" >>\nstream\n"
        + cmap
        + b"\nendstream"
    )
    assert font == 4
    assert cmap_stream == 6
    return pdf.render(catalog)


def shaped_rtl_text_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"BT /F1 24 Tf 120 60 Td <0001> Tj ET "
        b"BT /F1 24 Tf 102 60 Td <0002> Tj ET "
        b"BT /F1 24 Tf 84 60 Td <0003> Tj ET "
        b"BT /F1 24 Tf 66 60 Td <0004> Tj ET"
    )
    cmap = (
        b"/CIDInit /ProcSet findresource begin\n"
        b"1 begincmap\n"
        b"4 beginbfchar\n"
        b"<0001> <05e9>\n"
        b"<0002> <05dc>\n"
        b"<0003> <05d5>\n"
        b"<0004> <05dd>\n"
        b"endbfchar\n"
        b"endcmap\n"
        b"CMapName currentdict /CMap defineresource pop\n"
        b"end"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 180 100] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(
        "<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+RtlFixture "
        "/Encoding /Identity-H /DescendantFonts [<< /Type /Font "
        "/Subtype /CIDFontType2 /BaseFont /ABCDEE+RtlFixture "
        "/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> "
        "/DW 600 >>] /ToUnicode 6 0 R >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    cmap_stream = pdf.add(
        b"<< /Length "
        + str(len(cmap)).encode("ascii")
        + b" >>\nstream\n"
        + cmap
        + b"\nendstream"
    )
    assert font == 4
    assert cmap_stream == 6
    return pdf.render(catalog)


def shaped_text_pdf(
    content: bytes,
    cmap: bytes,
    font_dictionary: str,
    media_box: str = "[0 0 180 100]",
) -> bytes:
    pdf = Pdf()
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        f"<< /Type /Page /Parent 3 0 R /MediaBox {media_box} "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(font_dictionary)
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    cmap_stream = pdf.add(
        b"<< /Length "
        + str(len(cmap)).encode("ascii")
        + b" >>\nstream\n"
        + cmap
        + b"\nendstream"
    )
    assert font == 4
    assert cmap_stream == 6
    return pdf.render(catalog)


def opentype_ligature_text_pdf() -> bytes:
    cmap = (
        b"/CIDInit /ProcSet findresource begin\n"
        b"1 begincmap\n"
        b"1 beginbfchar\n"
        b"<01> <00660069>\n"
        b"endbfchar\n"
        b"endcmap\n"
        b"CMapName currentdict /CMap defineresource pop\n"
        b"end"
    )
    return shaped_text_pdf(
        b"BT /F1 30 Tf 30 60 Td <01> Tj ET",
        cmap,
        "<< /Type /Font /Subtype /Type1 /BaseFont /ABCDEE+LigatureFixture "
        "/ToUnicode 6 0 R >>",
    )


def combining_mark_text_pdf() -> bytes:
    cmap = (
        b"/CIDInit /ProcSet findresource begin\n"
        b"1 begincmap\n"
        b"1 beginbfchar\n"
        b"<01> <00650301>\n"
        b"endbfchar\n"
        b"endcmap\n"
        b"CMapName currentdict /CMap defineresource pop\n"
        b"end"
    )
    return shaped_text_pdf(
        b"BT /F1 30 Tf 30 60 Td <01> Tj ET",
        cmap,
        "<< /Type /Font /Subtype /Type1 /BaseFont /ABCDEE+CombiningFixture "
        "/ToUnicode 6 0 R >>",
    )


def chat_emoji_fallback_boundary_pdf() -> bytes:
    cmap = (
        b"/CIDInit /ProcSet findresource begin\n"
        b"1 begincmap\n"
        b"1 beginbfchar\n"
        b"<01> <D83DDE00>\n"
        b"endbfchar\n"
        b"endcmap\n"
        b"CMapName currentdict /CMap defineresource pop\n"
        b"end"
    )
    return shaped_text_pdf(
        (
            b"q 0.98 0.99 1 rg 0 0 220 140 re f Q "
            b"q 0.84 0.92 1 rg 18 58 150 42 re f Q "
            b"BT /F1 12 Tf 28 84 Td (chat export emoji boundary) Tj ET "
            b"BT /F1 24 Tf 176 68 Td <01> Tj ET"
        ),
        cmap,
        "<< /Type /Font /Subtype /Type1 /BaseFont /ABCDEE+EmojiBoundary "
        "/ToUnicode 6 0 R >>",
        "[0 0 220 140]",
    )


def arabic_shaped_text_pdf() -> bytes:
    cmap = (
        b"/CIDInit /ProcSet findresource begin\n"
        b"1 begincmap\n"
        b"3 beginbfchar\n"
        b"<0001> <feb3>\n"
        b"<0002> <fefc>\n"
        b"<0003> <fee1>\n"
        b"endbfchar\n"
        b"endcmap\n"
        b"CMapName currentdict /CMap defineresource pop\n"
        b"end"
    )
    return shaped_text_pdf(
        (
            b"BT /F1 28 Tf 118 60 Td <0001> Tj ET "
            b"BT /F1 28 Tf 92 60 Td <0002> Tj ET "
            b"BT /F1 28 Tf 66 60 Td <0003> Tj ET"
        ),
        cmap,
        "<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+ArabicFixture "
        "/Encoding /Identity-H /DescendantFonts [<< /Type /Font "
        "/Subtype /CIDFontType2 /BaseFont /ABCDEE+ArabicFixture "
        "/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> "
        "/DW 600 >>] /ToUnicode 6 0 R >>",
    )


def identity_h_cjk_text_pdf() -> bytes:
    return shaped_text_pdf(
        b"BT /F1 26 Tf 30 60 Td <65e5672c> Tj ET",
        b"",
        "<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+IdentityCjkFixture "
        "/Encoding /Identity-H /DescendantFonts [<< /Type /Font "
        "/Subtype /CIDFontType2 /BaseFont /ABCDEE+IdentityCjkFixture "
        "/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> "
        "/DW 1000 >>] >>",
    )


def identity_v_cjk_text_pdf() -> bytes:
    return shaped_text_pdf(
        b"BT /F1 26 Tf 90 80 Td <65e5672c> Tj ET",
        b"",
        "<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+IdentityVerticalFixture "
        "/Encoding /Identity-V /DescendantFonts [<< /Type /Font "
        "/Subtype /CIDFontType2 /BaseFont /ABCDEE+IdentityVerticalFixture "
        "/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> "
        "/DW 1000 >>] >>",
        "[0 0 180 120]",
    )


def cmap_codespace_range_text_pdf() -> bytes:
    cmap = (
        b"/CIDInit /ProcSet findresource begin\n"
        b"1 begincmap\n"
        b"1 begincodespacerange\n"
        b"<0000> <ffff>\n"
        b"endcodespacerange\n"
        b"2 beginbfchar\n"
        b"<65e5> <65e5>\n"
        b"<672c> <672c>\n"
        b"endbfchar\n"
        b"endcmap\n"
        b"CMapName currentdict /CMap defineresource pop\n"
        b"end"
    )
    return shaped_text_pdf(
        b"BT /F1 26 Tf 30 60 Td <65e5672c> Tj ET",
        cmap,
        "<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+CodeSpaceCjkFixture "
        "/Encoding /Identity-H /DescendantFonts [<< /Type /Font "
        "/Subtype /CIDFontType2 /BaseFont /ABCDEE+CodeSpaceCjkFixture "
        "/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> "
        "/DW 1000 >>] /ToUnicode 6 0 R >>",
    )


def encoding_differences_pdf() -> bytes:
    pdf = Pdf()
    content = b"BT /F1 24 Tf 30 60 Td (A) Tj ET"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 100] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(
        "<< /Type /Font /Subtype /Type1 /BaseFont /DifferencesFont "
        "/Encoding << /Differences [65 /Z] >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def text_spacing_pdf() -> bytes:
    return page_pdf(
        "[0 0 260 120]",
        (
            "BT /F1 18 Tf 1.5 Tc 5 Tw 90 Tz 20 76 Td "
            "[(office) 160 (export)] TJ "
            "0 Tc 0 Tw 100 Tz 20 -42 Td (normal text) Tj "
            "3 Tr 0 20 Td (hidden) Tj ET"
        ),
    )


def missing_font_pdf(base_font: str, text: str, media_box: str = "[0 0 260 120]") -> bytes:
    pdf = Pdf()
    content = f"BT /F1 18 Tf 24 72 Td ({text}) Tj ET".encode("ascii")
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        f"<< /Type /Page /Parent 3 0 R /MediaBox {media_box} "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(
        f"<< /Type /Font /Subtype /TrueType /BaseFont /{base_font} "
        f"/FontDescriptor 6 0 R >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    descriptor = pdf.add(f"<< /Type /FontDescriptor /FontName /{base_font} >>")
    assert font == 4
    assert descriptor == 6
    return pdf.render(catalog)


def embedded_font_program_text_pdf(
    base_font: str,
    descriptor_field: str,
    font_file_dictionary: str,
    font_program: bytes,
    text: str,
    media_box: str = "[0 0 240 120]",
) -> bytes:
    pdf = Pdf()
    content = f"BT /F1 18 Tf 24 72 Td ({text}) Tj ET".encode("ascii")
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        f"<< /Type /Page /Parent 3 0 R /MediaBox {media_box} "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(
        f"<< /Type /Font /Subtype /Type1 /BaseFont /{base_font} "
        f"/FontDescriptor 6 0 R >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    descriptor = pdf.add(
        f"<< /Type /FontDescriptor /FontName /{base_font} /{descriptor_field} 7 0 R >>"
    )
    font_file_base = font_file_dictionary.strip()
    if font_file_base.endswith(">>"):
        font_file_base = font_file_base[:-2].strip()
    font_file = pdf.add(
        f"{font_file_base} /Length {len(font_program)} >>\nstream\n".encode("ascii")
        + font_program
        + b"\nendstream"
    )
    assert font == 4
    assert descriptor == 6
    assert font_file == 7
    return pdf.render(catalog)


def type1_fontfile_text_pdf() -> bytes:
    program = (
        b"%!PS-AdobeFont-1.0: PdfrustTypeOne 1.0\n"
        b"/CharStrings 2 dict dup begin\n"
        b"/.notdef <0e> def\n"
        b"/A <8bef0d8b8b15ef8b058bef05278b058b2705090e> def\n"
        b"end\n"
    )
    return embedded_font_program_text_pdf(
        "PdfrustTypeOne",
        "FontFile",
        "<<",
        program,
        "type1 fontfile",
    )


def cff_fontfile3_text_pdf() -> bytes:
    return embedded_font_program_text_pdf(
        "PdfrustCffOne",
        "FontFile3",
        "<< /Subtype /Type1C >>",
        b"fake-cff-font-program",
        "cff fontfile3",
    )


def subset_truetype_widths_pdf() -> bytes:
    pdf = Pdf()
    content = b"BT /F1 20 Tf 24 76 Td (ABCA) Tj ET"
    font_program = b"subset-truetype-program"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 220 120] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(
        "<< /Type /Font /Subtype /TrueType /BaseFont /ABCDEE+SubsetSans "
        "/FirstChar 65 /LastChar 67 /Widths [620 580 610] "
        "/FontDescriptor 6 0 R >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    descriptor = pdf.add(
        "<< /Type /FontDescriptor /FontName /ABCDEE+SubsetSans /FontFile2 7 0 R >>"
    )
    font_file = pdf.add(
        b"<< /Length "
        + str(len(font_program)).encode("ascii")
        + b" >>\nstream\n"
        + font_program
        + b"\nendstream"
    )
    assert font == 4
    assert descriptor == 6
    assert font_file == 7
    return pdf.render(catalog)


def subset_cff_tounicode_pdf() -> bytes:
    pdf = Pdf()
    content = b"BT /F1 24 Tf 24 76 Td <0102> Tj ET"
    font_program = b"subset-cff-program"
    cmap = (
        b"/CIDInit /ProcSet findresource begin\n"
        b"1 begincmap\n"
        b"2 beginbfchar\n"
        b"<01> <0043>\n"
        b"<02> <0046>\n"
        b"endbfchar\n"
        b"endcmap\n"
        b"CMapName currentdict /CMap defineresource pop\n"
        b"end"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 220 120] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(
        "<< /Type /Font /Subtype /Type1 /BaseFont /ABCDEE+SubsetCff "
        "/FontDescriptor 6 0 R /ToUnicode 8 0 R >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    descriptor = pdf.add(
        "<< /Type /FontDescriptor /FontName /ABCDEE+SubsetCff /FontFile3 7 0 R >>"
    )
    font_file = pdf.add(
        b"<< /Subtype /Type1C /Length "
        + str(len(font_program)).encode("ascii")
        + b" >>\nstream\n"
        + font_program
        + b"\nendstream"
    )
    cmap_stream = pdf.add(
        b"<< /Length "
        + str(len(cmap)).encode("ascii")
        + b" >>\nstream\n"
        + cmap
        + b"\nendstream"
    )
    assert font == 4
    assert descriptor == 6
    assert font_file == 7
    assert cmap_stream == 8
    return pdf.render(catalog)


def subset_cid_widths_pdf() -> bytes:
    return shaped_text_pdf(
        b"BT /F1 22 Tf 24 76 Td <000100020003> Tj ET",
        (
            b"/CIDInit /ProcSet findresource begin\n"
            b"1 begincmap\n"
            b"3 beginbfchar\n"
            b"<0001> <0057>\n"
            b"<0002> <0049>\n"
            b"<0003> <0044>\n"
            b"endbfchar\n"
            b"endcmap\n"
            b"CMapName currentdict /CMap defineresource pop\n"
            b"end"
        ),
        "<< /Type /Font /Subtype /Type0 /BaseFont /ABCDEE+SubsetCID "
        "/Encoding /Identity-H /DescendantFonts [<< /Type /Font "
        "/Subtype /CIDFontType2 /BaseFont /ABCDEE+SubsetCID "
        "/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> "
        "/DW 600 /W [1 [420 610 730]] >>] /ToUnicode 6 0 R >>",
        "[0 0 220 120]",
    )


def subset_type3_repeated_charprocs_pdf() -> bytes:
    return type3_font_pdf(
        b"BT /F1 34 Tf 22 54 Td (ABABBA) Tj ET",
        "<< /Type /Font /Subtype /Type3 /FontBBox [0 0 620 620] "
        "/FontMatrix [0.001 0 0 0.001 0 0] "
        "/FirstChar 65 /LastChar 66 /Widths [620 560] "
        "/Encoding << /Differences [65 /A /B] >> "
        "/CharProcs << /A 6 0 R /B 7 0 R >> >>",
        [
            b"0.1 0.1 0.1 rg 70 0 m 310 620 l 550 0 l 430 0 l 380 130 l 240 130 l 190 0 l h f",
            b"0.1 0.1 0.1 rg 70 0 180 620 re f 220 0 260 260 re f 220 360 230 260 re f",
        ],
        "[0 0 260 120]",
    )


def subset_missing_font_pdf() -> bytes:
    return missing_font_pdf(
        "ABCDEE+SubsetMissingSans",
        "subset missing font",
        "[0 0 240 120]",
    )


def type3_font_pdf(
    content: bytes,
    font_dictionary: str,
    char_procs: list[bytes],
    media_box: str = "[0 0 220 120]",
) -> bytes:
    pdf = Pdf()
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        f"<< /Type /Page /Parent 3 0 R /MediaBox {media_box} "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add(font_dictionary)
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    for char_proc in char_procs:
        pdf.add(
            f"<< /Length {len(char_proc)} >>\nstream\n".encode("ascii")
            + char_proc
            + b"\nendstream"
        )
    return pdf.render(catalog)


def type3_vector_text_pdf() -> bytes:
    return type3_font_pdf(
        b"BT /F1 28 Tf 20 56 Td (ABBA) Tj ET",
        "<< /Type /Font /Subtype /Type3 /FontBBox [0 0 700 700] "
        "/FontMatrix [0.001 0 0 0.001 0 0] "
        "/FirstChar 65 /LastChar 66 /Widths [700 700] "
        "/Encoding << /Differences [65 /A /B] >> "
        "/CharProcs << /A 6 0 R /B 7 0 R >> >>",
        [
            b"0 0 0 rg 60 0 m 350 700 l 640 0 l 500 0 l 440 150 l 260 150 l 200 0 l h f",
            b"0 0 0 rg 80 0 360 700 re f 320 0 260 300 re f 320 400 220 300 re f",
        ],
    )


def type3_symbol_font_pdf() -> bytes:
    return type3_font_pdf(
        b"BT /F1 42 Tf 36 42 Td (SSS) Tj ET",
        "<< /Type /Font /Subtype /Type3 /FontBBox [0 0 600 600] "
        "/FontMatrix [0.001 0 0 0.001 0 0] "
        "/FirstChar 83 /LastChar 83 /Widths [640] "
        "/Encoding << /Differences [83 /S] >> "
        "/CharProcs << /S 6 0 R >> >>",
        [
            b"0.05 0.15 0.55 rg 300 580 m 390 380 l 580 360 l 430 240 l 480 40 l 300 150 l 120 40 l 170 240 l 20 360 l 210 380 l h f",
        ],
    )


def type3_barcode_font_pdf() -> bytes:
    return type3_font_pdf(
        b"BT /F1 76 Tf 20 42 Td (III) Tj ET",
        "<< /Type /Font /Subtype /Type3 /FontBBox [0 0 520 700] "
        "/FontMatrix [0.001 0 0 0.001 0 0] "
        "/FirstChar 73 /LastChar 73 /Widths [520] "
        "/Encoding << /Differences [73 /I] >> "
        "/CharProcs << /I 6 0 R >> >>",
        [b"0 0 0 rg 30 0 80 700 re f 170 0 80 700 re f 320 0 150 700 re f"],
        media_box="[0 0 220 160]",
    )


def office_table_pdf() -> bytes:
    return page_pdf(
        "[0 0 260 160]",
        (
            "q 0.94 0.96 1 rg 20 112 220 24 re f Q "
            "q 0.15 0.22 0.36 RG 1 w "
            "20 40 m 240 40 l 240 136 l 20 136 l h S "
            "20 112 m 240 112 l S "
            "20 88 m 240 88 l S "
            "20 64 m 240 64 l S "
            "90 40 m 90 136 l S "
            "160 40 m 160 136 l S Q "
            "BT /F1 11 Tf 28 122 Td (Item) Tj "
            "70 0 Td (Q1) Tj "
            "70 0 Td (Q2) Tj "
            "-140 -24 Td (Alpha) Tj 70 0 Td (42) Tj 70 0 Td (55) Tj "
            "-140 -24 Td (Beta) Tj 70 0 Td (37) Tj 70 0 Td (49) Tj "
            "-140 -24 Td (Total) Tj 70 0 Td (79) Tj 70 0 Td (104) Tj ET"
        ),
    )


def report_page_content(title: str, rows: list[tuple[str, str, str]]) -> str:
    ops = [
        "q 0.12 0.34 0.58 rg 18 126 18 18 re f Q",
        "q 0.94 0.96 1 rg 42 118 196 28 re f Q",
        "q 0.15 0.22 0.36 RG 0.8 w",
        "18 36 m 238 36 l 238 146 l 18 146 l h S",
        "18 118 m 238 118 l S",
        "18 94 m 238 94 l S",
        "18 70 m 238 70 l S",
        "92 36 m 92 146 l S",
        "164 36 m 164 146 l S Q",
        f"BT /F1 12 Tf 48 132 Td ({title}) Tj",
        "0 -28 Td (Account) Tj 74 0 Td (Debit) Tj 72 0 Td (Credit) Tj",
    ]
    for name, debit, credit in rows:
        ops.append(f"-146 -24 Td ({name}) Tj 74 0 Td ({debit}) Tj 72 0 Td ({credit}) Tj")
    ops.append("ET")
    return " ".join(ops)


def multi_page_report_pdf() -> bytes:
    pdf = Pdf()
    content_1 = report_page_content(
        "Report p1",
        [("Services", "120", "0"), ("Tax", "24", "0"), ("Paid", "0", "144")],
    ).encode("ascii")
    content_2 = report_page_content(
        "Report p2",
        [("Hosting", "80", "0"), ("Support", "64", "0"), ("Balance", "0", "144")],
    ).encode("ascii")
    contents_1 = pdf.add(
        f"<< /Length {len(content_1)} >>\nstream\n".encode("ascii")
        + content_1
        + b"\nendstream"
    )
    contents_2 = pdf.add(
        f"<< /Length {len(content_2)} >>\nstream\n".encode("ascii")
        + content_2
        + b"\nendstream"
    )
    page_1 = pdf.add(
        "<< /Type /Page /Parent 5 0 R /MediaBox [0 0 260 160] "
        "/Resources << /Font << /F1 6 0 R >> >> "
        f"/Contents {contents_1} 0 R >>"
    )
    page_2 = pdf.add(
        "<< /Type /Page /Parent 5 0 R /MediaBox [0 0 240 180] "
        "/Resources << /Font << /F1 6 0 R >> >> "
        f"/Contents {contents_2} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page_1} 0 R {page_2} 0 R] /Count 2 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert pages == 5
    assert font == 6
    return pdf.render(catalog)


def business_invoice_dense_pdf() -> bytes:
    return page_pdf(
        "[0 0 300 200]",
        (
            "q 0.97 0.98 1 rg 0 0 300 200 re f Q "
            "q 0.08 0.30 0.58 rg 20 158 32 24 re f 0.02 0.12 0.24 rg 56 158 66 24 re f Q "
            "q 0.72 0.04 0.04 RG 1.2 w 220 150 48 20 re S Q "
            "q 0 0 0 rg 210 34 2 26 re f 216 34 1 26 re f 221 34 3 26 re f "
            "228 34 1 26 re f 233 34 4 26 re f 242 34 2 26 re f 248 34 1 26 re f Q "
            "q 0.15 0.22 0.36 RG 0.7 w "
            "20 68 m 280 68 l 280 136 l 20 136 l h S "
            "20 120 m 280 120 l S 20 104 m 280 104 l S 20 88 m 280 88 l S "
            "132 68 m 132 136 l S 192 68 m 192 136 l S 236 68 m 236 136 l S "
            "170 30 m 280 30 l S Q "
            "BT /F1 10 Tf 20 184 Td (PD Acme) Tj 200 0 Td (Invoice 1007) Tj "
            "-200 -20 Td (Bill To: Example LLC) Tj 200 0 Td (PAID) Tj "
            "-200 -30 Td (Item) Tj 112 0 Td (Qty) Tj 60 0 Td (Rate) Tj 44 0 Td (Amount) Tj "
            "-216 -16 Td (Design) Tj 112 0 Td (2) Tj 60 0 Td (120) Tj 44 0 Td (240) Tj "
            "-216 -16 Td (Hosting) Tj 112 0 Td (1) Tj 60 0 Td (80) Tj 44 0 Td (80) Tj "
            "-216 -16 Td (Tax) Tj 112 0 Td (1) Tj 60 0 Td (24) Tj 44 0 Td (24) Tj "
            "56 -24 Td (Total Due 344) Tj -198 -20 Td (Signature) Tj ET"
        ),
    )


def account_statement_ledger_pdf() -> bytes:
    return page_pdf(
        "[0 0 300 200]",
        (
            "q 0.96 0.96 0.94 rg 0 0 300 200 re f Q "
            "q 0.10 0.45 0.28 rg 20 162 22 22 re f 46 162 60 22 re f Q "
            "q 0.15 0.22 0.36 RG 0.7 w "
            "20 42 m 280 42 l 280 146 l 20 146 l h S "
            "20 130 m 280 130 l S 20 114 m 280 114 l S 20 98 m 280 98 l S "
            "20 82 m 280 82 l S 20 66 m 280 66 l S "
            "80 42 m 80 146 l S 162 42 m 162 146 l S 220 42 m 220 146 l S Q "
            "q 0 0 0 rg 34 24 1 12 re f 38 24 3 12 re f 45 24 1 12 re f "
            "50 24 2 12 re f 57 24 1 12 re f 62 24 4 12 re f Q "
            "BT /F1 9 Tf 20 184 Td (Account Statement) Tj 178 0 Td (Period 2026-06) Tj "
            "-178 -30 Td (Date) Tj 60 0 Td (Description) Tj 82 0 Td (Debit) Tj 58 0 Td (Credit) Tj "
            "-200 -16 Td (06-01) Tj 60 0 Td (Opening) Tj 82 0 Td (0) Tj 58 0 Td (500) Tj "
            "-200 -16 Td (06-03) Tj 60 0 Td (Invoice) Tj 82 0 Td (120) Tj 58 0 Td (0) Tj "
            "-200 -16 Td (06-10) Tj 60 0 Td (Payment) Tj 82 0 Td (0) Tj 58 0 Td (120) Tj "
            "-200 -16 Td (06-18) Tj 60 0 Td (Service) Tj 82 0 Td (64) Tj 58 0 Td (0) Tj "
            "-200 -16 Td (06-25) Tj 60 0 Td (Balance) Tj 82 0 Td (64) Tj 58 0 Td (500) Tj ET"
        ),
    )


def financial_annual_report_page_pdf() -> bytes:
    return page_pdf(
        "[0 0 420 300]",
        (
            "q 0.98 0.99 1 rg 0 0 420 300 re f Q "
            "q 0.07 0.17 0.31 rg 24 246 372 30 re f Q "
            "q 0.15 0.22 0.36 RG 0.7 w "
            "24 64 m 260 64 l 260 218 l 24 218 l h S "
            "24 194 m 260 194 l S 24 170 m 260 170 l S 24 146 m 260 146 l S "
            "24 122 m 260 122 l S 24 98 m 260 98 l S "
            "104 64 m 104 218 l S 156 64 m 156 218 l S 208 64 m 208 218 l S "
            "288 82 m 288 214 l 382 82 l S Q "
            "q 0.10 0.45 0.60 rg 300 82 12 52 re f 326 82 12 78 re f 352 82 12 98 re f Q "
            "q 0.72 0.16 0.12 rg 314 82 12 34 re f 340 82 12 48 re f 366 82 12 62 re f Q "
            "BT /F1 12 Tf 32 256 Td (Annual Report Snapshot) Tj 252 0 Td (FY 2026) Tj "
            "/F1 8 Tf -248 -54 Td (Metric) Tj 80 0 Td (2024) Tj 52 0 Td (2025) Tj 52 0 Td (2026) Tj "
            "-184 -24 Td (Revenue) Tj 80 0 Td (940.25) Tj 52 0 Td (981.40) Tj 52 0 Td (1032.10) Tj "
            "-184 -24 Td (Gross Margin) Tj 80 0 Td (41.8%) Tj 52 0 Td (42.6%) Tj 52 0 Td (43.1%) Tj "
            "-184 -24 Td (Operating Cash) Tj 80 0 Td (120.00) Tj 52 0 Td (138.60) Tj 52 0 Td (151.25) Tj "
            "-184 -24 Td (Net Income) Tj 80 0 Td (88.10) Tj 52 0 Td (92.75) Tj 52 0 Td (101.80) Tj "
            "-184 -24 Td (EPS Diluted) Tj 80 0 Td (1.42) Tj 52 0 Td (1.51) Tj 52 0 Td (1.67) Tj "
            "188 126 Td (Revenue) Tj 0 -12 Td (Income) Tj "
            "-264 -162 Td (Synthetic financial fixture; values are examples only.) Tj ET"
        ),
    )


def financial_cashflow_statement_pdf() -> bytes:
    return page_pdf(
        "[0 0 340 260]",
        (
            "q 1 1 1 rg 0 0 340 260 re f Q "
            "q 0.91 0.95 0.98 rg 18 210 304 28 re f Q "
            "q 0.13 0.20 0.28 RG 0.7 w "
            "18 54 m 322 54 l 322 202 l 18 202 l h S "
            "18 178 m 322 178 l S 18 154 m 322 154 l S 18 130 m 322 130 l S "
            "18 106 m 322 106 l S 18 82 m 322 82 l S "
            "178 54 m 178 202 l S 238 54 m 238 202 l S 286 54 m 286 202 l S Q "
            "q 0 0 0 rg 260 28 1 16 re f 265 28 3 16 re f 272 28 1 16 re f "
            "278 28 4 16 re f 286 28 1 16 re f 292 28 2 16 re f Q "
            "BT /F1 10 Tf 24 222 Td (Condensed Cash Flow Statement) Tj 210 0 Td (USD m) Tj "
            "/F1 8 Tf -210 -36 Td (Line item) Tj 160 0 Td (Q1) Tj 60 0 Td (Q2) Tj 48 0 Td (YTD) Tj "
            "-268 -24 Td (Cash from operations) Tj 160 0 Td (41.20) Tj 60 0 Td (44.75) Tj 48 0 Td (85.95) Tj "
            "-268 -24 Td (Capital expenditure) Tj 160 0 Td ((12.05)) Tj 60 0 Td ((13.10)) Tj 48 0 Td ((25.15)) Tj "
            "-268 -24 Td (Free cash flow) Tj 160 0 Td (29.15) Tj 60 0 Td (31.65) Tj 48 0 Td (60.80) Tj "
            "-268 -24 Td (Debt service) Tj 160 0 Td ((8.00)) Tj 60 0 Td ((8.00)) Tj 48 0 Td ((16.00)) Tj "
            "-268 -24 Td (Ending cash) Tj 160 0 Td (76.40) Tj 60 0 Td (100.05) Tj 48 0 Td (100.05) Tj "
            "-268 -46 Td (Decimal columns exercise text alignment and table-rule fidelity.) Tj ET"
        ),
    )


def financial_chart_summary_pdf() -> bytes:
    return page_pdf(
        "[0 0 380 240]",
        (
            "q 0.99 0.99 0.97 rg 0 0 380 240 re f Q "
            "q 0.12 0.28 0.42 rg 0 202 380 38 re f Q "
            "q 0.18 0.24 0.30 RG 0.8 w 32 54 m 32 174 l 222 54 l S "
            "32 84 m 222 84 l 32 114 m 222 114 l 32 144 m 222 144 l S Q "
            "q 0.14 0.48 0.66 rg 54 54 18 54 re f 94 54 18 74 re f 134 54 18 94 re f 174 54 18 70 re f Q "
            "q 0.78 0.28 0.12 rg 74 54 18 38 re f 114 54 18 44 re f 154 54 18 58 re f 194 54 18 64 re f Q "
            "q 0.10 0.10 0.10 RG 1.2 w 56 110 m 96 122 l 136 150 l 176 132 l 216 166 l S Q "
            "q 0.12 0.18 0.25 RG 0.7 w "
            "246 72 m 352 72 l 352 164 l 246 164 l h S "
            "246 141 m 352 141 l S 246 118 m 352 118 l S 246 95 m 352 95 l S "
            "304 72 m 304 164 l S Q "
            "BT /F1 11 Tf 26 216 Td (Financial KPI Summary) Tj 240 0 Td (Q2) Tj "
            "/F1 8 Tf -238 -34 Td (Revenue) Tj 40 0 Td (Cost) Tj 40 0 Td (Margin trend) Tj "
            "116 -18 Td (KPI) Tj 58 0 Td (Value) Tj "
            "-58 -23 Td (ARR) Tj 58 0 Td (12.4) Tj "
            "-58 -23 Td (NRR) Tj 58 0 Td (108%) Tj "
            "-58 -23 Td (OpEx) Tj 58 0 Td (4.8) Tj "
            "-218 -126 Td (Synthetic chart summary; no customer financial data.) Tj ET"
        ),
    )


def thermal_receipt_pdf() -> bytes:
    return page_pdf(
        "[0 0 160 260]",
        (
            "q 0.98 0.98 0.94 rg 0 0 160 260 re f Q "
            "q 0.2 0.2 0.2 RG 0.6 w 18 44 m 142 44 l 18 190 m 142 190 l S Q "
            "q 0 0 0 rg 34 20 1 18 re f 38 20 2 18 re f 44 20 1 18 re f "
            "50 20 4 18 re f 60 20 1 18 re f 66 20 3 18 re f 74 20 1 18 re f "
            "82 20 2 18 re f 90 20 1 18 re f 96 20 5 18 re f Q "
            "BT /F1 9 Tf 36 230 Td (PD MARKET) Tj -10 -18 Td (Receipt 42) Tj "
            "-8 -26 Td (Coffee) Tj 78 0 Td (4.50) Tj "
            "-78 -14 Td (Bagel) Tj 78 0 Td (3.25) Tj "
            "-78 -14 Td (Tax) Tj 78 0 Td (0.62) Tj "
            "-78 -22 Td (Total) Tj 78 0 Td (8.37) Tj "
            "-72 -28 Td (CARD APPROVED) Tj "
            "-2 -88 Td (Thank you) Tj ET"
        ),
    )


def business_form_stamp_signature_pdf() -> bytes:
    return page_pdf(
        "[0 0 260 180]",
        (
            "q 0.95 0.97 0.98 rg 0 0 260 180 re f Q "
            "q 0.15 0.22 0.36 RG 0.8 w 18 22 224 132 re S "
            "18 124 m 242 124 l S 18 94 m 242 94 l S 18 64 m 242 64 l S "
            "126 22 m 126 154 l S Q "
            "q 0.78 0.06 0.05 RG 1.5 w 176 112 46 24 re S Q "
            "q 0.05 0.14 0.28 rg 168 28 1 12 re f 172 28 3 12 re f 179 28 1 12 re f "
            "184 28 2 12 re f 191 28 1 12 re f 196 28 4 12 re f Q "
            "q 0 0 0 RG 1 w 34 76 10 10 re S 38 80 m 43 86 l S 43 86 m 51 70 l S "
            "34 48 10 10 re S 152 48 m 222 48 l S Q "
            "BT /F1 10 Tf 24 142 Td (Business Intake Form) Tj 156 0 Td (REVIEWED) Tj "
            "-156 -32 Td (Company) Tj 108 0 Td (Example LLC) Tj "
            "-108 -30 Td (Approved) Tj 108 0 Td (Yes) Tj "
            "-108 -30 Td (Signature) Tj 108 0 Td (A Example) Tj ET"
        ),
    )


def government_permit_checkbox_form_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.98 0.99 1 rg 0 0 360 260 re f Q "
        b"q 0.10 0.24 0.38 rg 0 222 360 38 re f Q "
        b"q 0.15 0.22 0.36 RG 0.8 w 24 36 312 164 re S "
        b"24 168 m 336 168 l S 24 132 m 336 132 l S 24 96 m 336 96 l S "
        b"128 36 m 128 200 l S 232 36 m 232 200 l S Q "
        b"q 0.72 0.05 0.05 RG 1.4 w 262 176 46 18 re S Q "
        b"q 0 0 0 rg 258 54 1 18 re f 263 54 3 18 re f 270 54 1 18 re f "
        b"277 54 4 18 re f 286 54 1 18 re f 292 54 3 18 re f Q "
        b"BT /F1 11 Tf 24 238 Td (Public Works Permit) Tj "
        b"/F1 8 Tf 30 188 Td (Applicant) Tj 104 0 Td (Permit ID) Tj 104 0 Td (Status) Tj "
        b"-208 -36 Td (Example Works LLC) Tj 104 0 Td (PW-0148) Tj 104 0 Td (APPROVED) Tj "
        b"-208 -36 Td (Inspection) Tj 104 0 Td (Required) Tj 104 0 Td (Scheduled) Tj "
        b"-208 -58 Td (I certify the information is complete) Tj "
        b"174 -32 Td (Authorized signature) Tj ET"
    )
    yes_appearance = (
        b"1 1 1 rg 0 0 16 16 re f "
        b"0 0 0 RG 0.8 w 0.5 0.5 15 15 re S "
        b"0 0 0 RG 1.2 w 4 8 m 7 4 l S 7 4 m 13 13 l S"
    )
    off_appearance = b"1 1 1 rg 0 0 16 16 re f 0 0 0 RG 0.8 w 0.5 0.5 15 15 re S"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 360 260] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R /Annots [7 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    yes_appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 16 16] /Length "
        + str(len(yes_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + yes_appearance
        + b"\nendstream"
    )
    off_appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 16 16] /Length "
        + str(len(off_appearance)).encode("ascii")
        + b" >>\nstream\n"
        + off_appearance
        + b"\nendstream"
    )
    field = pdf.add(
        "<< /Type /Annot /Subtype /Widget /FT /Btn /T (Certification) /V /Yes /AS /Yes "
        "/Rect [30 64 46 80] "
        f"/AP << /N << /Yes {yes_appearance_object} 0 R /Off {off_appearance_object} 0 R >> >> >>"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /AcroForm << /Fields [{field} 0 R] >> >>"
    )
    assert font == 4
    assert field == 7
    return pdf.render(catalog)


def government_certificate_seal_signature_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.99 0.98 0.93 rg 0 0 420 300 re f Q "
        b"q 0.18 0.22 0.28 RG 1.2 w 24 24 372 252 re S 32 32 356 236 re S Q "
        b"q 0.72 0.05 0.05 RG 1.8 w 294 152 58 58 re S 304 162 38 38 re S "
        b"314 181 m 332 181 l S 323 172 m 323 190 l S Q "
        b"q 0 0 0 rg 54 44 4 4 re f 66 44 4 4 re f 78 44 4 4 re f "
        b"54 56 4 4 re f 78 56 4 4 re f 90 56 4 4 re f "
        b"54 68 4 4 re f 66 68 4 4 re f 90 68 4 4 re f Q "
        b"q 0.12 0.12 0.12 RG 0.9 w 236 70 m 360 70 l S Q "
        b"BT /F1 16 Tf 92 242 Td (Certificate of Public Record) Tj "
        b"/F1 9 Tf 74 210 Td (This synthetic certificate records a public-agency style layout.) Tj "
        b"/F1 10 Tf 98 178 Td (Certificate No. CERT-0148) Tj "
        b"/F1 8 Tf 278 136 Td (OFFICIAL SEAL) Tj "
        b"236 56 Td (Authorized signature appearance) Tj "
        b"-182 28 Td (QR marker) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 420 300] "
        "/Resources << /Font << /F1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R "
        "/PageLabels << /Nums [0 << /P (CERT-) /S /D /St 1 >>] >> >>"
    )
    assert font == 4
    return pdf.render(catalog)


def government_tax_notice_barcode_pdf() -> bytes:
    return page_pdf(
        "[0 0 360 260]",
        (
            "q 1 1 1 rg 0 0 360 260 re f Q "
            "q 0.92 0.95 0.98 rg 18 206 324 34 re f Q "
            "q 0.18 0.22 0.28 RG 0.7 w 18 42 324 164 re S "
            "18 174 m 342 174 l S 18 142 m 342 142 l S 18 110 m 342 110 l S "
            "18 78 m 342 78 l S 126 42 m 126 206 l S 246 42 m 246 206 l S Q "
            "q 0.76 0.06 0.04 RG 1.4 w 260 184 48 18 re S Q "
            "q 0 0 0 rg 252 54 1 22 re f 258 54 3 22 re f 266 54 1 22 re f "
            "272 54 5 22 re f 282 54 1 22 re f 288 54 3 22 re f 296 54 1 22 re f Q "
            "BT /F1 11 Tf 24 222 Td (Tax Payment Notice) Tj 214 0 Td (FILED) Tj "
            "/F1 8 Tf 28 190 Td (Account) Tj 108 0 Td (Period) Tj 120 0 Td (Amount) Tj "
            "-228 -32 Td (TAX-0148) Tj 108 0 Td (2026-Q2) Tj 120 0 Td (128.40) Tj "
            "-228 -32 Td (Due Date) Tj 108 0 Td (Status) Tj 120 0 Td (Barcode) Tj "
            "-228 -32 Td (2026-07-31) Tj 108 0 Td (Open) Tj "
            "0 -70 Td (Public agency synthetic fixture; no private identity data.) Tj ET"
        ),
    )


def browser_chromium_article_print_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 1 1 1 rg 0 0 360 460 re f Q "
        b"q 0.10 0.16 0.26 rg 0 414 360 46 re f Q "
        b"q 0.92 0.95 0.99 rg 28 286 304 96 re f Q "
        b"q 76 0 0 46 42 316 cm /Hero Do Q "
        b"q 0.16 0.20 0.28 RG 0.6 w 28 274 m 332 274 l 28 248 m 332 248 l "
        b"28 222 m 332 222 l 28 196 m 332 196 l S Q "
        b"q 0.05 0.28 0.65 RG 0.8 w 42 74 m 168 74 l S Q "
        b"BT /F1 14 Tf 28 430 Td (Browser Article Print) Tj "
        b"/F1 9 Tf 28 394 Td (Author: Example Newsroom) Tj "
        b"/F1 10 Tf 132 350 Td (Hero image with CSS background panel) Tj "
        b"/F1 8 Tf 42 258 Td (Section) Tj 94 0 Td (Metric) Tj 82 0 Td (Status) Tj "
        b"-176 -26 Td (Traffic) Tj 94 0 Td (128k) Tj 82 0 Td (stable) Tj "
        b"-176 -26 Td (Signup) Tj 94 0 Td (4.8%) Tj 82 0 Td (up) Tj "
        b"/F1 8 Tf 42 160 Td (Main paragraph text wraps into print columns with link styling.) Tj "
        b"0 -18 Td (CSS backgrounds and table rules stay visible in print mode.) Tj "
        b"0 -68 Td (https://example.invalid/article) Tj ET"
    )
    hero = bytes(
        [
            56,
            96,
            150,
            76,
            124,
            178,
            104,
            148,
            194,
            224,
            232,
            240,
            232,
            140,
            72,
            218,
            122,
            56,
            188,
            102,
            50,
            42,
            74,
            126,
        ]
    )
    appearance = b"0.05 0.28 0.65 RG 1 w 0 1 m 126 1 l S"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 360 460] "
        "/Resources << /Font << /F1 4 0 R >> /XObject << /Hero 5 0 R >> >> "
        f"/Contents {contents} 0 R /Annots [7 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    image = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 4 /Height 2 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length "
        + str(len(hero)).encode("ascii")
        + b" >>\nstream\n"
        + hero
        + b"\nendstream"
    )
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 126 12] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Link /Rect [42 68 168 82] "
        "/Border [0 0 0] /A << /S /URI /URI (https://example.invalid/article) >> "
        f"/AP << /N {appearance_object} 0 R >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    assert image == 5
    assert annotation == 7
    return pdf.render(catalog)


def browser_firefox_dashboard_print_pdf() -> bytes:
    ops: list[str] = [
        "q 0.97 0.98 0.99 rg 0 0 420 300 re f Q",
        "q 0.12 0.16 0.24 rg 0 256 420 44 re f Q",
        "q 1 1 1 rg 24 168 112 62 re f 154 168 112 62 re f 284 168 112 62 re f Q",
        "q 0.15 0.47 0.72 rg 24 168 112 16 re f 154 168 112 16 re f Q",
        "q 0.85 0.36 0.14 rg 284 168 112 16 re f Q",
        "q 1 1 1 rg 24 38 184 100 re f 230 38 166 100 re f Q",
        "q 0.18 0.20 0.26 RG 0.6 w 24 38 184 100 re S 230 38 166 100 re S",
    ]
    for y in range(58, 139, 20):
        ops.append(f"230 {y} m 396 {y} l S")
    for x in range(258, 397, 34):
        ops.append(f"{x} 38 m {x} 138 l S")
    ops.extend(
        [
            "Q",
            "q 0.18 0.47 0.72 rg 48 58 24 58 re f 88 58 24 42 re f 128 58 24 70 re f Q",
            "q 0.86 0.36 0.14 RG 1.6 w 48 96 m 88 84 l 128 116 l 168 104 l S Q",
            "BT /F1 12 Tf 24 274 Td (Dashboard Print) Tj /F1 8 Tf 24 214 Td (Revenue) Tj 130 0 Td (Orders) Tj 130 0 Td (Queue) Tj ET",
            "BT /F1 16 Tf 36 192 Td (128k) Tj 130 0 Td (912) Tj 130 0 Td (17) Tj ET",
            "BT /F1 8 Tf 34 146 Td (Chart) Tj 212 0 Td (Detail table) Tj ET",
        ]
    )
    for row in range(4):
        y = 122 - row * 20
        ops.append(
            f"BT /F1 6 Tf 236 {y} Td (Item {row + 1}) Tj 68 0 Td ({row * 17 + 40}) Tj "
            f"44 0 Td (ok) Tj ET"
        )
    return page_pdf("[0 0 420 300]", " ".join(ops))


def browser_webkit_receipt_form_print_pdf() -> bytes:
    ops: list[str] = [
        "q 0.98 0.98 0.94 rg 0 0 240 360 re f Q",
        "q 1 1 1 rg 24 38 192 284 re f Q",
        "q 0.18 0.18 0.18 RG 0.6 w 24 38 192 284 re S 24 278 m 216 278 l 24 116 m 216 116 l S Q",
        "q 0.96 0.72 0.16 rg 188 294 14 14 re f Q",
        "q 42 236 156 18 re W n BT /F1 8 Tf 42 242 Td (Very long clipped product title from print CSS overflow) Tj ET Q",
        "q 0 0 0 rg 52 58 1 24 re f 58 58 3 24 re f 66 58 1 24 re f 74 58 4 24 re f 84 58 1 24 re f 92 58 3 24 re f Q",
        "q 0 0 0 RG 0.8 w 42 152 10 10 re S 46 156 m 52 164 l S 52 164 m 60 146 l S Q",
        "BT /F1 10 Tf 42 302 Td (Web Checkout Receipt) Tj /F1 7 Tf 42 264 Td (Item) Tj 110 0 Td (Amount) Tj ET",
        "BT /F1 8 Tf 42 218 Td (Coffee beans) Tj 110 0 Td (12.00) Tj 0 -20 Td (Shipping) Tj 110 0 Td (4.50) Tj ET",
        "BT /F1 9 Tf 42 96 Td (Total 16.50) Tj /F1 7 Tf 42 138 Td (Subscribe to email updates) Tj 42 42 Td (Order 2026-0146) Tj ET",
    ]
    return page_pdf("[0 0 240 360]", " ".join(ops))


def browser_print_sticky_headers_pdf() -> bytes:
    ops: list[str] = [
        "q 0.99 0.99 0.98 rg 0 0 160 160 re f Q",
        "q 0.10 0.16 0.26 rg 0 136 160 24 re f Q",
        "q 0.92 0.95 0.99 rg 16 108 128 14 re f 16 58 128 14 re f Q",
        "q 0.70 0.78 0.88 rg 16 96 128 8 re f 16 46 128 8 re f Q",
    ]
    for y in range(24, 123, 12):
        ops.append(f"0.78 0.82 0.88 rg 16 {y} 128 1 re f")
    for x in (16, 56, 104, 144):
        ops.append(f"0.78 0.82 0.88 rg {x} 24 1 98 re f")
    ops.extend(
        [
            "q 0.12 0.42 0.68 rg 22 112 28 5 re f 62 112 28 5 re f 110 112 28 5 re f Q",
            "q 0.12 0.42 0.68 rg 22 62 28 5 re f 62 62 28 5 re f 110 62 28 5 re f Q",
        ]
    )
    return page_pdf("[0 0 160 160]", " ".join(ops))


def browser_print_clipped_backgrounds_pdf() -> bytes:
    ops: list[str] = [
        "q 0.98 0.98 0.96 rg 0 0 160 160 re f Q",
        "q 18 30 48 96 re W n 0.10 0.34 0.72 rg 0 0 160 160 re f Q",
        "q 94 30 48 96 re W n 0.88 0.42 0.12 rg 0 0 160 160 re f Q",
        "q 0.92 0.94 0.98 rg 28 58 28 18 re f 104 58 28 18 re f Q",
    ]
    return page_pdf("[0 0 160 160]", " ".join(ops))


def browser_print_transformed_cards_pdf() -> bytes:
    ops: list[str] = [
        "q 0.98 0.99 1 rg 0 0 160 160 re f Q",
        "q 2 0 0 2 16 24 cm 0.16 0.36 0.68 rg 0 0 28 18 re f Q",
        "q 1 0 0 1 86 28 cm 0.86 0.34 0.16 rg 0 0 54 34 re f Q",
        "q 1 0 0 1 24 100 cm 0.18 0.56 0.38 rg 0 0 112 22 re f Q",
    ]
    return page_pdf("[0 0 160 160]", " ".join(ops))


def browser_print_raster_vector_mix_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.99 0.99 0.98 rg 0 0 160 160 re f Q "
        b"q 56 0 0 32 18 88 cm /Hero Do Q "
        b"q 0.12 0.34 0.66 rg 88 88 16 34 re f 112 88 16 24 re f 136 88 16 44 re f Q "
        b"q 0.10 0.12 0.16 rg 18 78 134 2 re f 88 88 64 2 re f Q"
    )
    hero = bytes(
        [
            42,
            78,
            130,
            64,
            102,
            158,
            92,
            132,
            180,
            224,
            232,
            240,
            236,
            154,
            82,
            210,
            120,
            56,
            180,
            96,
            48,
            66,
            96,
            142,
        ]
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 160 160] "
        "/Resources << /Font << /F1 4 0 R >> /XObject << /Hero 5 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    image = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 4 /Height 2 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length "
        + str(len(hero)).encode("ascii")
        + b" >>\nstream\n"
        + hero
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    assert image == 5
    return pdf.render(catalog)


def email_client_thread_pdf() -> bytes:
    pdf = Pdf()
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")

    def content(page: int) -> bytes:
        y_offset = (page - 1) * 8
        ops = [
            "q 0.99 0.99 0.98 rg 0 0 300 420 re f Q",
            "q 0.12 0.18 0.28 rg 0 382 300 38 re f Q",
            "q 0.91 0.94 0.98 rg 24 304 252 54 re f Q",
            "q 0.96 0.96 0.94 rg 24 206 252 78 re f Q",
            "q 0.90 0.93 0.88 rg 24 94 252 88 re f Q",
            "q 0.70 0.76 0.84 rg 36 316 92 5 re f 36 266 168 4 re f 36 246 196 4 re f Q",
            "q 0.70 0.70 0.66 rg 36 224 132 4 re f 36 202 180 4 re f 36 154 206 4 re f Q",
            "q 0.16 0.38 0.66 rg 36 110 78 5 re f 36 88 158 4 re f Q",
            (
                "BT /F1 12 Tf 24 394 Td (Re: quarterly launch thread) Tj "
                f"/F1 8 Tf 36 {340 - y_offset} Td (From: Priya Example) Tj "
                f"0 -18 Td (To: release-list@example.invalid) Tj "
                f"0 -68 Td (On Tue, Alex wrote:) Tj "
                f"0 -104 Td (Forwarded web archive summary follows.) Tj ET"
            ),
        ]
        return " ".join(ops).encode("ascii")

    contents = [
        pdf.add(
            f"<< /Length {len(page_content)} >>\nstream\n".encode("ascii")
            + page_content
            + b"\nendstream"
        )
        for page_content in (content(1), content(2), content(3))
    ]
    page_1 = pdf.add(
        "<< /Type /Page /Parent 8 0 R /MediaBox [0 0 300 420] "
        f"/Resources << /Font << /F1 {font} 0 R >> >> /Contents {contents[0]} 0 R >>"
    )
    page_2 = pdf.add(
        "<< /Type /Page /Parent 8 0 R /MediaBox [0 0 300 420] "
        f"/Resources << /Font << /F1 {font} 0 R >> >> /Contents {contents[1]} 0 R >>"
    )
    page_3 = pdf.add(
        "<< /Type /Page /Parent 8 0 R /MediaBox [0 0 300 420] "
        f"/Resources << /Font << /F1 {font} 0 R >> >> /Contents {contents[2]} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page_1} 0 R {page_2} 0 R {page_3} 0 R] /Count 3 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 1
    assert pages == 8
    return pdf.render(catalog)


def email_inline_image_link_pdf() -> bytes:
    pdf = Pdf()
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    image = bytes(
        [
            34,
            78,
            128,
            82,
            138,
            190,
            224,
            232,
            240,
            238,
            184,
            96,
            186,
            94,
            72,
            88,
            132,
            172,
        ]
        * 3
    )
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 3 /Height 6 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )
    content = (
        b"q 0.99 0.99 1 rg 0 0 260 220 re f Q "
        b"q 0.14 0.22 0.36 rg 0 188 260 32 re f Q "
        b"q 72 0 0 72 24 84 cm /Hero Do Q "
        b"q 0.86 0.92 0.98 rg 116 92 112 44 re f Q "
        b"q 0.12 0.38 0.66 rg 126 118 76 5 re f 126 104 88 4 re f Q "
        b"BT /F1 11 Tf 24 199 Td (Mobile mail inline image) Tj "
        b"/F1 8 Tf 116 154 Td (Campaign preview) Tj 0 -92 Td (Open archived link) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    appearance = b"0.72 0.86 1 rg 0 0 74 12 re f"
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 74 12] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Link /Rect [116 58 190 70] "
        "/Border [0 0 0] /A << /S /URI /URI (https://example.invalid/archive) >> "
        f"/AP << /N {appearance_object} 0 R >> >>"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 7 0 R /MediaBox [0 0 260 220] "
        f"/Resources << /Font << /F1 {font} 0 R >> /XObject << /Hero {image_object} 0 R >> >> "
        f"/Contents {contents} 0 R /Annots [{annotation} 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert pages == 7
    return pdf.render(catalog)


def web_archive_snapshot_pdf() -> bytes:
    pdf = Pdf()
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    content = (
        b"q 0.98 0.98 0.96 rg 0 0 320 360 re f Q "
        b"q 0.12 0.18 0.28 rg 0 326 320 34 re f Q "
        b"q 0.92 0.95 0.98 rg 24 236 176 68 re f Q "
        b"q 0.95 0.92 0.84 rg 216 92 76 212 re f Q "
        b"q 0.14 0.36 0.58 rg 36 274 118 6 re f 36 254 144 4 re f 36 122 150 4 re f Q "
        b"q 0.64 0.42 0.18 rg 228 268 52 5 re f 228 232 44 4 re f 228 196 56 4 re f Q "
        b"BT /F1 12 Tf 24 337 Td (Saved Web Archive) Tj "
        b"/F1 8 Tf 36 292 Td (Article content captured from reader mode.) Tj "
        b"0 -188 Td (Original URL remains an inert link annotation.) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    appearance = b"0.72 0.86 1 rg 0 0 92 12 re f"
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 92 12] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Link /Rect [36 98 128 110] "
        "/Border [0 0 0] /A << /S /URI /URI (https://example.invalid/story) >> "
        f"/AP << /N {appearance_object} 0 R >> >>"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 6 0 R /MediaBox [0 0 320 360] "
        f"/Resources << /Font << /F1 {font} 0 R >> >> "
        f"/Contents {contents} 0 R /Annots [{annotation} 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert pages == 6
    return pdf.render(catalog)


def email_attachment_summary_pdf() -> bytes:
    pdf = Pdf()
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    attachment = b"meeting notes attachment\n"
    content = (
        b"q 1 1 1 rg 0 0 240 180 re f Q "
        b"q 0.13 0.19 0.30 rg 0 146 240 34 re f Q "
        b"q 1 0.94 0.70 rg 24 58 118 28 re f Q "
        b"q 0.16 0.16 0.16 RG 0.8 w 24 58 118 28 re S Q "
        b"BT /F1 11 Tf 24 158 Td (Mail Attachment Summary) Tj "
        b"/F1 8 Tf 34 68 Td (notes.txt attached) Tj 24 116 Td (Payload is metadata only.) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    embedded_file = pdf.add(
        f"<< /Type /EmbeddedFile /Subtype /text#2Fplain /Length {len(attachment)} >>\nstream\n".encode("ascii")
        + attachment
        + b"endstream"
    )
    filespec = pdf.add(
        f"<< /Type /Filespec /F (notes.txt) /UF (notes.txt) /EF << /F {embedded_file} 0 R >> >>"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 6 0 R /MediaBox [0 0 240 180] "
        f"/Resources << /Font << /F1 {font} 0 R >> >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R "
        f"/Names << /EmbeddedFiles << /Names [(notes.txt) {filespec} 0 R] >> >> >>"
    )
    assert pages == 6
    return pdf.render(catalog)


def office_report_header_footer_link_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.98 0.99 1 rg 0 0 480 300 re f Q "
        b"q 0.10 0.24 0.38 rg 0 256 480 44 re f Q "
        b"q 0.92 0.95 0.98 rg 24 54 432 170 re f Q "
        b"q 0.15 0.20 0.28 RG 0.7 w "
        b"24 54 m 456 54 l 456 224 l 24 224 l h S "
        b"24 196 m 456 196 l S 24 168 m 456 168 l S 24 140 m 456 140 l S "
        b"24 112 m 456 112 l S 24 84 m 456 84 l S "
        b"150 54 m 150 224 l S 252 54 m 252 224 l S 354 54 m 354 224 l S Q "
        b"q 30 0 0 20 34 242 cm /ImLogo Do Q "
        b"q 0.05 0.28 0.65 RG 0.9 w 360 38 m 456 38 l S Q "
        b"BT /F1 13 Tf 72 276 Td (Quarterly Business Review) Tj "
        b"/F1 8 Tf 348 0 Td (Generated Office Export) Tj "
        b"/F1 9 Tf -396 -36 Td (Section) Tj 126 0 Td (Owner) Tj 102 0 Td (Status) Tj 102 0 Td (Due) Tj "
        b"-330 -28 Td (Revenue Plan) Tj 126 0 Td (M Lee) Tj 102 0 Td (Green) Tj 102 0 Td (Jun 30) Tj "
        b"-330 -28 Td (Hiring) Tj 126 0 Td (A Diaz) Tj 102 0 Td (Amber) Tj 102 0 Td (Jul 12) Tj "
        b"-330 -28 Td (Risk Review) Tj 126 0 Td (R Chen) Tj 102 0 Td (Green) Tj 102 0 Td (Jul 18) Tj "
        b"-330 -28 Td (Launch) Tj 126 0 Td (P Khan) Tj 102 0 Td (Blue) Tj 102 0 Td (Aug 05) Tj "
        b"/F1 8 Tf 24 -48 Td (Footer: synthetic fixture, no private data) Tj "
        b"276 0 Td (https://example.invalid/report) Tj ET"
    )
    logo = bytes(
        [
            20,
            64,
            120,
            20,
            64,
            120,
            230,
            238,
            247,
            230,
            238,
            247,
            236,
            120,
            48,
            236,
            120,
            48,
        ]
    )
    appearance = b"0.05 0.28 0.65 RG 1 w 0 1 m 96 1 l S"
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 480 300] "
        "/Resources << /Font << /F1 4 0 R >> /XObject << /ImLogo 5 0 R >> >> "
        f"/Contents {contents} 0 R /Annots [7 0 R] >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    image = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 3 /Height 2 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length "
        + str(len(logo)).encode("ascii")
        + b" >>\nstream\n"
        + logo
        + b"\nendstream"
    )
    appearance_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 96 12] /Length "
        + str(len(appearance)).encode("ascii")
        + b" >>\nstream\n"
        + appearance
        + b"\nendstream"
    )
    annotation = pdf.add(
        "<< /Type /Annot /Subtype /Link /Rect [360 32 456 44] "
        "/Border [0 0 0] /A << /S /URI /URI (https://example.invalid/report) >> "
        f"/AP << /N {appearance_object} 0 R >> >>"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    assert image == 5
    assert annotation == 7
    return pdf.render(catalog)


def office_spreadsheet_chart_comments_pdf() -> bytes:
    ops: list[str] = [
        "q 1 1 1 rg 0 0 420 260 re f Q",
        "q 0.93 0.96 0.99 rg 20 210 380 26 re f Q",
        "q 0.30 0.30 0.30 RG 0.4 w",
    ]
    for x in range(20, 221, 40):
        ops.append(f"{x} 60 m {x} 236 l S")
    for y in range(60, 237, 16):
        ops.append(f"20 {y} m 220 {y} l S")
    ops.extend(
        [
            "Q",
            "q 0.95 0.72 0.18 rg 202 204 14 14 re f 202 156 14 14 re f Q",
            "q 0.18 0.44 0.72 rg 268 74 24 84 re f 308 74 24 118 re f 348 74 24 96 re f Q",
            "q 0.86 0.34 0.16 RG 1.8 w 268 134 m 308 172 l 348 150 l 388 196 l S Q",
            "q 0.16 0.18 0.22 RG 0.6 w 246 74 m 246 206 l 398 74 l S Q",
            "BT /F1 10 Tf 24 220 Td (Sales Plan) Tj 244 0 Td (Chart) Tj ET",
            "BT /F1 6 Tf 28 198 Td (Region) Tj 40 0 Td (Q1) Tj 40 0 Td (Q2) Tj 40 0 Td (Q3) Tj 40 0 Td (Note) Tj ET",
        ]
    )
    for row in range(8):
        y = 182 - row * 16
        ops.append(
            f"BT /F1 5 Tf 28 {y} Td (R{row + 1}) Tj 40 0 Td ({100 + row * 9}) Tj "
            f"40 0 Td ({120 + row * 7}) Tj 40 0 Td ({130 + row * 11}) Tj "
            f"40 0 Td (ok) Tj ET"
        )
    ops.extend(
        [
            "BT /F1 6 Tf 270 62 Td (Jan) Tj 40 0 Td (Feb) Tj 40 0 Td (Mar) Tj ET",
            "BT /F1 7 Tf 246 222 Td (Comment markers use yellow triangles) Tj ET",
        ]
    )
    return page_pdf("[0 0 420 260]", " ".join(ops))


def office_presentation_handout_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.98 0.98 0.96 rg 0 0 360 300 re f Q "
        b"q 0.12 0.18 0.30 rg 24 166 136 82 re f 200 166 136 82 re f Q "
        b"q 0.94 0.47 0.16 rg 42 220 48 10 re f 218 214 78 12 re f Q "
        b"q 56 0 0 34 76 178 cm /ImPhoto Do Q "
        b"q 0.18 0.48 0.72 rg 226 178 20 42 re f 258 178 20 56 re f 290 178 20 32 re f Q "
        b"q 0.68 0.68 0.68 RG 0.6 w 24 130 m 336 130 l "
        b"24 106 m 336 106 l 24 82 m 336 82 l 24 58 m 336 58 l 24 34 m 336 34 l S Q "
        b"BT /F1 11 Tf 24 268 Td (Presentation Handout) Tj "
        b"/F1 8 Tf 42 232 Td (Product update) Tj 218 232 Td (Pipeline chart) Tj "
        b"24 142 Td (Speaker notes) Tj "
        b"0 -24 Td (1. Launch readiness) Tj "
        b"0 -24 Td (2. Customer feedback) Tj "
        b"0 -24 Td (3. Follow-up actions) Tj ET"
    )
    photo = bytes(
        [
            36,
            80,
            140,
            48,
            104,
            166,
            70,
            128,
            184,
            98,
            150,
            190,
            222,
            226,
            218,
            198,
            210,
            214,
            160,
            190,
            210,
            120,
            166,
            198,
            232,
            150,
            72,
            224,
            132,
            56,
            208,
            110,
            44,
            180,
            92,
            38,
        ]
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 360 300] "
        "/Resources << /Font << /F1 4 0 R >> /XObject << /ImPhoto 5 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    image = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 4 /Height 3 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length "
        + str(len(photo)).encode("ascii")
        + b" >>\nstream\n"
        + photo
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    assert image == 5
    return pdf.render(catalog)


def office_vector_grouped_shapes_pdf() -> bytes:
    ops: list[str] = [
        "q 0.98 0.98 0.97 rg 0 0 320 220 re f Q",
        "q 1 0 0 1 22 34 cm",
        "0.12 0.28 0.52 rg 0 0 116 64 re f",
        "0.94 0.55 0.18 rg 12 14 38 28 re f",
        "0.18 0.58 0.42 rg 62 14 38 28 re f",
        "0.06 0.12 0.22 RG 1.2 w 0 0 116 64 re S",
        "Q",
        "q 0.92 0.94 0.98 rg 164 34 128 64 re f Q",
        "q 0.16 0.36 0.68 RG 2 w 176 52 m 206 82 l 236 48 l 276 86 l S Q",
        "q 0.12 0.14 0.20 rg 176 110 30 30 re f 222 110 30 30 re f 268 110 30 30 re f Q",
        "BT /F1 10 Tf 24 186 Td (Grouped vector shapes) Tj ET",
    ]
    return page_pdf("[0 0 320 220]", " ".join(ops))


def office_vector_nested_clips_pdf() -> bytes:
    ops: list[str] = [
        "q 0.97 0.97 0.95 rg 0 0 300 210 re f Q",
        "q 28 34 238 132 re W n",
        "56 54 170 92 re W n",
        "0.88 0.94 1 rg 0 0 300 210 re f",
        "0.18 0.42 0.72 RG 1.2 w",
    ]
    for x in range(20, 286, 12):
        ops.append(f"{x} 18 m {x - 66} 186 l S")
    ops.extend(
        [
            "0.86 0.30 0.14 rg 42 52 208 90 re f",
            "Q",
            "0.08 0.10 0.14 RG 1.2 w 28 34 238 132 re S 56 54 170 92 re S",
            "BT /F1 10 Tf 30 184 Td (Nested clip masks) Tj ET",
        ]
    )
    return page_pdf("[0 0 300 210]", " ".join(ops))


def office_vector_clipped_transparency_group_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.98 0.98 0.97 rg 0 0 320 220 re f Q "
        b"q 70 54 154 104 re W n 1 0 0 1 38 34 cm /Fm1 Do Q "
        b"q 0.05 0.08 0.12 RG 1.2 w 70 54 154 104 re S Q "
        b"BT /F1 10 Tf 24 190 Td (Clipped transparency group) Tj ET"
    )
    form = (
        b"q /GSHalf gs 0.10 0.34 0.78 rg 0 0 226 150 re f Q "
        b"q /GSHalf gs 0.94 0.28 0.10 rg 80 18 112 112 re f Q "
        b"q 0.12 0.16 0.22 RG 2 w 8 12 m 210 132 l S Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 320 220] "
        "/Resources << /Font << /F1 5 0 R >> "
        "/ExtGState << /GSHalf << /ca 0.62 /CA 0.62 >> >> "
        "/XObject << /Fm1 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    form_object = pdf.add(
        b"<< /Type /XObject /Subtype /Form /BBox [0 0 226 150] "
        b"/Group << /S /Transparency /I true /CS /DeviceRGB >> /Length "
        + str(len(form)).encode("ascii")
        + b" >>\nstream\n"
        + form
        + b"\nendstream"
    )
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert form_object == 4
    assert font == 5
    return pdf.render(catalog)


def office_vector_repeated_effects_pdf() -> bytes:
    pdf = Pdf()
    ops: list[str] = [
        "q 0.98 0.98 0.96 rg 0 0 360 240 re f Q",
        "q /GSLow gs",
    ]
    colors = [(0.12, 0.34, 0.78), (0.86, 0.32, 0.16), (0.18, 0.58, 0.40)]
    for row in range(3):
        for col in range(5):
            x = 28 + col * 62
            y = 44 + row * 48
            r, g, b = colors[(row + col) % len(colors)]
            ops.append(f"{r} {g} {b} rg {x} {y} 44 28 re f")
            ops.append(f"0.08 0.10 0.14 RG 0.8 w {x} {y} 44 28 re S")
    ops.extend(
        [
            "Q",
            "q 0.10 0.12 0.16 RG 1.4 w [4 3] 0 d",
            "24 192 m 336 192 l 24 28 m 336 28 l S Q",
            "BT /F1 10 Tf 26 212 Td (Repeated decorative vector effects) Tj ET",
        ]
    )
    content = " ".join(ops).encode("ascii")
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 360 240] "
        "/Resources << /Font << /F1 4 0 R >> "
        "/ExtGState << /GSLow << /ca 0.58 /CA 0.72 >> >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def legal_contract_signature_blocks_pdf() -> bytes:
    return page_pdf(
        "[0 0 320 420]",
        (
            "q 1 1 1 rg 0 0 320 420 re f Q "
            "q 0.12 0.12 0.12 RG 0.7 w 32 72 256 276 re S "
            "32 318 m 288 318 l S 32 136 m 288 136 l S Q "
            "q 0.10 0.10 0.10 RG 1.2 w 54 96 m 132 108 l 204 92 l 260 104 l S Q "
            "q 0.75 0.05 0.04 RG 1.5 w 220 294 44 24 re S Q "
            "BT /F1 12 Tf 52 382 Td (Mutual Services Agreement) Tj "
            "/F1 8 Tf 52 296 Td (1. Services. Provider will perform the services in Exhibit A.) Tj "
            "0 -18 Td (2. Term. This agreement starts on the effective date.) Tj "
            "0 -18 Td (3. Confidentiality. Each party protects confidential information.) Tj "
            "0 -18 Td (4. Notices. Written notices are delivered to the addresses above.) Tj "
            "0 -118 Td (Authorized Signature) Tj 150 0 Td (Reviewed Stamp) Tj ET"
        ),
    )


def legal_visible_redactions_pdf() -> bytes:
    return page_pdf(
        "[0 0 300 380]",
        (
            "q 1 1 1 rg 0 0 300 380 re f Q "
            "BT /F1 10 Tf 36 338 Td (Declaration In Support Of Motion) Tj "
            "/F1 8 Tf 36 300 Td (Party name: Example Holdings LLC) Tj "
            "0 -24 Td (Account number: 1234-5678-9000) Tj "
            "0 -24 Td (Personal identifier: 555-44-3333) Tj "
            "0 -24 Td (Address: 100 Main Street, Example City) Tj "
            "0 -24 Td (Exhibit reference: Confidential attachment A) Tj ET "
            "q 0 0 0 rg 110 270 126 13 re f 132 246 88 13 re f 86 222 160 13 re f Q "
            "BT /F1 7 Tf 36 54 Td (Redaction rectangles are visible page content; semantic redaction is not validated.) Tj ET"
        ),
    )


def legal_filing_stamp_comments_pdf() -> bytes:
    return page_pdf(
        "[0 0 320 400]",
        (
            "q 0.98 0.98 0.96 rg 0 0 320 400 re f Q "
            "q 0.12 0.12 0.12 RG 0.7 w 34 64 252 274 re S 34 304 m 286 304 l S Q "
            "q 1 0.92 0.28 rg 48 244 176 18 re f Q "
            "q 0.76 0.04 0.03 RG 1.5 w 214 318 48 28 re S Q "
            "q 0.15 0.24 0.48 rg 232 122 34 24 re f Q "
            "BT /F1 12 Tf 48 362 Td (Court Filing Packet) Tj "
            "/F1 8 Tf 48 286 Td (Motion for administrative review and supporting memorandum.) Tj "
            "0 -38 Td (Highlighted clause remains visible in the thumbnail.) Tj "
            "0 -76 Td (Comment marker) Tj 150 198 Td (FILED) Tj ET"
        ),
    )


def legal_scanned_attachment_packet_pdf() -> bytes:
    pdf = Pdf()
    scan_width = 160
    scan_height = 220
    scan = bytes(218 + ((x * 2 + y * 3) % 28) for y in range(scan_height) for x in range(scan_width))
    scan_compressed = zlib.compress(scan)
    content_1 = (
        b"q 1 1 1 rg 0 0 260 340 re f Q "
        b"q 0.1 0.1 0.1 RG 0.7 w 28 52 204 224 re S Q "
        b"BT /F1 10 Tf 42 298 Td (Attachment Index) Tj "
        b"/F1 8 Tf 42 250 Td (Attachment A: scanned exhibit with signature.) Tj "
        b"0 -20 Td (Attachment B: visible redaction sample.) Tj ET"
    )
    content_2 = (
        b"q 220 0 0 300 20 20 cm /Scan Do Q "
        b"q 0 0 0 rg 84 198 96 14 re f Q "
        b"BT /F1 8 Tf 42 306 Td (Scanned Exhibit) Tj ET"
    )
    contents_1 = pdf.add(
        f"<< /Length {len(content_1)} >>\nstream\n".encode("ascii")
        + content_1
        + b"\nendstream"
    )
    contents_2 = pdf.add(
        f"<< /Length {len(content_2)} >>\nstream\n".encode("ascii")
        + content_2
        + b"\nendstream"
    )
    page_1 = pdf.add(
        "<< /Type /Page /Parent 6 0 R /MediaBox [0 0 260 340] "
        "/Resources << /Font << /F1 5 0 R >> >> "
        f"/Contents {contents_1} 0 R >>"
    )
    page_2 = pdf.add(
        "<< /Type /Page /Parent 6 0 R /MediaBox [0 0 260 340] "
        "/Resources << /Font << /F1 5 0 R >> /XObject << /Scan 7 0 R >> >> "
        f"/Contents {contents_2} 0 R >>"
    )
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    pages = pdf.add(f"<< /Type /Pages /Kids [{page_1} 0 R {page_2} 0 R] /Count 2 >>")
    scan_object = pdf.add(
        (
            f"<< /Type /XObject /Subtype /Image /Width {scan_width} /Height {scan_height} "
            f"/ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode "
            f"/Length {len(scan_compressed)} >>\nstream\n"
        ).encode("ascii")
        + scan_compressed
        + b"\nendstream"
    )
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 5
    assert pages == 6
    assert scan_object == 7
    return pdf.render(catalog)


def slide_title_gradient_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"/Bg sh "
        b"q /Shadow gs 0 0 0 rg 34 40 220 72 re f Q "
        b"q 1 1 1 rg 28 48 220 72 re f Q "
        b"q 0.10 0.18 0.38 rg 28 132 44 14 re f Q "
        b"BT /F1 24 Tf 42 96 Td (Quarterly Review) Tj "
        b"/F1 12 Tf 0 -24 Td (Native slide export fixture) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 320 180] "
        "/Resources << "
        "/Font << /F1 4 0 R >> "
        "/ExtGState << /Shadow << /ca 0.22 >> >> "
        "/Shading << /Bg << /ShadingType 2 /ColorSpace /DeviceRGB "
        "/Coords [0 0 320 180] "
        "/Function << /FunctionType 2 /Domain [0 1] /C0 [0.08 0.18 0.34] /C1 [0.95 0.54 0.18] /N 1 >> "
        "/Extend [true true] >> >> "
        ">> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def slide_layered_image_shadow_pdf() -> bytes:
    pdf = Pdf()
    image = bytes(
        [
            22,
            90,
            150,
            76,
            130,
            190,
            160,
            190,
            210,
            245,
            210,
            130,
            48,
            120,
            170,
            104,
            164,
            205,
            205,
            218,
            230,
            255,
            226,
            156,
            72,
            144,
            182,
            128,
            180,
            210,
            222,
            232,
            238,
            255,
            238,
            188,
            42,
            105,
            160,
            96,
            150,
            198,
            184,
            210,
            228,
            250,
            220,
            170,
        ]
    )
    content = (
        b"q 0.96 0.97 0.98 rg 0 0 320 180 re f Q "
        b"q /Shadow gs 0 0 0 rg 92 36 154 98 re f Q "
        b"q 150 0 0 96 84 42 cm /Hero Do Q "
        b"q /Tint gs 0.04 0.28 0.55 rg 84 42 150 96 re f Q "
        b"q 0.95 0.38 0.10 rg 214 102 42 28 re f Q "
        b"BT /F1 18 Tf 28 144 Td (Layered image slide) Tj "
        b"/F1 11 Tf 0 -18 Td (image, tint overlay, shadow) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 320 180] "
        "/Resources << /Font << /F1 5 0 R >> "
        "/ExtGState << /Shadow << /ca 0.18 >> /Tint << /ca 0.18 >> >> "
        "/XObject << /Hero 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 4 /Height 4 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    assert font == 5
    return pdf.render(catalog)


def slide_rotated_callout_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.08 0.10 0.16 rg 0 0 320 180 re f Q "
        b"q 0.18 0.62 0.76 rg 36 36 42 66 re f 94 36 42 92 re f 152 36 42 48 re f Q "
        b"q /Panel gs 1 1 1 rg 188 34 98 92 re f Q "
        b"q 0.95 0.44 0.16 RG 2 w 188 34 98 92 re S Q "
        b"BT /F1 22 Tf 28 146 Td (Metrics) Tj /F1 11 Tf 0 -18 Td (rotated callout) Tj ET "
        b"q 0.866 0.5 -0.5 0.866 214 58 cm BT /F1 14 Tf 0 0 Td (Growth +24%) Tj ET Q"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 320 180] "
        "/Resources << /Font << /F1 4 0 R >> /ExtGState << /Panel << /ca 0.88 >> >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def slide_speaker_notes_page_pdf() -> bytes:
    return page_pdf(
        "[0 0 240 320]",
        (
            "q 0.98 0.98 0.96 rg 0 0 240 320 re f Q "
            "q 0.12 0.20 0.34 rg 24 184 192 108 re f Q "
            "q 0.96 0.48 0.16 rg 42 244 54 18 re f 42 218 92 12 re f Q "
            "q 0.72 0.72 0.72 RG 0.6 w 24 160 m 216 160 l "
            "24 136 m 216 136 l 24 112 m 216 112 l 24 88 m 216 88 l "
            "24 64 m 216 64 l 24 40 m 216 40 l S Q "
            "BT /F1 12 Tf 24 300 Td (Speaker Notes) Tj "
            "0 -154 Td (Key talking points) Tj "
            "0 -24 Td (1. Revenue bridge) Tj "
            "0 -24 Td (2. Product adoption) Tj "
            "0 -24 Td (3. Follow-up owners) Tj ET"
        ),
    )


def spreadsheet_frozen_header_pdf() -> bytes:
    lines: list[str] = [
        "q 0.99 0.99 0.97 rg 0 0 320 200 re f Q",
        "q 0.18 0.28 0.42 rg 24 152 272 24 re f Q",
        "q 0.86 0.92 0.98 rg 24 32 46 120 re f Q",
        "q 0.08 0.12 0.18 RG 0.5 w",
    ]
    for x in range(24, 297, 34):
        lines.append(f"{x} 32 m {x} 176 l S")
    for y in range(32, 177, 12):
        lines.append(f"24 {y} m 296 {y} l S")
    lines.append("0.95 0.42 0.12 RG 1.5 w 70 32 m 70 176 l S 24 152 m 296 152 l S Q")
    lines.append("BT /F1 7 Tf 30 162 Td (Region) Tj 48 0 Td (Q1) Tj 34 0 Td (Q2) Tj 34 0 Td (Q3) Tj 34 0 Td (Q4) Tj ET")
    for row in range(8):
        y = 140 - row * 12
        lines.append(
            f"BT /F1 6 Tf 30 {y} Td (R{row + 1}) Tj 48 0 Td ({120 + row}) Tj "
            f"34 0 Td ({132 + row}) Tj 34 0 Td ({141 + row}) Tj 34 0 Td ({155 + row}) Tj ET"
        )
    return page_pdf("[0 0 320 200]", " ".join(lines))


def spreadsheet_dense_numeric_grid_pdf() -> bytes:
    lines: list[str] = [
        "q 1 1 1 rg 0 0 320 220 re f Q",
        "q 0.92 0.95 0.98 rg 18 184 284 18 re f Q",
        "q 0.34 0.34 0.34 RG 0.35 w",
    ]
    for x in range(18, 303, 28):
        lines.append(f"{x} 26 m {x} 202 l S")
    for y in range(26, 203, 10):
        lines.append(f"18 {y} m 302 {y} l S")
    lines.append("Q")
    lines.append("BT /F1 5 Tf 22 191 Td (Dense spreadsheet export) Tj ET")
    for row in range(14):
        y = 174 - row * 10
        cells = [
            f"BT /F1 4 Tf 22 {y} Td ({row + 1:02d}) Tj",
            f"26 0 Td ({1000 + row * 7}) Tj",
            f"28 0 Td ({2000 + row * 9}) Tj",
            f"28 0 Td ({3000 + row * 11}) Tj",
            f"28 0 Td ({4000 + row * 13}) Tj",
            f"28 0 Td ({5000 + row * 17}) Tj",
            "ET",
        ]
        lines.append(" ".join(cells))
    return page_pdf("[0 0 320 220]", " ".join(lines))


def spreadsheet_clipped_cells_pdf() -> bytes:
    lines: list[str] = [
        "q 0.97 0.98 1 rg 0 0 260 180 re f Q",
        "q 0.16 0.20 0.30 rg 20 138 220 20 re f Q",
        "q 0.18 0.20 0.24 RG 0.5 w",
    ]
    for x in range(20, 241, 44):
        lines.append(f"{x} 38 m {x} 158 l S")
    for y in range(38, 159, 20):
        lines.append(f"20 {y} m 240 {y} l S")
    lines.append("Q")
    for row in range(5):
        for col in range(5):
            x = 24 + col * 44
            y = 122 - row * 20
            lines.append(
                f"q {x} {y} 36 12 re W n BT /F1 6 Tf {x} {y + 3} Td "
                f"(Cell {row + 1}-{col + 1} overflow text) Tj ET Q"
            )
    return page_pdf("[0 0 260 180]", " ".join(lines))


def spreadsheet_vector_stress_grid_pdf() -> bytes:
    lines: list[str] = [
        "q 0.98 0.98 0.98 rg 0 0 360 240 re f Q",
        "q 0.25 0.25 0.25 RG 0.35 w",
    ]
    for x in range(18, 343, 13):
        lines.append(f"{x} 24 m {x} 218 l S")
    for y in range(24, 219, 8):
        lines.append(f"18 {y} m 342 {y} l S")
    lines.append("Q")
    lines.append("q 0.10 0.28 0.52 rg 18 202 324 16 re f Q")
    for row in range(18):
        y = 190 - row * 8
        lines.append(
            f"BT /F1 4 Tf 22 {y} Td ({row:02d}) Tj 39 0 Td ({row * 17 + 3}) Tj "
            f"52 0 Td ({row * 19 + 5}) Tj 52 0 Td ({row * 23 + 7}) Tj "
            f"52 0 Td ({row * 29 + 11}) Tj ET"
        )
    return page_pdf("[0 0 360 240]", " ".join(lines))


def technical_linework_dimensions_pdf() -> bytes:
    ops: list[str] = [
        "q 0.99 0.99 0.97 rg 0 0 360 240 re f Q",
        "q 0.05 0.08 0.12 RG 0.45 w",
        "40 40 m 320 40 l 320 190 l 40 190 l h S",
        "70 70 m 150 70 l 150 130 l 70 130 l h S",
        "210 70 m 290 70 l 290 130 l 210 130 l h S",
        "40 205 m 320 205 l S 40 198 m 40 212 l S 320 198 m 320 212 l S",
        "330 40 m 330 190 l S 323 40 m 337 40 l S 323 190 m 337 190 l S",
        "Q",
        "q 0.72 0.72 0.72 RG 0.35 w [4 3] 0 d",
    ]
    for x in range(60, 321, 20):
        ops.append(f"{x} 36 m {x} 194 l S")
    for y in range(60, 191, 20):
        ops.append(f"36 {y} m 324 {y} l S")
    ops.extend(
        [
            "Q",
            "BT /F1 7 Tf 154 210 Td (2800 mm) Tj 334 112 Td (1500 mm) Tj ET",
            "BT /F1 6 Tf 72 134 Td (ROOM A) Tj 214 134 Td (ROOM B) Tj ET",
        ]
    )
    return page_pdf("[0 0 360 240]", " ".join(ops))


def technical_hatch_clipping_pdf() -> bytes:
    ops: list[str] = [
        "q 0.98 0.99 1 rg 0 0 300 220 re f Q",
        "q 0.05 0.08 0.12 RG 0.6 w 42 42 m 258 42 l 258 178 l 42 178 l h S Q",
        "q 42 42 m 258 42 l 232 178 l 68 178 l h W n",
        "0.28 0.28 0.28 RG 0.35 w",
    ]
    for offset in range(-120, 310, 10):
        ops.append(f"{offset} 28 m {offset + 220} 202 l S")
    ops.extend(
        [
            "Q",
            "q 0.06 0.20 0.36 RG 0.8 w 90 76 m 210 76 l 210 146 l 90 146 l h S Q",
            "q 0.85 0.30 0.12 RG 1.0 w [8 4] 0 d 118 104 m 184 104 l S Q",
            "BT /F1 7 Tf 94 154 Td (SECTION A-A) Tj ET",
        ]
    )
    return page_pdf("[0 0 300 220]", " ".join(ops))


def technical_large_coordinate_plan_pdf() -> bytes:
    ops: list[str] = [
        "q 0.99 0.99 0.99 rg 0 0 2000 1200 re f Q",
        "q 0.04 0.07 0.10 RG 3 w",
        "160 160 m 1840 160 l 1840 980 l 160 980 l h S",
        "320 300 m 760 300 l 760 780 l 320 780 l h S",
        "1040 300 m 1680 300 l 1680 780 l 1040 780 l h S",
        "Q",
        "q 0.70 0.70 0.70 RG 1.2 w [20 12] 0 d",
    ]
    for x in range(240, 1841, 160):
        ops.append(f"{x} 140 m {x} 1000 l S")
    for y in range(240, 981, 120):
        ops.append(f"140 {y} m 1860 {y} l S")
    ops.extend(
        [
            "Q",
            "q 0.80 0.22 0.10 RG 4 w 160 1040 m 1840 1040 l S 160 1020 m 160 1060 l S 1840 1020 m 1840 1060 l S Q",
            "BT /F1 48 Tf 820 1080 Td (SITE PLAN 1:200) Tj 820 -80 Td (168m) Tj ET",
        ]
    )
    return page_pdf("[0 0 2000 1200]", " ".join(ops))


def technical_repeated_symbols_pdf() -> bytes:
    ops: list[str] = [
        "q 0.97 0.97 0.95 rg 0 0 320 220 re f Q",
        "q 0.18 0.18 0.18 RG 0.45 w",
    ]
    for x in range(28, 293, 44):
        for y in range(42, 183, 35):
            ops.append(
                f"{x} {y} m {x + 16} {y + 24} l {x + 32} {y} l h S "
                f"{x + 16} {y + 6} m {x + 16} {y + 20} l S "
                f"{x + 8} {y + 6} m {x + 24} {y + 6} l S"
            )
    ops.extend(
        [
            "Q",
            "q 0.04 0.18 0.34 RG 0.8 w 20 30 m 300 30 l 300 196 l 20 196 l h S Q",
            "BT /F1 7 Tf 24 202 Td (SYMBOL LAYOUT) Tj 224 0 Td (REV B) Tj ET",
        ]
    )
    return page_pdf("[0 0 320 220]", " ".join(ops))


def engineering_floorplan_precision_pdf() -> bytes:
    ops: list[str] = [
        "q 0.99 0.99 0.98 rg 0 0 420 300 re f Q",
        "q 0.78 0.78 0.78 RG 0.35 w [5 3] 0 d",
    ]
    for x in range(40, 381, 20):
        ops.append(f"{x} 38 m {x} 262 l S")
    for y in range(40, 261, 20):
        ops.append(f"38 {y} m 382 {y} l S")
    ops.extend(
        [
            "Q",
            "q 0.04 0.07 0.10 RG 0.55 w",
            "60 60 m 360 60 l 360 240 l 60 240 l h S",
            "60 150 m 182 150 l 238 150 m 360 150 l S",
            "210 60 m 210 118 l 210 182 m 210 240 l S",
            "90 90 m 160 90 l 160 126 l 90 126 l h S",
            "254 90 m 330 90 l 330 126 l 254 126 l h S",
            "Q",
            "q 0.75 0.18 0.10 RG 0.8 w [10 5] 0 d 60 260 m 360 260 l S Q",
            "BT /F1 7 Tf 166 266 Td (30.00 m) Tj 368 148 Td (18.00 m) Tj ET",
            "BT /F1 6 Tf 92 132 Td (LAB A) Tj 256 132 Td (LAB B) Tj 214 152 Td (DOOR) Tj ET",
        ]
    )
    return page_pdf("[0 0 420 300]", " ".join(ops))


def engineering_schematic_symbols_pdf() -> bytes:
    ops: list[str] = [
        "q 0.99 1 0.99 rg 0 0 360 240 re f Q",
        "q 0.05 0.07 0.10 RG 0.6 w",
        "34 120 m 86 120 l S 114 120 m 166 120 l S 194 120 m 246 120 l S 274 120 m 326 120 l S",
        "86 104 m 86 136 l 114 136 l 114 104 l h S",
        "166 104 m 194 136 l 194 104 l h S",
        "246 104 m 274 104 l 274 136 l 246 136 l h S",
        "54 76 m 54 52 l 306 52 l 306 76 l S",
        "Q",
        "q 0.05 0.07 0.10 RG 0.4 w",
    ]
    for x in range(44, 321, 28):
        ops.append(f"{x} 176 m {x + 10} 194 l {x + 20} 176 l h S")
    ops.extend(
        [
            "Q",
            "q 0.10 0.28 0.48 RG 0.7 w [6 4] 0 d 34 176 m 326 176 l S Q",
            "BT /F1 8 Tf 28 212 Td (Schematic Symbol Sheet) Tj 236 0 Td (REV C) Tj ET",
            "BT /F1 6 Tf 82 142 Td (R1) Tj 168 142 Td (D1) Tj 246 142 Td (U2) Tj 44 42 Td (RETURN BUS) Tj ET",
        ]
    )
    return page_pdf("[0 0 360 240]", " ".join(ops))


def engineering_large_transform_detail_pdf() -> bytes:
    ops: list[str] = [
        "q 1 1 1 rg 0 0 2400 1600 re f Q",
        "q 0.05 0.08 0.11 RG 2.4 w",
        "180 180 m 2220 180 l 2220 1360 l 180 1360 l h S",
        "420 360 m 1060 360 l 1060 930 l 420 930 l h S",
        "1320 360 m 1980 360 l 1980 930 l 1320 930 l h S",
        "Q",
        "q 0.70 0.70 0.70 RG 0.9 w [24 18] 0 d",
    ]
    for x in range(300, 2221, 160):
        ops.append(f"{x} 160 m {x} 1380 l S")
    for y in range(280, 1361, 140):
        ops.append(f"160 {y} m 2240 {y} l S")
    ops.extend(
        [
            "Q",
            "q 0.80 0.20 0.10 RG 4 w 180 1440 m 2220 1440 l S 180 1418 m 180 1462 l S 2220 1418 m 2220 1462 l S Q",
            "q 0.04 0.24 0.34 RG 2 w 1180 520 m 1220 620 l 1260 520 l h S 1180 760 m 1260 760 l S Q",
            "BT /F1 44 Tf 880 1488 Td (LARGE COORDINATE DETAIL) Tj 980 -92 Td (204m) Tj ET",
        ]
    )
    return page_pdf("[0 0 2400 1600]", " ".join(ops))


def chart_combo_legend_pdf() -> bytes:
    ops: list[str] = [
        "q 0.99 0.99 0.98 rg 0 0 360 240 re f Q",
        "q 0.16 0.18 0.22 RG 0.7 w 44 44 m 44 194 l 320 44 l S Q",
        "q 0.18 0.42 0.72 rg 68 44 28 74 re f 118 44 28 110 re f 168 44 28 86 re f 218 44 28 132 re f Q",
        "q 0.88 0.42 0.12 RG 2 w 68 94 m 118 126 l 168 112 l 218 156 l 268 140 l S Q",
        "q 0.12 0.12 0.12 rg 278 150 10 10 re f 278 128 10 10 re f Q",
        "q 0.18 0.42 0.72 rg 280 152 6 6 re f Q",
        "q 0.88 0.42 0.12 RG 1.5 w 279 131 m 287 135 l S Q",
        "BT /F1 12 Tf 48 210 Td (Revenue dashboard) Tj ET",
        "BT /F1 7 Tf 294 151 Td (Bars) Tj 294 -22 Td (Trend) Tj ET",
        "BT /F1 6 Tf 62 32 Td (Q1) Tj 50 0 Td (Q2) Tj 50 0 Td (Q3) Tj 50 0 Td (Q4) Tj ET",
    ]
    return page_pdf("[0 0 360 240]", " ".join(ops))


def dashboard_kpi_panels_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.95 0.97 0.98 rg 0 0 360 220 re f Q "
        b"q 0.12 0.16 0.24 rg 0 184 360 36 re f Q "
        b"q 1 1 1 rg 24 110 92 54 re f 134 110 92 54 re f 244 110 92 54 re f "
        b"24 28 146 60 re f 190 28 146 60 re f Q "
        b"q /Overlay gs 0.12 0.46 0.74 rg 24 110 92 18 re f "
        b"0.86 0.38 0.12 rg 134 110 92 18 re f "
        b"0.18 0.56 0.34 rg 244 110 92 18 re f Q "
        b"q 0.18 0.20 0.24 RG 0.6 w 24 28 146 60 re S 190 28 146 60 re S "
        b"34 52 m 58 62 l 82 48 l 106 70 l 136 58 l S "
        b"202 42 m 322 42 l 202 56 m 322 56 l 202 70 m 322 70 l S Q "
        b"BT /F1 13 Tf 24 198 Td (Operations dashboard) Tj "
        b"/F1 8 Tf 34 146 Td (Revenue) Tj 110 0 Td (Margin) Tj 110 0 Td (Orders) Tj "
        b"/F1 16 Tf -220 -24 Td (128k) Tj 110 0 Td (34%) Tj 110 0 Td (912) Tj "
        b"/F1 8 Tf -220 -62 Td (Sparkline) Tj 166 0 Td (Heat table) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 360 220] "
        "/Resources << /Font << /F1 4 0 R >> "
        "/ExtGState << /Overlay << /ca 0.74 /CA 0.74 >> >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def map_marker_clusters_pdf() -> bytes:
    ops: list[str] = [
        "q 0.92 0.96 0.91 rg 0 0 360 240 re f Q",
        "q 0.78 0.88 0.96 rg 0 92 360 38 re f Q",
        "q 0.76 0.84 0.70 rg 30 34 94 62 re f 220 138 94 54 re f Q",
        "q 0.62 0.70 0.60 RG 0.7 w [5 3] 0 d 30 32 m 328 206 l S 52 198 m 310 42 l S Q",
        "q 0.82 0.24 0.14 rg",
    ]
    for cluster_x, cluster_y in [(88, 78), (174, 128), (268, 164)]:
        for dx, dy in [(-14, -8), (0, 0), (16, 7), (-6, 13), (12, -14)]:
            x = cluster_x + dx
            y = cluster_y + dy
            ops.append(f"{x} {y} 7 7 re f")
    ops.extend(
        [
            "Q",
            "q 0.10 0.20 0.32 RG 1.0 w 40 28 m 322 28 l 322 212 l 40 212 l h S Q",
            "BT /F1 9 Tf 44 218 Td (Depot coverage map) Tj ET",
            "BT /F1 6 Tf 76 94 Td (North) Tj 162 144 Td (Central) Tj 254 180 Td (East) Tj ET",
        ]
    )
    return page_pdf("[0 0 360 240]", " ".join(ops))


def map_raster_tile_routes_pdf() -> bytes:
    pdf = Pdf()
    tile = bytes(
        [
            224,
            238,
            216,
            196,
            224,
            238,
            214,
            226,
            206,
            236,
            232,
            194,
        ]
        * 4
    )
    content = (
        b"q 0.94 0.97 0.92 rg 0 0 420 300 re f Q "
        b"q 180 0 0 120 28 58 cm /Tile Do Q "
        b"q 180 0 0 120 208 58 cm /Tile Do Q "
        b"q 0.40 0.55 0.42 RG 0.7 w [6 4] 0 d 32 56 m 388 56 l 32 116 m 388 116 l "
        b"32 176 m 388 176 l 32 236 m 388 236 l 64 42 m 64 258 l 144 42 m 144 258 l "
        b"224 42 m 224 258 l 304 42 m 304 258 l 384 42 m 384 258 l S Q "
        b"q 0.12 0.28 0.52 RG 2.2 w 42 74 m 118 126 l 196 112 l 278 184 l 360 206 l S Q "
        b"q 0.86 0.22 0.10 rg 96 122 9 9 re f 190 108 9 9 re f 272 180 9 9 re f 354 202 9 9 re f Q "
        b"BT /F1 10 Tf 34 270 Td (Raster tile route map) Tj "
        b"/F1 6 Tf 96 136 Td (A1) Tj 190 122 Td (B2) Tj 272 194 Td (C3) Tj 330 -168 Td (Legend) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 420 300] "
        "/Resources << /Font << /F1 5 0 R >> /XObject << /Tile 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 4 /Height 4 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length "
        + str(len(tile)).encode("ascii")
        + b" >>\nstream\n"
        + tile
        + b"\nendstream"
    )
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert image_object == 4
    assert font == 5
    return pdf.render(catalog)


def map_transparent_zoning_overlay_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.91 0.95 0.90 rg 0 0 380 260 re f Q "
        b"q 0.72 0.82 0.90 rg 0 96 380 42 re f Q "
        b"q 0.36 0.48 0.34 RG 0.7 w 28 46 m 348 46 l 348 218 l 28 218 l h S "
        b"60 46 m 60 218 l S 112 46 m 112 218 l S 164 46 m 164 218 l S "
        b"216 46 m 216 218 l S 268 46 m 268 218 l S 320 46 m 320 218 l S Q "
        b"q /Zone gs 0.90 0.20 0.10 rg 54 74 118 92 re f Q "
        b"q /Zone gs 0.10 0.42 0.82 rg 188 84 118 106 re f Q "
        b"q 0.10 0.20 0.32 RG 1.2 w [7 4] 0 d 38 68 m 328 206 l S Q "
        b"BT /F1 10 Tf 30 232 Td (Transparent zoning overlay) Tj "
        b"/F1 6 Tf 72 172 Td (Zone A) Tj 214 184 Td (Zone B) Tj 42 58 Td (Route + grid remain visible) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 380 260] "
        "/Resources << /Font << /F1 4 0 R >> /ExtGState << /Zone << /ca 0.32 >> >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def map_optional_layer_policy_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.92 0.96 0.91 rg 0 0 360 240 re f Q "
        b"q 0.68 0.80 0.92 rg 0 88 360 46 re f Q "
        b"q 0.28 0.42 0.28 RG 0.7 w 28 38 m 332 38 l 332 208 l 28 208 l h S "
        b"48 68 m 310 190 l 72 188 m 304 62 l S Q "
        b"/OC /Zoning BDC q 0.95 0.25 0.10 rg 96 76 82 62 re f 204 128 72 52 re f Q EMC "
        b"BT /F1 10 Tf 34 218 Td (Optional zoning layer map) Tj "
        b"/F1 6 Tf 56 54 Td (Layer OFF by default; base map remains deterministic.) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 360 240] "
        "/Resources << /Font << /F1 5 0 R >> /Properties << /Zoning 4 0 R >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    layer = pdf.add("<< /Type /OCG /Name (Zoning Overlay) >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R "
        f"/OCProperties << /OCGs [{layer} 0 R] /D << /BaseState /ON /OFF [{layer} 0 R] >> >> >>"
    )
    assert layer == 4
    assert font == 5
    return pdf.render(catalog)


def dashboard_heatmap_overlay_pdf() -> bytes:
    pdf = Pdf()
    ops: list[str] = [
        "q 0.98 0.98 0.96 rg 0 0 340 220 re f Q",
        "q 0.16 0.18 0.23 rg 0 184 340 36 re f Q",
    ]
    palette = [
        "0.86 0.94 0.78",
        "0.58 0.78 0.54",
        "0.96 0.74 0.32",
        "0.86 0.32 0.22",
    ]
    for row in range(6):
        for col in range(10):
            color = palette[(row * 3 + col * 2) % len(palette)]
            x = 28 + col * 28
            y = 54 + row * 18
            ops.append(f"q {color} rg {x} {y} 24 14 re f Q")
    ops.extend(
        [
            "q /Overlay gs 0.10 0.28 0.50 rg 28 54 276 108 re f Q",
            "q 0.12 0.14 0.18 RG 0.45 w",
        ]
    )
    for x in range(28, 305, 28):
        ops.append(f"{x} 54 m {x} 162 l S")
    for y in range(54, 163, 18):
        ops.append(f"28 {y} m 304 {y} l S")
    ops.extend(
        [
            "Q",
            "BT /F1 12 Tf 24 198 Td (SLA heatmap) Tj ET",
            "BT /F1 6 Tf 28 36 Td (Mon) Tj 56 0 Td (Tue) Tj 56 0 Td (Wed) Tj 56 0 Td (Thu) Tj 56 0 Td (Fri) Tj ET",
        ]
    )
    content = " ".join(ops).encode("ascii")
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 340 220] "
        "/Resources << /Font << /F1 4 0 R >> "
        "/ExtGState << /Overlay << /ca 0.30 /CA 0.30 >> >> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def scientific_two_column_paper_pdf() -> bytes:
    ops: list[str] = [
        "q 1 1 1 rg 0 0 360 480 re f Q",
        "q 0.10 0.12 0.16 RG 0.7 w 28 90 m 332 90 l 28 420 m 332 420 l S Q",
        "BT /F1 15 Tf 52 444 Td (A Small Native Rendering Study) Tj ET",
        "BT /F1 7 Tf 74 430 Td (A Example, B Reviewer) Tj ET",
        "BT /F1 7 Tf 32 402 Td (Abstract) Tj ET",
    ]
    for row in range(16):
        y = 386 - row * 12
        width = 118 if row % 4 else 92
        ops.append(f"q 0.18 0.18 0.18 rg 32 {y} {width} 2 re f Q")
        ops.append(f"q 0.18 0.18 0.18 rg 190 {y} {width + 16} 2 re f Q")
    ops.extend(
        [
            "q 0.90 0.94 0.98 rg 190 206 118 72 re f Q",
            "q 0.12 0.24 0.40 RG 0.8 w 202 224 m 222 248 l 246 232 l 272 258 l 296 224 l S Q",
            "BT /F1 7 Tf 200 286 Td (Figure 1. Pipeline) Tj ET",
            "BT /F1 10 Tf 44 122 Td (E = mc2 + alpha) Tj ET",
            "BT /F1 6 Tf 32 72 Td (1 Footnote text remains readable at thumbnail scale.) Tj ET",
        ]
    )
    return page_pdf("[0 0 360 480]", " ".join(ops))


def scientific_equation_figure_pdf() -> bytes:
    ops: list[str] = [
        "q 0.99 0.99 1 rg 0 0 320 240 re f Q",
        "BT /F1 13 Tf 28 212 Td (Equation and Figure Page) Tj ET",
        "q 0.12 0.12 0.16 RG 0.7 w 28 40 124 128 re S 178 40 114 128 re S Q",
        "q 0.16 0.36 0.64 RG 1.1 w 44 68 m 76 136 l 112 84 l 140 150 l S Q",
        "q 0.82 0.34 0.16 rg 200 72 16 16 re f 228 94 16 16 re f 256 126 16 16 re f Q",
        "BT /F1 10 Tf 44 184 Td (sigma x_i / n = mu) Tj ET",
        "BT /F1 8 Tf 48 154 Td (a2 + b2 = c2) Tj 0 -18 Td (integral f dx) Tj ET",
        "BT /F1 7 Tf 184 174 Td (Observed groups) Tj 12 -120 Td (n=42) Tj ET",
    ]
    return page_pdf("[0 0 320 240]", " ".join(ops))


def reference_footnote_layout_pdf() -> bytes:
    ops: list[str] = [
        "q 1 1 1 rg 0 0 320 260 re f Q",
        "BT /F1 13 Tf 28 230 Td (References and Notes) Tj ET",
        "q 0.18 0.18 0.18 RG 0.45 w 28 82 m 292 82 l S Q",
    ]
    for row in range(9):
        y = 204 - row * 13
        ops.append(f"q 0.15 0.15 0.15 rg 32 {y} {210 - (row % 3) * 24} 2 re f Q")
    for row in range(5):
        y = 64 - row * 10
        ops.append(f"q 0.28 0.28 0.28 rg 34 {y} {238 - row * 18} 1.5 re f Q")
    ops.extend(
        [
            "BT /F1 7 Tf 34 74 Td (1 Notes use smaller text below the rule.) Tj ET",
            "BT /F1 6 Tf 34 20 Td ([1] Example reference title. Journal 2026.) Tj ET",
        ]
    )
    return page_pdf("[0 0 320 260]", " ".join(ops))


def academic_publisher_first_page_pdf() -> bytes:
    ops: list[str] = [
        "q 1 1 1 rg 0 0 360 480 re f Q",
        "q 0.06 0.10 0.18 rg 0 438 360 42 re f Q",
        "q 0.11 0.18 0.28 RG 0.6 w 30 92 m 330 92 l 30 384 m 330 384 l S Q",
        "BT /F1 13 Tf 38 454 Td (Journal of Synthetic Documents) Tj ET",
        "BT /F1 14 Tf 42 414 Td (Publisher-Style Two Column Article) Tj ET",
        "BT /F1 7 Tf 42 400 Td (DOI 10.0000/pdfrust.0150) Tj 0 -12 Td (Received 2026-06-26) Tj ET",
        "BT /F1 8 Tf 42 374 Td (Abstract) Tj ET",
    ]
    for row in range(17):
        y = 356 - row * 13
        left_width = 124 if row % 5 else 96
        right_width = 118 if row % 4 else 88
        ops.append(f"q 0.18 0.18 0.18 rg 42 {y} {left_width} 2 re f Q")
        ops.append(f"q 0.18 0.18 0.18 rg 194 {y} {right_width} 2 re f Q")
    ops.extend(
        [
            "q 0.91 0.94 0.98 rg 194 190 108 62 re f Q",
            "q 0.10 0.28 0.46 RG 0.8 w 204 204 m 226 236 l 250 214 l 276 242 l 294 206 l S Q",
            "BT /F1 7 Tf 204 260 Td (Figure 2. Publisher panel) Tj ET",
            "BT /F1 6 Tf 42 70 Td (Synthetic publisher article fixture; no real manuscript source.) Tj ET",
        ]
    )
    return page_pdf("[0 0 360 480]", " ".join(ops))


def academic_equation_symbols_page_pdf() -> bytes:
    return page_pdf(
        "[0 0 340 260]",
        (
            "q 0.99 0.99 1 rg 0 0 340 260 re f Q "
            "q 0.12 0.16 0.24 RG 0.7 w 28 44 284 168 re S 28 156 m 312 156 l S "
            "28 100 m 312 100 l S 168 44 m 168 212 l S Q "
            "q 0.14 0.32 0.56 RG 1.0 w 190 68 m 218 132 l 246 94 l 284 166 l S Q "
            "q 0.78 0.30 0.14 rg 204 76 14 14 re f 232 102 14 14 re f 268 146 14 14 re f Q "
            "BT /F1 13 Tf 28 230 Td (Equation Symbols And Figure) Tj "
            "/F1 9 Tf 42 184 Td (sum_i w_i x_i = theta) Tj "
            "0 -22 Td (lim n->inf sample_n = mu) Tj "
            "0 -22 Td (matrix A times vector b) Tj "
            "142 66 Td (Vector figure) Tj "
            "-142 -122 Td (ASCII math proxies track symbol placement without custom fonts.) Tj ET"
        ),
    )


def academic_references_appendix_pdf() -> bytes:
    ops: list[str] = [
        "q 1 1 1 rg 0 0 340 300 re f Q",
        "BT /F1 13 Tf 28 268 Td (References, Appendix, And Footnotes) Tj ET",
        "q 0.18 0.18 0.18 RG 0.5 w 28 92 m 312 92 l 28 238 m 312 238 l S Q",
    ]
    for row in range(10):
        y = 224 - row * 13
        ops.append(f"q 0.16 0.16 0.16 rg 34 {y} {230 - (row % 4) * 22} 2 re f Q")
    for row in range(7):
        y = 76 - row * 9
        ops.append(f"q 0.28 0.28 0.28 rg 36 {y} {250 - row * 18} 1.4 re f Q")
    ops.extend(
        [
            "BT /F1 7 Tf 34 246 Td (References) Tj ET",
            "BT /F1 7 Tf 34 82 Td (Appendix note 1: synthetic citation row.) Tj ET",
            "BT /F1 6 Tf 34 18 Td ([A1] Example appendix source. Synthetic Press, 2026.) Tj ET",
        ]
    )
    return page_pdf("[0 0 340 300]", " ".join(ops))


def long_report_sampling_pdf() -> bytes:
    pdf = Pdf()

    def page_content(title: str, accent: str) -> bytes:
        ops: list[str] = [
            "q 0.98 0.98 0.96 rg 0 0 300 220 re f Q",
            f"q {accent} rg 24 184 252 18 re f Q",
            "q 0.20 0.22 0.26 RG 0.5 w 24 44 252 124 re S 24 144 m 276 144 l S 24 112 m 276 112 l S 24 80 m 276 80 l S Q",
            f"BT /F1 11 Tf 28 198 Td ({title}) Tj ET",
        ]
        for row in range(5):
            y = 154 - row * 22
            ops.append(
                f"BT /F1 7 Tf 30 {y} Td (Section {row + 1}) Tj "
                f"84 0 Td ({100 + row * 13}) Tj 66 0 Td ({200 + row * 17}) Tj ET"
            )
        ops.append("BT /F1 6 Tf 24 24 Td (Footer and page marker) Tj ET")
        return " ".join(ops).encode("ascii")

    content_1 = page_content("Long Report p1", "0.12 0.22 0.36")
    content_2 = page_content("Long Report p2", "0.20 0.42 0.32")
    content_3 = page_content("Long Report p3", "0.74 0.32 0.16")
    contents_1 = pdf.add(
        f"<< /Length {len(content_1)} >>\nstream\n".encode("ascii")
        + content_1
        + b"\nendstream"
    )
    contents_2 = pdf.add(
        f"<< /Length {len(content_2)} >>\nstream\n".encode("ascii")
        + content_2
        + b"\nendstream"
    )
    contents_3 = pdf.add(
        f"<< /Length {len(content_3)} >>\nstream\n".encode("ascii")
        + content_3
        + b"\nendstream"
    )
    page_1 = pdf.add(
        "<< /Type /Page /Parent 7 0 R /MediaBox [0 0 300 220] "
        "/Resources << /Font << /F1 8 0 R >> >> "
        f"/Contents {contents_1} 0 R >>"
    )
    page_2 = pdf.add(
        "<< /Type /Page /Parent 7 0 R /MediaBox [0 0 300 220] "
        "/Resources << /Font << /F1 8 0 R >> >> "
        f"/Contents {contents_2} 0 R >>"
    )
    page_3 = pdf.add(
        "<< /Type /Page /Parent 7 0 R /MediaBox [0 0 300 220] "
        "/Resources << /Font << /F1 8 0 R >> >> "
        f"/Contents {contents_3} 0 R >>"
    )
    pages = pdf.add(
        f"<< /Type /Pages /Kids [{page_1} 0 R {page_2} 0 R {page_3} 0 R] /Count 3 >>"
    )
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert pages == 7
    assert font == 8
    return pdf.render(catalog)


def book_frontmatter_page_labels_pdf() -> bytes:
    pdf = Pdf()

    def page_content(title: str, body_width: int) -> bytes:
        ops: list[str] = [
            "q 1 1 1 rg 0 0 260 360 re f Q",
            "q 0.16 0.18 0.22 RG 0.5 w 34 54 m 226 54 l 34 302 m 226 302 l S Q",
            f"BT /F1 12 Tf 44 324 Td ({title}) Tj ET",
        ]
        for row in range(14):
            y = 286 - row * 14
            width = body_width - (row % 4) * 16
            ops.append(f"q 0.18 0.18 0.18 rg 44 {y} {width} 2 re f Q")
        ops.append("BT /F1 6 Tf 116 30 Td (page marker) Tj ET")
        return " ".join(ops).encode("ascii")

    contents = [
        pdf.add(
            f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
            + content
            + b"\nendstream"
        )
        for content in [
            page_content("Preface", 156),
            page_content("Contents", 172),
            page_content("Chapter One", 166),
            page_content("Chapter Two", 150),
            page_content("Appendix", 162),
        ]
    ]
    page_objects = [
        pdf.add(
            "<< /Type /Page /Parent 11 0 R /MediaBox [0 0 260 360] "
            "/Resources << /Font << /F1 12 0 R >> >> "
            f"/Contents {content} 0 R >>"
        )
        for content in contents
    ]
    pages = pdf.add(
        "<< /Type /Pages /Kids ["
        + " ".join(f"{page} 0 R" for page in page_objects)
        + "] /Count 5 >>"
    )
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Times-Roman >>")
    outline_one = pdf.add(
        f"<< /Title (Preface) /Parent 16 0 R /Dest [{page_objects[0]} 0 R /Fit] /Next 14 0 R >>"
    )
    outline_two = pdf.add(
        f"<< /Title (Chapter One) /Parent 16 0 R /Dest [{page_objects[2]} 0 R /Fit] /Next 15 0 R >>"
    )
    outline_three = pdf.add(
        f"<< /Title (Appendix) /Parent 16 0 R /Dest [{page_objects[4]} 0 R /Fit] >>"
    )
    outlines = pdf.add(
        f"<< /Type /Outlines /First {outline_one} 0 R /Last {outline_three} 0 R /Count 3 >>"
    )
    info = pdf.add(
        "<< /Title (Longform Book Fixture) /Author (pdfrust) /Creator (fixture generator) >>"
    )
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /Outlines {outlines} 0 R "
        "/PageLabels << /Nums [0 << /S /r /St 1 >> 2 << /P (Ch-) /S /D /St 1 >>] >> >>"
    )
    assert pages == 11
    assert font == 12
    assert outlines == 16
    return pdf.render(catalog, trailer_entries=f"/Info {info} 0 R ")


def manual_illustrated_chapter_pdf() -> bytes:
    return page_pdf(
        "[0 0 320 260]",
        (
            "q 0.98 0.98 0.96 rg 0 0 320 260 re f Q "
            "q 0.12 0.16 0.24 rg 24 214 272 20 re f Q "
            "q 0.90 0.93 0.96 rg 30 86 116 94 re f Q "
            "q 0.18 0.30 0.48 RG 0.8 w 44 104 m 74 156 l 112 118 l 138 168 l S "
            "172 88 100 80 re S 172 144 m 272 144 l 172 118 m 272 118 l "
            "212 88 m 212 168 l S Q "
            "BT /F1 12 Tf 30 219 Td (Manual Chapter 3) Tj ET "
            "BT /F1 7 Tf 32 190 Td (Installation steps) Tj 0 -22 Td (1. Mount bracket) Tj "
            "0 -16 Td (2. Connect cable) Tj 0 -16 Td (3. Verify display) Tj "
            "144 44 Td (Part) Tj 40 0 Td (Qty) Tj 44 0 Td (Note) Tj ET"
        ),
    )


def ebook_narrow_longform_pdf() -> bytes:
    ops: list[str] = [
        "q 0.99 0.98 0.94 rg 0 0 180 300 re f Q",
        "BT /F1 13 Tf 28 270 Td (Chapter 4) Tj ET",
    ]
    for row in range(18):
        y = 244 - row * 11
        width = 124 - (row % 5) * 9
        ops.append(f"q 0.20 0.18 0.16 rg 28 {y} {width} 1.6 re f Q")
    ops.extend(
        [
            "q 0.54 0.38 0.20 RG 0.6 w 28 34 m 152 34 l S Q",
            "BT /F1 6 Tf 68 20 Td (ebook flow sample) Tj ET",
        ]
    )
    return page_pdf("[0 0 180 300]", " ".join(ops))


def longform_repeated_resources_pdf() -> bytes:
    pdf = Pdf()
    image = bytes(
        [
            242,
            248,
            255,
            190,
            214,
            240,
            80,
            120,
            180,
            230,
            236,
            244,
        ]
        * 4
    )
    image_object = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 4 /Height 4 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length "
        + str(len(image)).encode("ascii")
        + b" >>\nstream\n"
        + image
        + b"\nendstream"
    )

    def page_content(title: str) -> bytes:
        ops: list[str] = [
            "q 1 1 1 rg 0 0 240 320 re f Q",
            "q 88 0 0 68 34 186 cm /Img Do Q",
            f"BT /F1 11 Tf 34 282 Td ({title}) Tj ET",
        ]
        for row in range(10):
            y = 160 - row * 12
            ops.append(f"q 0.16 0.16 0.16 rg 34 {y} {150 - (row % 3) * 18} 2 re f Q")
        ops.append("BT /F1 6 Tf 94 24 Td (shared resources) Tj ET")
        return " ".join(ops).encode("ascii")

    content_objects = [
        pdf.add(
            f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
            + content
            + b"\nendstream"
        )
        for content in [
            page_content("Illustrated Essay p1"),
            page_content("Illustrated Essay p2"),
            page_content("Illustrated Essay p3"),
        ]
    ]
    page_objects = [
        pdf.add(
            "<< /Type /Page /Parent 8 0 R /MediaBox [0 0 240 320] "
            f"/Resources << /Font << /F1 9 0 R >> /XObject << /Img {image_object} 0 R >> >> "
            f"/Contents {content} 0 R >>"
        )
        for content in content_objects
    ]
    pages = pdf.add(
        "<< /Type /Pages /Kids ["
        + " ".join(f"{page} 0 R" for page in page_objects)
        + "] /Count 3 >>"
    )
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert pages == 8
    assert font == 9
    return pdf.render(catalog)


def prepress_trim_bleed_marks_pdf() -> bytes:
    pdf = Pdf()
    ops: list[str] = [
        "q 0.98 0.98 0.96 rg 0 0 340 260 re f Q",
        "q 0.10 0.12 0.16 RG 0.6 w 30 30 280 200 re S Q",
        "q 0.86 0.18 0.12 RG 0.8 w 20 20 300 220 re S Q",
        "q 0.05 0.05 0.05 RG 0.5 w",
    ]
    for x, y, dx, dy in [
        (12, 30, 26, 0),
        (302, 30, 26, 0),
        (12, 230, 26, 0),
        (302, 230, 26, 0),
        (30, 12, 0, 26),
        (310, 12, 0, 26),
        (30, 222, 0, 26),
        (310, 222, 0, 26),
    ]:
        ops.append(f"{x} {y} m {x + dx} {y + dy} l S")
    ops.extend(
        [
            "Q",
            "q 0.16 0.32 0.52 rg 54 70 232 118 re f Q",
            "BT /F1 11 Tf 76 132 Td (Trim/Bleed Thumbnail) Tj ET",
        ]
    )
    content = " ".join(ops).encode("ascii")
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 340 260] "
        "/CropBox [20 20 320 240] /BleedBox [10 10 330 250] /TrimBox [30 30 310 230] "
        f"/Resources << /Font << /F1 4 0 R >> >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def prepress_output_intent_page_boxes_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.97 0.97 0.97 rg 0 0 360 260 re f Q "
        b"q 0.16 0.18 0.22 RG 0.8 w 40 40 280 180 re S Q "
        b"q 0.08 0.38 0.58 rg 62 72 88 108 re f "
        b"0.86 0.34 0.12 rg 166 72 88 108 re f Q "
        b"BT /F1 10 Tf 74 198 Td (OutputIntent + CropBox) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    profile = b"tiny-rgb-profile"
    profile_object = pdf.add(
        b"<< /N 3 /Length "
        + str(len(profile)).encode("ascii")
        + b" >>\nstream\n"
        + profile
        + b"\nendstream"
    )
    output_intent = pdf.add(
        f"<< /Type /OutputIntent /S /GTS_PDFA1 /OutputConditionIdentifier (sRGB thumbnail) /DestOutputProfile {profile_object} 0 R >>"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 5 0 R /MediaBox [0 0 360 260] "
        "/CropBox [30 20 330 240] /BleedBox [20 10 340 250] /TrimBox [40 40 320 220] "
        f"/Resources << /Font << /F1 6 0 R >> >> /Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(
        f"<< /Type /Catalog /Pages {pages} 0 R /OutputIntents [{output_intent} 0 R] >>"
    )
    assert pages == 5
    assert font == 6
    return pdf.render(catalog)


def prepress_registration_color_bars_pdf() -> bytes:
    ops: list[str] = [
        "q 1 1 1 rg 0 0 360 180 re f Q",
        "q 0 1 1 rg 32 126 46 18 re f Q",
        "q 1 0 1 rg 84 126 46 18 re f Q",
        "q 1 1 0 rg 136 126 46 18 re f Q",
        "q 0 0 0 rg 188 126 46 18 re f Q",
        "q 0.5 0.5 0.5 rg 240 126 46 18 re f Q",
        "q 0.02 0.02 0.02 RG 0.8 w",
    ]
    for cx, cy in [(44, 42), (316, 42), (44, 154), (316, 154)]:
        ops.append(f"{cx - 12} {cy} m {cx + 12} {cy} l S {cx} {cy - 12} m {cx} {cy + 12} l S")
        ops.append(f"{cx - 7} {cy - 7} 14 14 re S")
    ops.extend(
        [
            "Q",
            "q 0.12 0.20 0.32 RG 0.6 w 32 34 296 112 re S Q",
            "BT /F1 8 Tf 36 106 Td (Registration and process color bars) Tj ET",
        ]
    )
    return page_pdf("[0 0 360 180]", " ".join(ops))


def prepress_spot_overprint_boundary_pdf() -> bytes:
    pdf = Pdf()
    content = (
        b"q 0.96 0.96 0.94 rg 0 0 240 180 re f Q "
        b"q /GSOP gs /CS1 cs 0.92 scn 34 42 120 80 re f "
        b"0.12 0.18 0.28 rg 78 72 120 72 re f Q "
        b"BT /F1 9 Tf 34 144 Td (Spot/Overprint Approximation) Tj ET"
    )
    contents = pdf.add(
        f"<< /Length {len(content)} >>\nstream\n".encode("ascii")
        + content
        + b"\nendstream"
    )
    page = pdf.add(
        "<< /Type /Page /Parent 3 0 R /MediaBox [0 0 240 180] "
        "/Resources << "
        "/Font << /F1 4 0 R >> "
        "/ExtGState << /GSOP << /OP true /op true /OPM 1 >> >> "
        "/ColorSpace << /CS1 "
        "[/Separation /BoundaryOrange /DeviceRGB "
        "<< /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [1 0.42 0.10] /N 1 >>] "
        ">> >> "
        f"/Contents {contents} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>")
    font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert font == 4
    return pdf.render(catalog)


def page_targeted_stream_pdf() -> bytes:
    pdf = Pdf()
    content_1 = b"q 0.1 0.6 0.2 rg 20 20 80 40 re f Q"
    content_2 = b"this stream should not decode while rendering page zero"
    unused_image = b"unused image should not decode"
    contents_1 = pdf.add(
        f"<< /Length {len(content_1)} >>\nstream\n".encode("ascii")
        + content_1
        + b"\nendstream"
    )
    contents_2 = pdf.add(
        f"<< /Length {len(content_2)} /Filter /UnsupportedDecode >>\nstream\n".encode(
            "ascii"
        )
        + content_2
        + b"\nendstream"
    )
    image = pdf.add(
        b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /BitsPerComponent 8 "
        b"/ColorSpace /DeviceRGB /Filter /UnsupportedDecode /Length "
        + str(len(unused_image)).encode("ascii")
        + b" >>\nstream\n"
        + unused_image
        + b"\nendstream"
    )
    page_1 = pdf.add(
        "<< /Type /Page /Parent 6 0 R /MediaBox [0 0 120 80] "
        f"/Resources << /XObject << /Unused {image} 0 R >> >> /Contents {contents_1} 0 R >>"
    )
    page_2 = pdf.add(
        "<< /Type /Page /Parent 6 0 R /MediaBox [0 0 120 80] "
        f"/Resources << >> /Contents {contents_2} 0 R >>"
    )
    pages = pdf.add(f"<< /Type /Pages /Kids [{page_1} 0 R {page_2} 0 R] /Count 2 >>")
    catalog = pdf.add(f"<< /Type /Catalog /Pages {pages} 0 R >>")
    assert pages == 6
    return pdf.render(catalog)


def write(name: str, data: bytes) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / name).write_bytes(data)


def main() -> None:
    write(
        "page-size-letter.pdf",
        page_pdf("[0 0 612 792]", "q 0.9 0.9 0.9 rg 0 0 612 792 re f Q"),
    )
    write("rotated-office-export.pdf", rotated_office_export_pdf())
    write("cropped-scan-page.pdf", cropped_scan_page_pdf())
    write("user-unit-page.pdf", user_unit_page_pdf())
    write("metadata-outline-page-labels.pdf", metadata_outline_page_labels_pdf())
    write("tagged-accessibility-metadata.pdf", tagged_accessibility_metadata_pdf())
    write("tagged-report-visual-integrity.pdf", tagged_report_visual_integrity_pdf())
    write("tagged-form-visual-integrity.pdf", tagged_form_visual_integrity_pdf())
    write("tagged-office-alt-text.pdf", tagged_office_alt_text_pdf())
    write("tagged-structure-heavy-report.pdf", tagged_structure_heavy_report_pdf())
    write("malformed-tagged-structure.pdf", malformed_tagged_structure_pdf())
    write(
        "text-page.pdf",
        page_pdf(
            "[0 0 300 160]",
            "BT /F1 24 Tf 40 90 Td (pdfrust thumbnail fixture) Tj ET",
        ),
    )
    write(
        "vector-paths.pdf",
        page_pdf(
            "[0 0 220 180]",
            "q 0.1 0.4 0.8 RG 4 w 30 30 m 110 150 l 190 30 l S "
            "0.9 0.2 0.1 rg 70 55 80 50 re f Q",
        ),
    )
    write(
        "inline-image.pdf",
        page_pdf(
            "[0 0 120 120]",
            b"q 64 0 0 64 28 28 cm BI /W 2 /H 2 /CS /RGB /BPC 8 ID "
            b"\xff\x00\x00\x00\xff\x00\x00\x00\xff\xff\xff\x00 EI Q",
        ),
    )
    write("form-xobject.pdf", form_xobject_pdf())
    write("image-xobject.pdf", image_xobject_pdf())
    write("cmyk-image.pdf", cmyk_image_pdf())
    write("icc-rgb-image.pdf", icc_rgb_image_pdf())
    write("icc-gray-image.pdf", icc_gray_image_pdf())
    write("icc-cmyk-image.pdf", icc_cmyk_image_pdf())
    write("output-intent-rgb.pdf", output_intent_rgb_pdf())
    write("indexed-image.pdf", indexed_image_pdf())
    write("dct-image.pdf", dct_image_pdf())
    write("predictor-image.pdf", predictor_image_pdf())
    write("soft-mask-image.pdf", soft_mask_image_pdf())
    write("image-mask-signature.pdf", image_mask_signature_pdf())
    write("image-mask-monochrome-icon.pdf", image_mask_monochrome_icon_pdf())
    write("image-mask-logo.pdf", image_mask_logo_pdf())
    write("unsupported-ccitt-image.pdf", unsupported_image_codec_pdf("CCITTFaxDecode"))
    write("unsupported-jbig2-image.pdf", unsupported_image_codec_pdf("JBIG2Decode"))
    write("unsupported-jpx-image.pdf", unsupported_image_codec_pdf("JPXDecode"))
    write("scanned-page.pdf", scanned_page_pdf())
    write("mobile-rotated-camera-scan.pdf", mobile_rotated_camera_scan_pdf())
    write("mobile-cropped-photo-scan.pdf", mobile_cropped_photo_scan_pdf())
    write("mobile-ocr-overlay-scan.pdf", mobile_ocr_overlay_scan_pdf())
    write("mobile-mixed-compression-scan.pdf", mobile_mixed_compression_scan_pdf())
    write("scanner-skewed-mailroom-page.pdf", scanner_skewed_mailroom_page_pdf())
    write("scanner-large-image-budget.pdf", scanner_large_image_budget_pdf())
    write("scanner-ocr-form-overlay.pdf", scanner_ocr_form_overlay_pdf())
    write("ocr-invisible-text-layer.pdf", ocr_invisible_text_layer_pdf())
    write("mixed-text-image.pdf", mixed_text_image_pdf())
    write("transparency-group.pdf", transparency_group_pdf())
    write("transparency-knockout-group.pdf", transparency_knockout_group_pdf())
    write("transparency-isolated-alpha-group.pdf", transparency_isolated_alpha_group_pdf())
    write("blend-modes.pdf", blend_modes_pdf())
    write("blend-mode-array-fallback.pdf", blend_mode_array_fallback_pdf())
    write("unsupported-blend-mode.pdf", unsupported_blend_mode_pdf())
    write("extgstate-luminosity-soft-mask.pdf", extgstate_luminosity_soft_mask_pdf())
    write("transparency-alpha.pdf", transparency_alpha_pdf())
    write("axial-gradient.pdf", axial_gradient_pdf())
    write("radial-gradient.pdf", radial_gradient_pdf())
    write("mesh-shading-unsupported.pdf", mesh_shading_unsupported_pdf())
    write("type4-mesh-shading.pdf", type4_mesh_shading_pdf())
    write("separation-spot-color.pdf", separation_spot_color_pdf())
    write("overprint-spot-approximation.pdf", overprint_spot_approximation_pdf())
    write("devicen-spot-color.pdf", devicen_spot_color_pdf())
    write("tiling-pattern.pdf", tiling_pattern_pdf())
    write("uncolored-tiling-pattern.pdf", uncolored_tiling_pattern_pdf())
    write("dashed-stroke.pdf", dashed_stroke_pdf())
    write("line-caps.pdf", line_caps_pdf())
    write("line-joins.pdf", line_joins_pdf())
    write("clipped-paths.pdf", clipped_paths_pdf())
    write("vector-stress.pdf", vector_stress_pdf())
    write("annotation-appearance.pdf", annotation_appearance_pdf())
    write("annotation-missing-appearance.pdf", annotation_missing_appearance_pdf())
    write("link-annotation-appearance.pdf", link_annotation_appearance_pdf())
    write("highlight-annotation-appearance.pdf", highlight_annotation_appearance_pdf())
    write(
        "highlight-annotation-without-appearance.pdf",
        highlight_annotation_without_appearance_pdf(),
    )
    write(
        "markup-annotations-without-appearance.pdf",
        markup_annotations_without_appearance_pdf(),
    )
    write(
        "link-annotation-without-appearance.pdf",
        link_annotation_without_appearance_pdf(),
    )
    write(
        "text-note-annotation-without-appearance.pdf",
        text_note_annotation_without_appearance_pdf(),
    )
    write("widget-annotation-appearance.pdf", widget_annotation_appearance_pdf())
    write("acroform-text-field.pdf", acroform_text_field_pdf())
    write("xfa-static-appearance.pdf", xfa_static_appearance_pdf())
    write("xfa-dynamic-no-static-appearance.pdf", xfa_dynamic_no_static_appearance_pdf())
    write(
        "acroform-text-field-missing-appearance.pdf",
        acroform_text_field_missing_appearance_pdf(),
    )
    write("acroform-checkbox.pdf", acroform_checkbox_pdf())
    write(
        "acroform-checkbox-missing-appearance.pdf",
        acroform_checkbox_missing_appearance_pdf(),
    )
    write(
        "acroform-choice-missing-appearance.pdf",
        acroform_choice_missing_appearance_pdf(),
    )
    write("acroform-radio.pdf", acroform_radio_pdf())
    write(
        "acroform-radio-missing-appearance.pdf",
        acroform_radio_missing_appearance_pdf(),
    )
    write("acroform-radio-off.pdf", acroform_radio_off_pdf())
    write("acroform-signature-placeholder.pdf", acroform_signature_placeholder_pdf())
    write("digital-signature-appearance.pdf", digital_signature_appearance_pdf())
    write("e-signature-contract-workflow.pdf", e_signature_contract_workflow_pdf())
    write("e-signature-audit-certificate.pdf", e_signature_audit_certificate_pdf())
    write("e-signature-incremental-revision.pdf", e_signature_incremental_revision_pdf())
    write("embedded-source-file.pdf", embedded_source_file_pdf())
    write("portfolio-embedded-files.pdf", portfolio_embedded_files_pdf())
    write("file-attachment-annotation.pdf", file_attachment_annotation_pdf())
    write("linearized-first-page.pdf", linearized_first_page_pdf())
    write("linearized-malformed-hints.pdf", linearized_first_page_pdf(malformed_hint=True))
    write("optional-content-layer-on.pdf", optional_content_layer_pdf(visible=True))
    write("optional-content-layer-off.pdf", optional_content_layer_pdf(visible=False))
    write("optional-content-ocmd.pdf", optional_content_ocmd_pdf())
    write("incremental-update.pdf", incremental_update_pdf())
    write("incremental-deleted-object.pdf", incremental_deleted_object_pdf())
    write("hybrid-reference.pdf", hybrid_reference_pdf())
    write("encrypted-placeholder.pdf", encrypted_placeholder_pdf())
    write("malformed-xref-offset-drift.pdf", malformed_xref_offset_drift_pdf())
    write("embedded-font.pdf", embedded_font_pdf())
    write("tounicode-text.pdf", tounicode_text_pdf())
    write("cid-font-text.pdf", cid_font_text_pdf())
    write("vertical-cjk-text.pdf", vertical_cjk_text_pdf())
    write("shaped-rtl-text.pdf", shaped_rtl_text_pdf())
    write("opentype-ligature-text.pdf", opentype_ligature_text_pdf())
    write("combining-mark-text.pdf", combining_mark_text_pdf())
    write("chat-emoji-fallback-boundary.pdf", chat_emoji_fallback_boundary_pdf())
    write("arabic-shaped-text.pdf", arabic_shaped_text_pdf())
    write("identity-h-cjk-text.pdf", identity_h_cjk_text_pdf())
    write("identity-v-cjk-text.pdf", identity_v_cjk_text_pdf())
    write("cmap-codespace-range-text.pdf", cmap_codespace_range_text_pdf())
    write("encoding-differences.pdf", encoding_differences_pdf())
    write("text-spacing.pdf", text_spacing_pdf())
    write(
        "missing-font-office-export.pdf",
        missing_font_pdf("ABCDEE+InvoiceSerif", "office missing font"),
    )
    write(
        "missing-font-invoice.pdf",
        missing_font_pdf("ABCDEE+InvoiceSans", "invoice missing font", "[0 0 220 120]"),
    )
    write(
        "missing-font-browser-print.pdf",
        missing_font_pdf("ABCDEE+BrowserMono", "browser missing font", "[0 0 260 120]"),
    )
    write("type1-fontfile-text.pdf", type1_fontfile_text_pdf())
    write("cff-fontfile3-text.pdf", cff_fontfile3_text_pdf())
    write("subset-truetype-widths.pdf", subset_truetype_widths_pdf())
    write("subset-cff-tounicode.pdf", subset_cff_tounicode_pdf())
    write("subset-cid-widths.pdf", subset_cid_widths_pdf())
    write("subset-type3-repeated-charprocs.pdf", subset_type3_repeated_charprocs_pdf())
    write("subset-missing-font.pdf", subset_missing_font_pdf())
    write("type3-vector-text.pdf", type3_vector_text_pdf())
    write("type3-symbol-font.pdf", type3_symbol_font_pdf())
    write("type3-barcode-font.pdf", type3_barcode_font_pdf())
    write("office-table.pdf", office_table_pdf())
    write("multi-page-report.pdf", multi_page_report_pdf())
    write("business-invoice-dense.pdf", business_invoice_dense_pdf())
    write("account-statement-ledger.pdf", account_statement_ledger_pdf())
    write("financial-annual-report-page.pdf", financial_annual_report_page_pdf())
    write("financial-cashflow-statement.pdf", financial_cashflow_statement_pdf())
    write("financial-chart-summary.pdf", financial_chart_summary_pdf())
    write("thermal-receipt.pdf", thermal_receipt_pdf())
    write("business-form-stamp-signature.pdf", business_form_stamp_signature_pdf())
    write("government-permit-checkbox-form.pdf", government_permit_checkbox_form_pdf())
    write("government-certificate-seal-signature.pdf", government_certificate_seal_signature_pdf())
    write("government-tax-notice-barcode.pdf", government_tax_notice_barcode_pdf())
    write("browser-chromium-article-print.pdf", browser_chromium_article_print_pdf())
    write("browser-firefox-dashboard-print.pdf", browser_firefox_dashboard_print_pdf())
    write("browser-webkit-receipt-form-print.pdf", browser_webkit_receipt_form_print_pdf())
    write("browser-print-sticky-headers.pdf", browser_print_sticky_headers_pdf())
    write("browser-print-clipped-backgrounds.pdf", browser_print_clipped_backgrounds_pdf())
    write("browser-print-transformed-cards.pdf", browser_print_transformed_cards_pdf())
    write("browser-print-raster-vector-mix.pdf", browser_print_raster_vector_mix_pdf())
    write("email-client-thread.pdf", email_client_thread_pdf())
    write("email-inline-image-link.pdf", email_inline_image_link_pdf())
    write("web-archive-snapshot.pdf", web_archive_snapshot_pdf())
    write("email-attachment-summary.pdf", email_attachment_summary_pdf())
    write("pdf20-basic-office.pdf", pdf20_basic_office_pdf())
    write("pdf20-associated-files.pdf", pdf20_associated_files_pdf())
    write("pdf20-black-point-compensation.pdf", pdf20_black_point_compensation_pdf())
    write("office-report-header-footer-link.pdf", office_report_header_footer_link_pdf())
    write("office-spreadsheet-chart-comments.pdf", office_spreadsheet_chart_comments_pdf())
    write("office-presentation-handout.pdf", office_presentation_handout_pdf())
    write("office-vector-grouped-shapes.pdf", office_vector_grouped_shapes_pdf())
    write("office-vector-nested-clips.pdf", office_vector_nested_clips_pdf())
    write(
        "office-vector-clipped-transparency-group.pdf",
        office_vector_clipped_transparency_group_pdf(),
    )
    write("office-vector-repeated-effects.pdf", office_vector_repeated_effects_pdf())
    write("legal-contract-signature-blocks.pdf", legal_contract_signature_blocks_pdf())
    write("legal-visible-redactions.pdf", legal_visible_redactions_pdf())
    write("legal-filing-stamp-comments.pdf", legal_filing_stamp_comments_pdf())
    write("legal-scanned-attachment-packet.pdf", legal_scanned_attachment_packet_pdf())
    write("slide-title-gradient.pdf", slide_title_gradient_pdf())
    write("slide-layered-image-shadow.pdf", slide_layered_image_shadow_pdf())
    write("slide-rotated-callout.pdf", slide_rotated_callout_pdf())
    write("slide-speaker-notes-page.pdf", slide_speaker_notes_page_pdf())
    write("spreadsheet-frozen-header.pdf", spreadsheet_frozen_header_pdf())
    write("spreadsheet-dense-numeric-grid.pdf", spreadsheet_dense_numeric_grid_pdf())
    write("spreadsheet-clipped-cells.pdf", spreadsheet_clipped_cells_pdf())
    write("spreadsheet-vector-stress-grid.pdf", spreadsheet_vector_stress_grid_pdf())
    write("technical-linework-dimensions.pdf", technical_linework_dimensions_pdf())
    write("technical-hatch-clipping.pdf", technical_hatch_clipping_pdf())
    write("technical-large-coordinate-plan.pdf", technical_large_coordinate_plan_pdf())
    write("technical-repeated-symbols.pdf", technical_repeated_symbols_pdf())
    write("engineering-floorplan-precision.pdf", engineering_floorplan_precision_pdf())
    write("engineering-schematic-symbols.pdf", engineering_schematic_symbols_pdf())
    write("engineering-large-transform-detail.pdf", engineering_large_transform_detail_pdf())
    write("chart-combo-legend.pdf", chart_combo_legend_pdf())
    write("dashboard-kpi-panels.pdf", dashboard_kpi_panels_pdf())
    write("map-marker-clusters.pdf", map_marker_clusters_pdf())
    write("map-raster-tile-routes.pdf", map_raster_tile_routes_pdf())
    write("map-transparent-zoning-overlay.pdf", map_transparent_zoning_overlay_pdf())
    write("map-optional-layer-policy.pdf", map_optional_layer_policy_pdf())
    write("dashboard-heatmap-overlay.pdf", dashboard_heatmap_overlay_pdf())
    write("scientific-two-column-paper.pdf", scientific_two_column_paper_pdf())
    write("scientific-equation-figure.pdf", scientific_equation_figure_pdf())
    write("reference-footnote-layout.pdf", reference_footnote_layout_pdf())
    write("academic-publisher-first-page.pdf", academic_publisher_first_page_pdf())
    write("academic-equation-symbols-page.pdf", academic_equation_symbols_page_pdf())
    write("academic-references-appendix.pdf", academic_references_appendix_pdf())
    write("long-report-sampling.pdf", long_report_sampling_pdf())
    write("book-frontmatter-page-labels.pdf", book_frontmatter_page_labels_pdf())
    write("manual-illustrated-chapter.pdf", manual_illustrated_chapter_pdf())
    write("ebook-narrow-longform.pdf", ebook_narrow_longform_pdf())
    write("longform-repeated-resources.pdf", longform_repeated_resources_pdf())
    write("prepress-trim-bleed-marks.pdf", prepress_trim_bleed_marks_pdf())
    write("prepress-output-intent-page-boxes.pdf", prepress_output_intent_page_boxes_pdf())
    write("prepress-registration-color-bars.pdf", prepress_registration_color_bars_pdf())
    write("prepress-spot-overprint-boundary.pdf", prepress_spot_overprint_boundary_pdf())
    write("page-targeted-stream.pdf", page_targeted_stream_pdf())


if __name__ == "__main__":
    main()
