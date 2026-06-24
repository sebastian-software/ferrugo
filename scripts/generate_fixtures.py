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

    def render(self, root: int) -> bytes:
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
        for offset in offsets[1:]:
            out.extend(f"{offset:010d} 00000 n \n".encode("ascii"))
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
    write("transparency-group.pdf", transparency_group_pdf())
    write("blend-modes.pdf", blend_modes_pdf())
    write("axial-gradient.pdf", axial_gradient_pdf())
    write("embedded-font.pdf", embedded_font_pdf())
    write("tounicode-text.pdf", tounicode_text_pdf())
    write("encoding-differences.pdf", encoding_differences_pdf())
    write("text-spacing.pdf", text_spacing_pdf())


if __name__ == "__main__":
    main()
