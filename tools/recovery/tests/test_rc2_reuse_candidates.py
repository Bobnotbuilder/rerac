"""Candidate matches must retain the data differences a relocation mask hides."""
import importlib.util
from pathlib import Path
import struct
import sys
import unittest

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
spec = importlib.util.spec_from_file_location('reuse', TOOLS / 'rc2_reuse_candidates.py')
reuse = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reuse)


class SyntheticNormalizer:
    @staticmethod
    def mask(words):
        return [w & 0xffff0000 if w >> 26 in (15, 0x24, 0x29) else w for w in words]
    @staticmethod
    def frame_norm(words):
        return words
    @staticmethod
    def sched_norm(words):
        return words


# Synthetic leaves preserve the forms under test without publishing original bodies.
BODIES = {0x1000: (0x3c02001c, 0x90420010, 0x03e00008, 0x0002102b),
          0x2000: (0x3c030014, 0x24020002, 0x03e00008, 0xa4620010)}


class CandidateTests(unittest.TestCase):
    def test_hidden_absolute_binding_difference_is_not_compatible(self):
        words = list(BODIES[0x2000])
        words[3] = (words[3] & 0xffff0000) | 0x16f0
        result = reuse.scan(SyntheticNormalizer, [(0x216b08, struct.pack('<4I', *words))], BODIES)
        self.assertEqual(len(result), 1)
        self.assertEqual(result[0]['rac1_memory_binding'], '001416f0')
        self.assertEqual(result[0]['rc2_memory_binding'], '00140010')
        self.assertFalse(result[0]['same_absolute_binding'])
        self.assertEqual(result[0]['reuse_decision'], 'UNKNOWN_REQUIRES_EVIDENCE')
        self.assertFalse(result[0]['function_boundary_verified'])

    def test_exact_bytes_are_still_candidate_only_without_context(self):
        words = BODIES[0x1000]
        result = reuse.scan(SyntheticNormalizer, [(0x111000, struct.pack('<4I', *words))], BODIES)
        self.assertTrue(result[0]['same_absolute_binding'])
        self.assertTrue(result[0]['candidate_only'])
        self.assertEqual(result[0]['reuse_decision'], 'UNKNOWN_REQUIRES_EVIDENCE')

    def test_non_return_and_truncated_input_never_match(self):
        words = list(BODIES[0x1000])
        words[2] = 0
        self.assertEqual(reuse.scan(SyntheticNormalizer, [(0x111000, struct.pack('<4I', *words))], BODIES), [])
        self.assertEqual(reuse.scan(SyntheticNormalizer, [(0x111000, b'\0' * 15)], BODIES), [])

    def test_negative_low_immediate_retains_sign_extension(self):
        words = (0x3c02001b, 0x9042ffff, 0x03e00008, 0x0002102b)
        self.assertEqual(reuse.memory_binding(words), 0x1affff)


if __name__ == '__main__':
    unittest.main()
