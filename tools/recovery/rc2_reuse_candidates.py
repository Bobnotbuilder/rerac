#!/usr/bin/env python3
"""Bounded original-binary candidate scan using upstream's Ghidra normalizer.

Four-instruction leaves only. Output is evidence for investigation, never proof
of behavior, function identity or a production reuse decision. The scan retains
absolute guest bindings hidden by the upstream relocation mask.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import struct

from rc2_leaf_aot import initialized_elf_sections, SIGNATURES, verify


def load_normalizer(path):
    spec = importlib.util.spec_from_file_location('upstream_crossgame_normalizer', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def normalized(module, words):
    return tuple(module.sched_norm(module.frame_norm(module.mask(words))))


def memory_binding(words):
    # Reviewed leaves use LUI followed by LBU/SH with the same base register.
    # Return None for any other body; this is not a general address analysis.
    hi, low = words[0], words[1] if words[1] >> 26 == 0x24 else words[3]
    if hi >> 26 != 15 or low >> 26 not in (0x24, 0x29):
        return None
    if (hi >> 16) & 31 != (low >> 21) & 31:
        return None
    offset = low & 0xffff
    if offset & 0x8000:
        offset -= 0x10000
    return (((hi & 0xffff) << 16) + offset) & 0xffffffff


def scan(module, sections, rc2_bodies):
    results = []
    keys = {normalized(module, words): (address, words) for address, words in rc2_bodies.items()}
    for base, data in sections:
        for offset in range(0, len(data) - 15, 4):
            words = struct.unpack_from('<4I', data, offset)
            # Preserve exact control-flow and delay-slot placement. Normalizer
            # drops NOPs/branch offsets; neither is accepted as equivalence.
            if words[2] != 0x03e00008:
                continue
            key = normalized(module, words)
            if key not in keys:
                continue
            rc2_address, rc2_words = keys[key]
            a, b = memory_binding(words), memory_binding(rc2_words)
            results.append({
                'rc2_address': f'{rc2_address:08x}', 'rac1_candidate_address': f'{base + offset:08x}',
                'rac1_memory_binding': None if a is None else f'{a:08x}',
                'rc2_memory_binding': None if b is None else f'{b:08x}',
                'same_absolute_binding': a == b and a is not None,
                'reuse_decision': 'UNKNOWN_REQUIRES_EVIDENCE',
                'function_boundary_verified': False,
                'candidate_only': True,
            })
    return results


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--normalizer', type=Path, required=True)
    ap.add_argument('--rac1-elf', type=Path, required=True)
    ap.add_argument('--rc2-elf', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    a = ap.parse_args()
    r1, r2 = a.rac1_elf.read_bytes(), a.rc2_elf.read_bytes()
    r2_sections = initialized_elf_sections(r2)
    verify(r2_sections)
    bodies = {address: struct.unpack('<4I', data[address-base:address-base+16])
              for address in SIGNATURES for base, data in r2_sections
              if base <= address and address+16 <= base+len(data)}
    candidates = scan(load_normalizer(a.normalizer), initialized_elf_sections(r1), bodies)
    a.out.write_text(json.dumps({
        'status': 'ORIGINAL_BINARY_CANDIDATES_NOT_SEMANTIC_COMPATIBILITY',
        'normalizer_sha256': hashlib.sha256(a.normalizer.read_bytes()).hexdigest(),
        'rac1_elf_sha256': hashlib.sha256(r1).hexdigest(),
        'rc2_elf_sha256': hashlib.sha256(r2).hexdigest(),
        'candidate_count': len(candidates), 'candidates': candidates,
        'limitations': ['RAC1 boot only, not all gameplay overlays',
                        'Ghidra function boundaries/call graph not present in this input',
                        'All normalized matches require data-lifetime/ownership/callee review'],
    }, indent=2) + '\n')


if __name__ == '__main__':
    main()
