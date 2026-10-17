#!/usr/bin/env python3
"""Retain symbols only when their UUID exactly matches the packaged executable."""
import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess


def parse_uuid(text):
    rows = re.findall(r'^UUID: ([0-9A-Fa-f-]{36}) \(([^)]+)\)', text, re.M)
    if len(rows) != 1 or rows[0][1] != 'arm64':
        raise ValueError('one ARM64 UUID required')
    return rows[0][0].lower()


def check_pair(binary_text, dwarf_text):
    binary, dwarf = parse_uuid(binary_text), parse_uuid(dwarf_text)
    if binary != dwarf:
        raise ValueError('dSYM UUID does not match packaged executable')
    return binary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--rust-binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    target = args.output/'ReRAC.app.dSYM'
    if target.exists():
        raise ValueError('refuse to overwrite prior symbols')
    packed = args.rust_binary.with_name(args.rust_binary.name+'.dSYM')
    if packed.is_dir():
        shutil.copytree(packed, target)
        dwarf_root = target/'Contents/Resources/DWARF'
        files = list(dwarf_root.iterdir())
        if len(files) != 1:
            raise ValueError('one Rust DWARF file required')
        if files[0].name != 'ReRAC':
            files[0].rename(dwarf_root/'ReRAC')
    else:
        subprocess.run(['xcrun','dsymutil',str(args.app/'ReRAC'),'-o',str(target)],check=True)
    dwarf = target/'Contents/Resources/DWARF/ReRAC'
    if not dwarf.is_file() or dwarf.stat().st_size < 4096:
        raise ValueError('missing or empty DWARF payload')
    command = ['xcrun','dwarfdump','--uuid']
    binary_text = subprocess.check_output(command+[str(args.app/'ReRAC')],text=True)
    dwarf_text = subprocess.check_output(command+[str(dwarf)],text=True)
    uuid = check_pair(binary_text, dwarf_text)
    sections = subprocess.check_output(['otool','-l',str(dwarf)],text=True)
    for section in ('__debug_info','__debug_line'):
        if section not in sections:
            raise ValueError('required DWARF section absent: '+section)
    manifest = dict(executable_uuid=uuid,dsym_uuid=uuid,architecture='arm64',
                    match='PASS',dwarf_bytes=dwarf.stat().st_size)
    args.evidence.mkdir(parents=True,exist_ok=True)
    (args.evidence/'symbols-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    (args.output/'symbols-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps(manifest))


if __name__ == '__main__':
    main()
