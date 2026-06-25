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
    ) -> bytes:
        offset_drift = offset_drift or {}
        if isinstance(trailer_entries, str):
            trailer_entries = trailer_entries.encode("ascii")
        out = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
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


def dct_image_pdf() -> bytes:
    pdf = Pdf()
    content = b"q 80 0 0 80 20 20 cm /Im1 Do Q"
    image = bytes.fromhex(
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
    write("mixed-text-image.pdf", mixed_text_image_pdf())
    write("transparency-group.pdf", transparency_group_pdf())
    write("blend-modes.pdf", blend_modes_pdf())
    write("transparency-alpha.pdf", transparency_alpha_pdf())
    write("axial-gradient.pdf", axial_gradient_pdf())
    write("radial-gradient.pdf", radial_gradient_pdf())
    write("mesh-shading-unsupported.pdf", mesh_shading_unsupported_pdf())
    write("tiling-pattern.pdf", tiling_pattern_pdf())
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
    write("type3-vector-text.pdf", type3_vector_text_pdf())
    write("type3-symbol-font.pdf", type3_symbol_font_pdf())
    write("type3-barcode-font.pdf", type3_barcode_font_pdf())
    write("office-table.pdf", office_table_pdf())
    write("multi-page-report.pdf", multi_page_report_pdf())
    write("page-targeted-stream.pdf", page_targeted_stream_pdf())


if __name__ == "__main__":
    main()
