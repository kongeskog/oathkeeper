import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import shutil
from unittest.mock import patch
import subprocess
import oathkeeper

ROOT=Path(__file__).resolve().parents[2]

def policy(operation,**fields):
    return json.loads(oathkeeper.assess_policy(json.dumps({'operation':operation,**fields})))

class PolicyTests(unittest.TestCase):
    def context(self):
        c=json.loads(oathkeeper.compile_file(str(ROOT/'.deps/lawlib/us/wa/seattle/cases/long-ability-to-pay.case.oath')))
        b={**c['module'],**c['case']};b['constants']=c['module'].get('constants',[])+c['case'].get('constants',[])
        return b
    def test_frozen_native_api_and_scope_certificates(self):
        b=self.context();coverage=policy('coverage',bundle=b)
        self.assertEqual(coverage['status'],'conditions_supported')
        proofs=[{'id':c['id'],'root':c['supporting'][0]['proof'][-1]['id'],'proof':c['supporting'][0]['proof']} for c in coverage['conditions']]
        task=policy('freeze',bundle=b,policy={'allow_oracles':True,'require_coverage':True})
        self.assertEqual(task['context']['proof'],[])
        result=policy('evaluate',task=task,expected_sha256=task['task_sha256'],candidate={'proof':b['proof'],'scope_proofs':proofs})
        self.assertEqual(result['reward'],1)
        self.assertEqual(policy('evaluate',task=task,expected_sha256=task['task_sha256'],candidate=b)['reward'],0)
        forged=copy.deepcopy(task);forged['context']['query']['known_at']='2027-01-01T00:00:00Z'
        with self.assertRaises(ValueError):policy('evaluate',task=forged,expected_sha256=task['task_sha256'],candidate=[])
    def test_empty_review_history_does_not_create_approval(self):
        output=policy('reviews',artifact_sha256='a'*64,journal={'format':'oathkeeper.journal/1','events':[]},trust={'keys':[]},authors=[])
        self.assertEqual(output['current'],[])
        with self.assertRaises(ValueError):policy('reviews',artifact_sha256='a'*64,journal={'format':'oathkeeper.journal/1','events':[]},trust={'keys':[]},authors=[],approved=True)

class PackageTests(unittest.TestCase):
    def test_locked_snapshot_rejects_changed_or_extra_files_and_moving_refs(self):
        spec=importlib.util.spec_from_file_location('lawlib_package',ROOT/'tools/lawlib_package.py')
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as tmp:
            base=Path(tmp);source=base/'source';source.mkdir()
            (source/'oath-package.json').write_text(json.dumps({'format':'oathkeeper.law-package/1','version':'1','repository':'https://example.com/synthetic'}))
            (source/'example.oath').write_text('// Synthetic package test\n')
            subprocess.run(['git','init','-q',str(source)],check=True)
            subprocess.run(['git','-C',str(source),'add','.'],check=True)
            subprocess.run(['git','-C',str(source),'-c','user.name=Synthetic test','-c','user.email=test@example.invalid','commit','-qm','Synthetic fixture'],check=True)
            revision=subprocess.check_output(['git','-C',str(source),'rev-parse','HEAD'],text=True).strip()
            with patch.object(module,'DEST',base/'snapshot'),patch.object(module,'LOCK',base/'lock.json'):
                self.assertEqual(module.pin(source,revision)['status'],'verified')
                with self.assertRaises(ValueError):module.pin(source,'main')
                locked=module.LOCK.read_bytes()
                shutil.rmtree(module.DEST)
                with self.assertRaisesRegex(ValueError,'dependency is missing'):
                    module.verify()
                self.assertEqual(module.sync(source)['revision'],revision)
                self.assertEqual(module.LOCK.read_bytes(),locked)
                with patch.object(module.subprocess,'run',side_effect=AssertionError('verified sync must be offline')),patch.object(module.subprocess,'check_output',side_effect=AssertionError('verified sync must be offline')):
                    self.assertEqual(module.sync()['status'],'verified')
                (module.DEST/'extra').write_text('unexpected')
                with self.assertRaises(ValueError):module.verify()
                (module.DEST/'extra').unlink()
                (module.DEST/'example.oath').write_text('changed')
                with self.assertRaises(ValueError):module.verify()
                with self.assertRaises(ValueError):module.sync(source)
                self.assertEqual((module.DEST/'example.oath').read_text(),'changed')
                shutil.rmtree(module.DEST)
                invalid=json.loads(locked)
                invalid['files']['example.oath']='0'*64
                module.LOCK.write_text(json.dumps(invalid))
                with self.assertRaisesRegex(ValueError,'differs from its immutable lock'):
                    module.sync(source)
                self.assertFalse(module.DEST.exists())
    def test_split_grouping_rejects_cross_matter_leakage(self):
        import sys
        sys.dont_write_bytecode = True
        sys.path.insert(0,str(ROOT/'.deps/lawlib/tools'))
        from check import check_splits
        entry={'matter':'synthetic','source_family':'synthetic.family','amendment_lineage':'synthetic.lineage'}
        with self.assertRaises(ValueError):check_splits({'train':[entry],'held_out':[entry]})
        check_splits({'development':[entry],'held_out':[]})

if __name__=='__main__':unittest.main()
