import importlib.util
from pathlib import Path
import unittest
p=Path(__file__).resolve().parents[1]/'rc2_initializer_inventory.py'
spec=importlib.util.spec_from_file_location('inventory',p)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class InventoryTests(unittest.TestCase):
    def world(self):
        return {'provenance':'SYNTHETIC_METADATA_TEST','initializers':[
            {'ordinal':0,'owner':3,'class':'0010','source':'00001000'},
            {'ordinal':1,'owner':4,'class':'0011','source':'00002000'},
            {'ordinal':2,'owner':6,'class':'0012','source':'00001000'}]}
    def test_groups_preserve_loader_owner_order_and_shared_services(self):
        calls=[{'caller':'sub_00001000','callee':'sub_00003000','site':'0x1008'},
               {'caller':'sub_00002000','callee':'sub_00003000','site':'0x2004'},
               {'caller':'sub_00001000','callee':'sub_00004000','site':'0x1004'}]
        r=m.inventory(self.world(),calls)
        self.assertEqual(r['callback_groups'][0]['owners'],[3,6])
        self.assertEqual(r['callback_groups'][0]['native_scope'],'UNKNOWN_REQUIRES_EVIDENCE')
        self.assertEqual([c['callee'] for c in r['callback_groups'][0]['direct_static_calls']],['00004000','00003000'])
        self.assertEqual(r['shared_direct_services'],[{'service':'00003000','callback_sources':['00001000','00002000']}])
    def test_original_named_external_is_preserved_without_inventing_address(self):
        r=m.inventory(self.world(),[{'caller':'sub_00001000','callee':'DPRINT','site':'0x1000'}])
        self.assertEqual(r['callback_groups'][0]['direct_static_calls'],[{'site':'00001000','callee':'symbol:DPRINT'}])
    def test_reordered_or_duplicate_owners_rejected(self):
        w=self.world();w['initializers'][1]['owner']=3
        with self.assertRaises(ValueError):m.inventory(w,[])
        w=self.world();w['initializers'][1]['ordinal']=7
        with self.assertRaises(ValueError):m.inventory(w,[])
    def test_null_callback_and_state_scope_are_not_complete_support(self):
        w=self.world();w['initializers'][0]['source']=None;w['initializers'][1]['source']='003846d0'
        r=m.inventory(w,[])
        self.assertEqual(r['callback_groups'][0]['native_scope'],'UNKNOWN_REQUIRES_EVIDENCE')
        self.assertIn('PREFIX_ONLY',r['callback_groups'][1]['native_scope'])

if __name__=='__main__':unittest.main()
