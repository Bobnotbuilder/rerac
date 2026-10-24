import importlib.util
from pathlib import Path
import struct
import unittest

spec = importlib.util.spec_from_file_location('overlay_tool', Path(__file__).parents[1] / 'rc2_overlay_to_elf.py')
tool = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tool)


def wad_fixture():
    overlay = b''.join(struct.pack('<4I', a, len(b), k, 0x200008) + b for a, k, b in
                       [(0x180000, 8, b'private-bss-data'), (0x200000, 1, bytes(range(32)))])
    data = struct.pack('<2i', 0x80, len(overlay)) + bytes(0x78) + overlay
    wad = bytearray(2048) + data
    struct.pack_into('<I', wad, 0, 0x60)
    struct.pack_into('<2I', wad, 0x10, 1, 1)
    return wad


class OverlayAnalysisTests(unittest.TestCase):
    def test_preserves_original_initialized_code_and_nobits_memory_only(self):
        entry, sections, overlay = tool.overlay_sections(wad_fixture())
        elf = tool.analysis_elf(entry, sections)
        self.assertEqual(struct.unpack_from('<I', elf, 24)[0], 0x200008)
        for i, (address, kind, data) in enumerate(sections):
            tag, off, va, pa, filesz, memsz, flags, align = struct.unpack_from('<8I', elf, 52 + i * 32)
            self.assertEqual((va, memsz), (address, len(data)))
            self.assertEqual(filesz, 0 if kind == 8 else len(data))
            if kind == 1:
                self.assertEqual(elf[off:off + filesz], data)
                self.assertEqual(flags, 5)
        self.assertNotIn(b'private-bss-data', elf)

    def test_rejects_truncated_and_outside_data_overlay_extents(self):
        with self.assertRaises(ValueError):
            tool.overlay_sections(wad_fixture()[:0x810])
        w = wad_fixture()
        struct.pack_into('<i', w, 2048, -1)
        with self.assertRaises(ValueError):
            tool.overlay_sections(w)

    def test_rejects_entry_in_nobits(self):
        w = wad_fixture()
        struct.pack_into('<I', w, 2048 + 0x80 + 12, 0x180000)
        with self.assertRaises(ValueError):
            tool.overlay_sections(w)

    def test_rejects_overlapping_sections(self):
        w = wad_fixture()
        struct.pack_into('<I', w, 2048 + 0x80 + 16 + len(b'private-bss-data'), 0x180004)
        with self.assertRaises(ValueError):
            tool.overlay_sections(w)


if __name__ == '__main__':
    unittest.main()
