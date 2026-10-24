#!/usr/bin/env python3
"""Wrap an original RC2 LEVEL WAD overlay for private PS2Recomp analysis.

This creates a derived analysis ELF, not a boot executable or runtime adapter.
Keep its output private: it contains original game code. No assets are converted.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct


def overlay_sections(wad):
    if len(wad) < 0x60 or struct.unpack_from('<I', wad)[0] != 0x60:
        raise ValueError('not an RC2 level WAD')
    sector, sectors = struct.unpack_from('<II', wad, 0x10)
    start, end = sector * 2048, (sector + sectors) * 2048
    if not sectors or start + 0x80 > len(wad):
        raise ValueError('invalid data part')
    data = wad[start:min(end, len(wad))]
    off, size = struct.unpack_from('<ii', data)
    if off < 0x80 or size < 16 or off + size > len(data):
        raise ValueError('invalid overlay extent')
    overlay = data[off:off + size]
    sections, pos = [], 0
    entry = struct.unpack_from('<I', overlay, 12)[0]
    while pos + 16 <= len(overlay):
        address, size, kind, section_entry = struct.unpack_from('<4I', overlay, pos)
        if section_entry != entry:  # Original overlay termination contract.
            break
        if kind not in (1, 8) or not size or pos + 16 + size > len(overlay):
            raise ValueError('unsupported or truncated overlay section')
        if address + size > 0x2000000:
            raise ValueError('section outside EE RAM')
        if any(address < a + len(b) and a < address + size for a, k, b in sections):
            raise ValueError('overlapping overlay sections')
        sections.append((address, kind, overlay[pos + 16:pos + 16 + size]))
        pos += 16 + size
    if sum(k == 1 and a <= entry < a + len(b) for a, k, b in sections) != 1:
        raise ValueError('entry does not identify one initialized code section')
    return entry, sections, overlay


def analysis_elf(entry, sections):
    # Only the entry-containing section is marked executable. The source
    # container has no ELF execute flags; other PROGBITS are retained as data.
    out, names, programs, headers = bytearray(0x1000), bytearray(b'\0'), [], [bytes(40)]
    for i, (address, kind, data) in enumerate(sections):
        text = kind == 1 and address <= entry < address + len(data)
        name = ('.text' if text else '.bss' if kind == 8 else '.data') + str(i)
        name_offset = len(names)
        names += name.encode() + b'\0'
        offset, size = len(out), len(data)
        if kind != 8:
            out += data
        out += bytes(-len(out) % 16)
        programs.append(struct.pack('<8I', 1, offset, address, address,
                                    0 if kind == 8 else size, size, 5 if text else 6, 16))
        headers.append(struct.pack('<10I', name_offset, kind, 6 if text else 3,
                                   address, offset, size, 0, 0, 16, 0))
    name_offset = len(names)
    names += b'.shstrtab\0'
    string_offset = len(out)
    out += names
    out += bytes(-len(out) % 16)
    headers.append(struct.pack('<10I', name_offset, 3, 0, 0, string_offset, len(names), 0, 0, 1, 0))
    header_offset = len(out)
    out += b''.join(headers)
    if 52 + 32 * len(programs) > 0x1000:
        raise ValueError('too many overlay sections')
    out[:52] = struct.pack('<16sHHIIIIIHHHHHH', b'\x7fELF\x01\x01\x01' + bytes(9),
                          2, 8, 1, entry, 52, header_offset, 0x20924001,
                          52, 32, len(programs), 40, len(headers), len(headers) - 1)
    out[52:52 + 32 * len(programs)] = b''.join(programs)
    return bytes(out)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('level_wad', type=Path)
    parser.add_argument('private_output_elf', type=Path)
    args = parser.parse_args()
    wad = args.level_wad.read_bytes()
    entry, sections, overlay = overlay_sections(wad)
    elf = analysis_elf(entry, sections)
    # Never overwrite an existing analysis/checkpoint.
    with args.private_output_elf.open('xb') as out:
        out.write(elf)
    receipt = {'role': 'PRIVATE_STATIC_ANALYSIS_ONLY_NOT_ORIGINAL_ELF_NOT_RUNTIME',
               'source_wad_sha256': hashlib.sha256(wad).hexdigest(),
               'overlay_sha256': hashlib.sha256(overlay).hexdigest(),
               'derived_elf_sha256': hashlib.sha256(elf).hexdigest(), 'entry': hex(entry),
               'sections': [{'address': hex(a), 'size': len(b), 'type': k,
                             'payload_sha256': hashlib.sha256(b).hexdigest()} for a, k, b in sections],
               'boot_dependencies_included': False,
               'NOBITS': 'memory-only; source payload not emitted as initialized bytes'}
    args.private_output_elf.with_suffix('.provenance.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(f'Private analysis ELF: {len(sections)} sections; entry {entry:#x}; no runtime execution')


if __name__ == '__main__':
    main()
