"""Exact-signature and native differential contracts for the bounded AOT pilot."""
import importlib.util
import hashlib
from unittest.mock import patch
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location('leaf_aot', ROOT / 'tools/recovery/rc2_leaf_aot.py')
aot = importlib.util.module_from_spec(spec)
spec.loader.exec_module(aot)


class LeafAotTests(unittest.TestCase):
    def sections(self):
        return [(0x1000, b'A' * 16), (0x2000, b'B' * 16)]

    def setUp(self):
        # Synthetic byte bodies; real signatures are verified privately.
        self.gate = patch.object(aot, 'SIGNATURES', {address: hashlib.sha256(data).hexdigest()
                                                   for address, data in self.sections()})
        self.gate.start()
        self.addCleanup(self.gate.stop)

    def test_exact_bodies_include_both_return_delay_slots(self):
        self.assertEqual(len(aot.verify(self.sections())), 2)
        self.assertTrue(all(row['includes_return_delay_slot'] for row in aot.verify(self.sections())))

    def test_each_instruction_mutation_fails_closed(self):
        for i in range(2):
            for instruction in range(4):
                sections = self.sections()
                address, data = sections[i]
                words = list(struct.unpack('<4I', data))
                words[instruction] ^= 1
                sections[i] = address, struct.pack('<4I', *words)
                with self.assertRaises(ValueError):
                    aot.verify(sections)

    def test_truncated_leaf_and_duplicate_mapping_rejected(self):
        sections = self.sections()
        with self.assertRaises(ValueError):
            aot.verify(sections[:1])
        with self.assertRaises(ValueError):
            aot.verify(sections + sections[:1])

    def test_invalid_or_non_mips_elf_rejected(self):
        for blob in (b'', b'not an executable', b'\x7fELF\x02\x01\x01' + bytes(100)):
            with self.assertRaises(ValueError):
                aot.initialized_elf_sections(blob)

    @unittest.skipUnless(shutil.which('rustc'), 'native rustc unavailable')
    def test_generated_native_code_matches_hand_port_all_leaf_inputs(self):
        # Uses the real maintained native implementation, not another copy of
        # its logic. Original code-signature verification is a separate gate.
        hand = ROOT / 'crates/rc-game2/src/state_1330.rs'
        if not hand.exists():
            self.skipTest('state_1330 hand port unavailable')
        harness = '\n#[path=' + repr(str(hand)).replace("'", '"') + '] mod hand;\n' + r'''
struct Memory { query: u8, halfword: u16, reads: std::cell::Cell<u32>, writes: u32 }
impl LeafMemory for Memory {
    fn read_u8(&self, address: u32) -> Result<u8, LeafMemoryError> {
        assert_eq!(address, 0x1a7a12); self.reads.set(self.reads.get()+1); Ok(self.query)
    }
    fn write_u16_le(&mut self, address: u32, value: u16) -> Result<(), LeafMemoryError> {
        assert_eq!(address, 0x152c26); self.halfword = value; self.writes += 1; Ok(())
    }
}
fn hand_state(q: u8, h: u16) -> hand::State1330 {
    hand::State1330 { byte_1a7a12:q, byte_1a7be8:0x77, words_139948:[-123;29], halfword_152c26:h }
}
#[test] fn exhaustive_native_differential() {
    for query in 0..=255 {
        let m = Memory { query, halfword:0xbeef, reads:std::cell::Cell::new(0), writes:0 };
        let h = hand_state(query, m.halfword);
        let mut regs = LeafScratch { v0:u64::MAX, v1:0xfedcba98_76543210 };
        query_2f51a0(&m, &mut regs).unwrap();
        assert_eq!(regs.v0, u64::from(h.query_2f51a0()));
        assert_eq!(regs.v1, 0xfedcba98_76543210);
        assert_eq!((m.reads.get(),m.writes,m.halfword), (1,0,0xbeef));
    }
    for old in 0..=u16::MAX {
        let mut m = Memory { query:0x83, halfword:old, reads:std::cell::Cell::new(0), writes:0 };
        let mut h = hand_state(m.query, old);
        let before = h.clone();
        h.service_368560();
        let mut regs = LeafScratch { v0:u64::MAX, v1:u64::MAX };
        store_368560(&mut m, &mut regs).unwrap();
        assert_eq!(m.halfword, h.halfword_152c26);
        assert_eq!(regs, LeafScratch {v0:1,v1:0x150000});
        assert_eq!((m.reads.get(),m.writes,m.query), (0,1,0x83));
        assert_eq!(h.byte_1a7a12,before.byte_1a7a12);
        assert_eq!(h.byte_1a7be8,before.byte_1a7be8);
        assert_eq!(h.words_139948,before.words_139948);
    }
}
struct MissingMemory;
impl LeafMemory for MissingMemory {
    fn read_u8(&self, _:u32)->Result<u8,LeafMemoryError>{Err(LeafMemoryError)}
    fn write_u16_le(&mut self,_:u32,_:u16)->Result<(),LeafMemoryError>{Err(LeafMemoryError)}
}
#[test] fn missing_mapping_never_returns_success() {
    let mut regs=LeafScratch{v0:0,v1:0};
    assert_eq!(query_2f51a0(&MissingMemory,&mut regs), Err(LeafMemoryError));
    assert_eq!(store_368560(&mut MissingMemory,&mut regs), Err(LeafMemoryError));
}
'''
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'pilot.rs'
            executable = Path(directory) / 'pilot-tests'
            source.write_text(aot.rust_source() + harness)
            subprocess.run(['rustc', '--edition=2021', '--test', str(source), '-o', str(executable)], check=True)
            subprocess.run([str(executable)], check=True)


if __name__ == '__main__':
    unittest.main()
