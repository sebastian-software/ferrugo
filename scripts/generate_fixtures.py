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

    def render(self, root: int, offset_drift: dict[int, int] | None = None) -> bytes:
        offset_drift = offset_drift or {}
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
                f"trailer\n<< /Size {len(self.objects) + 1} /Root {root} 0 R >>\n"
                f"startxref\n{xref_offset}\n%%EOF\n"
            ).encode("ascii")
        )
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


def write(name: str, data: bytes) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / name).write_bytes(data)


def main() -> None:
    write(
        "page-size-letter.pdf",
        page_pdf("[0 0 612 792]", "q 0.9 0.9 0.9 rg 0 0 612 792 re f Q"),
    )
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
    write("indexed-image.pdf", indexed_image_pdf())
    write("dct-image.pdf", dct_image_pdf())
    write("predictor-image.pdf", predictor_image_pdf())
    write("soft-mask-image.pdf", soft_mask_image_pdf())
    write("scanned-page.pdf", scanned_page_pdf())
    write("mixed-text-image.pdf", mixed_text_image_pdf())
    write("transparency-group.pdf", transparency_group_pdf())
    write("blend-modes.pdf", blend_modes_pdf())
    write("transparency-alpha.pdf", transparency_alpha_pdf())
    write("axial-gradient.pdf", axial_gradient_pdf())
    write("radial-gradient.pdf", radial_gradient_pdf())
    write("tiling-pattern.pdf", tiling_pattern_pdf())
    write("dashed-stroke.pdf", dashed_stroke_pdf())
    write("line-caps.pdf", line_caps_pdf())
    write("line-joins.pdf", line_joins_pdf())
    write("clipped-paths.pdf", clipped_paths_pdf())
    write("annotation-appearance.pdf", annotation_appearance_pdf())
    write("annotation-missing-appearance.pdf", annotation_missing_appearance_pdf())
    write("link-annotation-appearance.pdf", link_annotation_appearance_pdf())
    write("highlight-annotation-appearance.pdf", highlight_annotation_appearance_pdf())
    write("widget-annotation-appearance.pdf", widget_annotation_appearance_pdf())
    write("acroform-text-field.pdf", acroform_text_field_pdf())
    write("acroform-checkbox.pdf", acroform_checkbox_pdf())
    write("acroform-signature-placeholder.pdf", acroform_signature_placeholder_pdf())
    write("optional-content-layer-on.pdf", optional_content_layer_pdf(visible=True))
    write("optional-content-layer-off.pdf", optional_content_layer_pdf(visible=False))
    write("optional-content-ocmd.pdf", optional_content_ocmd_pdf())
    write("incremental-update.pdf", incremental_update_pdf())
    write("hybrid-reference.pdf", hybrid_reference_pdf())
    write("encrypted-placeholder.pdf", encrypted_placeholder_pdf())
    write("malformed-xref-offset-drift.pdf", malformed_xref_offset_drift_pdf())
    write("embedded-font.pdf", embedded_font_pdf())
    write("tounicode-text.pdf", tounicode_text_pdf())
    write("cid-font-text.pdf", cid_font_text_pdf())
    write("vertical-cjk-text.pdf", vertical_cjk_text_pdf())
    write("shaped-rtl-text.pdf", shaped_rtl_text_pdf())
    write("encoding-differences.pdf", encoding_differences_pdf())
    write("text-spacing.pdf", text_spacing_pdf())
    write("office-table.pdf", office_table_pdf())


if __name__ == "__main__":
    main()
