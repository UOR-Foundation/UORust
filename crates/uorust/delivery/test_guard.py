from pathlib import Path
import importlib.util,json,tempfile,unittest,subprocess,sys
sys.dont_write_bytecode=True
spec=importlib.util.spec_from_file_location("delivery_guard",Path(__file__).with_name("guard.py"))
guard=importlib.util.module_from_spec(spec);spec.loader.exec_module(guard)

class DeliveryGuardTests(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)
  self.authority=self.root/"authority";self.authority.write_text("pinned model")
  self.policy=json.loads((Path(__file__).resolve().parents[3]/"model/delivery-policy.json").read_text())
  self.plan={"spec":"uorust/delivery-plan/1","strategy":self.policy["strategy"],"work_class":"runtime-model-integration","scope":"entire-registry","scopes":self.policy["required_scopes"],"excluded_functions":[],"leaf_symbols":[],"finish":"all-functions-formally-lifted","authority":{"path":str(self.authority),"sha256":guard.digest(self.authority),"revision":"locked-revision"}}
  repository=Path(__file__).resolve().parents[3]
  self.plan["implementation"]={"repository":str(repository),"baseline":subprocess.check_output(["git","rev-parse","HEAD"],cwd=repository,text=True).strip(),"semantic_registry":"crates/uorust/mir/std.toml"}
  self.binding={"source":"source","tool":"tool","mir":"mir","sdk":"sdk","verdicts":"verdicts"}
 def check(self,history=None):return guard.preflight(self.policy,self.plan,self.binding,history or {})
 def rejected(self,code):
  with self.assertRaises(guard.Refused) as c:self.check()
  self.assertEqual(c.exception.code,code)
 def test_structural_plan(self):self.assertEqual(self.check()["action"],"execute")
 def test_leaf_plan(self):self.plan["leaf_symbols"]=["some::function"];self.rejected("DG_LEAF_PLAN")
 def test_manual_strategy(self):self.plan["strategy"]="manual-leaf-fixes";self.rejected("DG_STRATEGY")
 def test_partial_registry(self):self.plan["excluded_functions"]=["some::function"];self.rejected("DG_PARTIAL_SCOPE")
 def test_missing_structural_scope(self):self.plan["scopes"]=["stdlib"];self.rejected("DG_PARTIAL_SCOPE")
 def test_wrong_authority(self):self.authority.write_text("changed");self.rejected("DG_AUTHORITY")
 def test_unchanged_failure(self):
  key=guard.identity(self.binding)
  with self.assertRaises(guard.Refused) as c:self.check({key:{"status":"failed"}})
  self.assertEqual(c.exception.code,"DG_UNCHANGED_FAILURE")
 def receipt(self):
  f=self.root/"receipt.json";f.write_text(json.dumps({"sdk_verified":True,"census":{"functions":10,"lifted":3},"lost_lifted":0,"contract_complete":False}))
  artifact=self.root/"artifact";artifact.write_text("source-bound artifact")
  return {"status":"verified","report":str(f),"report_sha256":guard.digest(f),"artifacts":{str(artifact):guard.digest(artifact)}}
 def test_verified_reuse(self):
  row=self.receipt();self.assertEqual(self.check({guard.identity(self.binding):row})["action"],"reuse")
 def test_changed_receipt_rejected(self):
  row=self.receipt();Path(row["report"]).write_text("{}")
  with self.assertRaises(guard.Refused) as c:self.check({guard.identity(self.binding):row})
  self.assertEqual(c.exception.code,"DG_RECEIPT")
 def test_changed_artifact_rejected(self):
  row=self.receipt();Path(next(iter(row["artifacts"]))).write_text("changed")
  with self.assertRaises(guard.Refused) as c:self.check({guard.identity(self.binding):row})
  self.assertEqual(c.exception.code,"DG_RECEIPT")
 def test_false_completion_rejected(self):
  with self.assertRaises(guard.Refused):guard.validate_report({"sdk_verified":True,"census":{"functions":10,"lifted":3},"lost_lifted":0,"contract_complete":True})
 def test_baseline_loss_rejected(self):
  with self.assertRaises(guard.Refused):guard.validate_report({"sdk_verified":True,"census":{"functions":10,"lifted":3},"lost_lifted":1})
 def test_record_keeps_an_immutable_report(self):
  row=self.receipt();history=self.root/"history.json"
  guard.record(self.binding,history,row["report"],list(row["artifacts"]))
  Path(row["report"]).write_text("later run")
  self.assertEqual(self.check(json.loads(history.read_text()))["action"],"reuse")
 def test_no_release_completion_without_adoption(self):
  with self.assertRaises(guard.Refused):guard.validate_report({"sdk_verified":True,"census":{"functions":10,"lifted":10},"lost_lifted":0,"contract_complete":True,"release_gate_passed":True,"source_bound_adopted":False})
 def test_ui_work_is_outside_the_strategy(self):self.plan["work_class"]="consumer-ui";self.rejected("DG_STRATEGY")
 def test_missing_source_binding(self):self.binding["source"]="";self.rejected("DG_INPUT_BINDING")
 def test_manual_semantic_patch_is_rejected(self):
  repository=self.root/"implementation";repository.mkdir();registry=repository/"registry.toml"
  registry.write_text('[[model]]\npath="operation"\nterm="old meaning"\n')
  subprocess.run(["git","init","-q"],cwd=repository,check=True)
  subprocess.run(["git","add","registry.toml"],cwd=repository,check=True)
  subprocess.run(["git","-c","user.name=Fixture","-c","user.email=fixture@example.invalid","commit","-qm","baseline"],cwd=repository,check=True)
  baseline=subprocess.check_output(["git","rev-parse","HEAD"],cwd=repository,text=True).strip()
  self.plan["implementation"]={"repository":str(repository),"baseline":baseline,"semantic_registry":"registry.toml"}
  registry.write_text('[[model]]\npath="operation"\nterm="manual replacement"\n')
  self.rejected("DG_MANUAL_SEMANTICS")
 def test_test_vectors_are_not_semantic_changes(self):
  a={"model":[{"path":"p","term":"x","vectors":[["old"]]}]};b={"model":[{"path":"p","term":"x","vectors":[["new"]]}]}
  self.assertEqual(guard.semantic_registry(a),guard.semantic_registry(b))
 def test_changed_consumer_source_rejected(self):
  self.plan["source_scope"]={"rust_tree":"original"};self.binding["registered_source"]="changed"
  self.rejected("DG_PARTIAL_SCOPE")
 def test_reduced_registry_rejected(self):
  with self.assertRaises(guard.Refused):guard.validate_report({"sdk_verified":True,"source":"original","census":{"functions":9,"lifted":3},"lost_lifted":0},{"registered_functions":10,"canonical_source":"original"})
if __name__=="__main__":unittest.main()
