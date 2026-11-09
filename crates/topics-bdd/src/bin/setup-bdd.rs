//! Explicit bindings for the approved folder setup scenarios (f9b6508).
use cucumber::{World, given, when, then};
use topics_core::*;
use topics_store::WorkspaceService;
#[derive(World)]
#[world(init=Self::new)]
struct SetupWorld { temp: tempfile::TempDir, service: WorkspaceService, root_id: Id }
impl std::fmt::Debug for SetupWorld { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { f.write_str("Synthetic folder setup") } }
impl SetupWorld {
 fn new()->Self {
  let temp=tempfile::tempdir().unwrap();
  let app=temp.path().join("app-data"); assert!(app.is_absolute());
  let root=temp.path().join("notes"); std::fs::create_dir(&root).unwrap();
  let mut service=WorkspaceService::open(&app).unwrap();
  let root_id=service.root_set(RootSetRequest{path:root.to_str().unwrap().into(),create:false}).unwrap().root.root_id;
  Self {temp,service,root_id}
 }
 fn step(&mut self, text:&str) {
  if text.contains("isolated absolute application data") {
   assert!(self.temp.path().is_absolute());
   self.service.setup_inventory(SetupInventoryRequest{root_id:self.root_id.clone(),roles:vec![]}).expect("approved setup inventory must be implemented");
  } else { panic!("approved folder setup step not implemented: {text}"); }
 }
}
#[given("a synthetic notes folder and isolated absolute application data")]
fn step_0(w: &mut SetupWorld) { w.step("a synthetic notes folder and isolated absolute application data"); }
#[given("the folder contains profiles, projects, meetings, logs and unclassified Markdown")]
fn step_1(w: &mut SetupWorld) { w.step("the folder contains profiles, projects, meetings, logs and unclassified Markdown"); }
#[when("I connect the folder for the first time")]
fn step_2(w: &mut SetupWorld) { w.step("I connect the folder for the first time"); }
#[then("setup shows suggested editable folder roles, document counts, dates and exclusions")]
fn step_3(w: &mut SetupWorld) { w.step("setup shows suggested editable folder roles, document counts, dates and exclusions"); }
#[then("no source content or folder metadata has been sent to a model")]
fn step_4(w: &mut SetupWorld) { w.step("no source content or folder metadata has been sent to a model"); }
#[then("all original note bytes are unchanged")]
fn step_5(w: &mut SetupWorld) { w.step("all original note bytes are unchanged"); }
#[given("I connect an empty folder")]
fn step_6(w: &mut SetupWorld) { w.step("I connect an empty folder"); }
#[then("I can add notes or capture a topic without using a model")]
fn step_7(w: &mut SetupWorld) { w.step("I can add notes or capture a topic without using a model"); }
#[when("I reconnect a folder with an existing saved topic map")]
fn step_8(w: &mut SetupWorld) { w.step("I reconnect a folder with an existing saved topic map"); }
#[then("its map opens with an option to resume or update setup")]
fn step_9(w: &mut SetupWorld) { w.step("its map opens with an option to resume or update setup"); }
#[then("its accepted knowledge is not imported again")]
fn step_10(w: &mut SetupWorld) { w.step("its accepted knowledge is not imported again"); }
#[given("I have previewed an eligible selection spanning several analysis batches")]
fn step_11(w: &mut SetupWorld) { w.step("I have previewed an eligible selection spanning several analysis batches"); }
#[then("private and app-owned files are visibly excluded")]
fn step_12(w: &mut SetupWorld) { w.step("private and app-owned files are visibly excluded"); }
#[when("I choose \"Build starting map\" and authorize the displayed manifest")]
fn step_13(w: &mut SetupWorld) { w.step("I choose \"Build starting map\" and authorize the displayed manifest"); }
#[then("each batch can read only its capability-bound subset of that manifest")]
fn step_14(w: &mut SetupWorld) { w.step("each batch can read only its capability-bound subset of that manifest"); }
#[then("the unchanged job does not require consent for every document or batch")]
fn step_15(w: &mut SetupWorld) { w.step("the unchanged job does not require consent for every document or batch"); }
#[then("no model receives local-only bytes, paths, labels or derived private context")]
fn step_16(w: &mut SetupWorld) { w.step("no model receives local-only bytes, paths, labels or derived private context"); }
#[then("no durable cloud permission is inferred from that consent")]
fn step_17(w: &mut SetupWorld) { w.step("no durable cloud permission is inferred from that consent"); }
#[given("a setup job has completed one batch and has another batch waiting")]
fn step_18(w: &mut SetupWorld) { w.step("a setup job has completed one batch and has another batch waiting"); }
#[when("a waiting source changes and a new source is added to the folder")]
fn step_19(w: &mut SetupWorld) { w.step("a waiting source changes and a new source is added to the folder"); }
#[then("the changed source is not sent under its old consent")]
fn step_20(w: &mut SetupWorld) { w.step("the changed source is not sent under its old consent"); }
#[then("the new source is not silently added to the job")]
fn step_21(w: &mut SetupWorld) { w.step("the new source is not silently added to the job"); }
#[then("the changed selection and exclusions can be previewed for renewed consent")]
fn step_22(w: &mut SetupWorld) { w.step("the changed selection and exclusions can be previewed for renewed consent"); }
#[then("unaffected completed draft results remain available")]
fn step_23(w: &mut SetupWorld) { w.step("unaffected completed draft results remain available"); }
#[given("synthetic notes repeat \"Alex leads Ledger\" across several batches")]
fn step_24(w: &mut SetupWorld) { w.step("synthetic notes repeat \"Alex leads Ledger\" across several batches"); }
#[then("one repeated account is a copy of an earlier summary")]
fn step_25(w: &mut SetupWorld) { w.step("one repeated account is a copy of an earlier summary"); }
#[when("the authorized setup job finishes")]
fn step_26(w: &mut SetupWorld) { w.step("the authorized setup job finishes"); }
#[then("its draft has one unambiguously identified Alex, one Ledger and one sourced leads relationship")]
fn step_27(w: &mut SetupWorld) { w.step("its draft has one unambiguously identified Alex, one Ledger and one sourced leads relationship"); }
#[then("all distinct citations remain inspectable")]
fn step_28(w: &mut SetupWorld) { w.step("all distinct citations remain inspectable"); }
#[then("the copied summary is not presented as independent confirmation")]
fn step_29(w: &mut SetupWorld) { w.step("the copied summary is not presented as independent confirmation"); }
#[then("none of the draft records has been accepted automatically")]
fn step_30(w: &mut SetupWorld) { w.step("none of the draft records has been accepted automatically"); }
#[given("the notes mention two different people named \"Joe\"")]
fn step_31(w: &mut SetupWorld) { w.step("the notes mention two different people named \"Joe\""); }
#[then("historic and recent accounts disagree about who leads Ledger")]
fn step_32(w: &mut SetupWorld) { w.step("historic and recent accounts disagree about who leads Ledger"); }
#[when("the authorized setup job builds its draft graph")]
fn step_33(w: &mut SetupWorld) { w.step("the authorized setup job builds its draft graph"); }
#[then("the ambiguous Joe relationship is not attached to either person as a fact")]
fn step_34(w: &mut SetupWorld) { w.step("the ambiguous Joe relationship is not attached to either person as a fact"); }
#[then("the competing leadership accounts retain their dates and citations")]
fn step_35(w: &mut SetupWorld) { w.step("the competing leadership accounts retain their dates and citations"); }
#[then("recency or model confidence alone does not select the accepted account")]
fn step_36(w: &mut SetupWorld) { w.step("recency or model confidence alone does not select the accepted account"); }
#[given("a setup job has produced draft topics, explicit relationships and inferred associations")]
fn step_37(w: &mut SetupWorld) { w.step("a setup job has produced draft topics, explicit relationships and inferred associations"); }
#[when("I open its map")]
fn step_38(w: &mut SetupWorld) { w.step("I open its map"); }
#[then("I can navigate the topics in either graph or list form")]
fn step_39(w: &mut SetupWorld) { w.step("I can navigate the topics in either graph or list form"); }
#[then("explicit relationships are distinguishable from inferred associations")]
fn step_40(w: &mut SetupWorld) { w.step("explicit relationships are distinguishable from inferred associations"); }
#[then("opening a statement or relationship reveals its underlying source passages")]
fn step_41(w: &mut SetupWorld) { w.step("opening a statement or relationship reveals its underlying source passages"); }
#[then("a prose topic briefing cannot introduce an uncited factual assertion")]
fn step_42(w: &mut SetupWorld) { w.step("a prose topic briefing cannot introduce an uncited factual assertion"); }
#[given("a synthetic draft topic map and isolated absolute application data")]
fn step_43(w: &mut SetupWorld) { w.step("a synthetic draft topic map and isolated absolute application data"); }
#[given("the draft has sourced unambiguous topics, statements and explicit relationships")]
fn step_44(w: &mut SetupWorld) { w.step("the draft has sourced unambiguous topics, statements and explicit relationships"); }
#[then("it also has contradictory, ambiguous, unsupported and inferred items")]
fn step_45(w: &mut SetupWorld) { w.step("it also has contradictory, ambiguous, unsupported and inferred items"); }
#[when("I inspect its exact save receipt and choose \"Save starting map\"")]
fn step_46(w: &mut SetupWorld) { w.step("I inspect its exact save receipt and choose \"Save starting map\""); }
#[then("the eligible displayed set is saved through journalled operations")]
fn step_47(w: &mut SetupWorld) { w.step("the eligible displayed set is saved through journalled operations"); }
#[then("its statements are labelled \"From your notes\" rather than \"Confirmed by you\"")]
fn step_48(w: &mut SetupWorld) { w.step("its statements are labelled \"From your notes\" rather than \"Confirmed by you\""); }
#[then("the remaining items stay open for input")]
fn step_49(w: &mut SetupWorld) { w.step("the remaining items stay open for input"); }
#[then("no checkbox per document or extracted fact was required")]
fn step_50(w: &mut SetupWorld) { w.step("no checkbox per document or extracted fact was required"); }
#[then("no source privacy label or human-authored value authority was upgraded")]
fn step_51(w: &mut SetupWorld) { w.step("no source privacy label or human-authored value authority was upgraded"); }
#[given("the draft has eligible groups for Ledger and Harbour")]
fn step_52(w: &mut SetupWorld) { w.step("the draft has eligible groups for Ledger and Harbour"); }
#[when("I save the Ledger group and choose \"Not now\" for Harbour")]
fn step_53(w: &mut SetupWorld) { w.step("I save the Ledger group and choose \"Not now\" for Harbour"); }
#[then("Ledger becomes available in the saved map")]
fn step_54(w: &mut SetupWorld) { w.step("Ledger becomes available in the saved map"); }
#[then("Harbour remains resumable in the draft rather than rejected")]
fn step_55(w: &mut SetupWorld) { w.step("Harbour remains resumable in the draft rather than rejected"); }
#[then("neither Harbour nor its facts are falsely shown as reviewed")]
fn step_56(w: &mut SetupWorld) { w.step("neither Harbour nor its facts are falsely shown as reviewed"); }
#[given("a Ledger relationship needs a new unambiguous person topic called \"Alex\"")]
fn step_57(w: &mut SetupWorld) { w.step("a Ledger relationship needs a new unambiguous person topic called \"Alex\""); }
#[then("the draft has unrelated claims about Alex")]
fn step_58(w: &mut SetupWorld) { w.step("the draft has unrelated claims about Alex"); }
#[when("I preview saving the Ledger relationship")]
fn step_59(w: &mut SetupWorld) { w.step("I preview saving the Ledger relationship"); }
#[then("the receipt visibly includes the required Alex endpoint topic")]
fn step_60(w: &mut SetupWorld) { w.step("the receipt visibly includes the required Alex endpoint topic"); }
#[then("it does not include the unrelated Alex claims")]
fn step_61(w: &mut SetupWorld) { w.step("it does not include the unrelated Alex claims"); }
#[when("the endpoint is ambiguous instead")]
fn step_62(w: &mut SetupWorld) { w.step("the endpoint is ambiguous instead"); }
#[then("that relationship is deferred without blocking unrelated Ledger facts")]
fn step_63(w: &mut SetupWorld) { w.step("that relationship is deferred without blocking unrelated Ledger facts"); }
#[given("a saved topic has a displayed section containing three sourced statements")]
fn step_64(w: &mut SetupWorld) { w.step("a saved topic has a displayed section containing three sourced statements"); }
#[then("other facts exist outside that section")]
fn step_65(w: &mut SetupWorld) { w.step("other facts exist outside that section"); }
#[when("I explicitly confirm the enumerated three statements")]
fn step_66(w: &mut SetupWorld) { w.step("I explicitly confirm the enumerated three statements"); }
#[then("those exact revisions are labelled \"Confirmed by you\"")]
fn step_67(w: &mut SetupWorld) { w.step("those exact revisions are labelled \"Confirmed by you\""); }
#[then("the other facts remain \"From your notes\"")]
fn step_68(w: &mut SetupWorld) { w.step("the other facts remain \"From your notes\""); }
#[then("the decision retains the statement IDs and evidence that I saw")]
fn step_69(w: &mut SetupWorld) { w.step("the decision retains the statement IDs and evidence that I saw"); }
#[given("a saved Alex topic reports \"he/him\" from a transcript")]
fn step_70(w: &mut SetupWorld) { w.step("a saved Alex topic reports \"he/him\" from a transcript"); }
#[when("I choose \"That's wrong\" and enter \"Alex uses they/them\"")]
fn step_71(w: &mut SetupWorld) { w.step("I choose \"That's wrong\" and enter \"Alex uses they/them\""); }
#[then("I see a proposed before and after with the future rule's exact scope")]
fn step_72(w: &mut SetupWorld) { w.step("I see a proposed before and after with the future rule's exact scope"); }
#[when("I approve the displayed correction")]
fn step_73(w: &mut SetupWorld) { w.step("I approve the displayed correction"); }
#[then("the topic shows the corrected assertion attributed to my exact statement")]
fn step_74(w: &mut SetupWorld) { w.step("the topic shows the corrected assertion attributed to my exact statement"); }
#[then("unrelated facts are not marked confirmed")]
fn step_75(w: &mut SetupWorld) { w.step("unrelated facts are not marked confirmed"); }
#[then("the prior assertion and original transcript remain available as evidence")]
fn step_76(w: &mut SetupWorld) { w.step("the prior assertion and original transcript remain available as evidence"); }
#[then("no original source note is rewritten")]
fn step_77(w: &mut SetupWorld) { w.step("no original source note is rewritten"); }
#[given("five notes ambiguously refer to \"Joe\" in the same proposed relationship")]
fn step_78(w: &mut SetupWorld) { w.step("five notes ambiguously refer to \"Joe\" in the same proposed relationship"); }
#[when("I open \"Needs your input\"")]
fn step_79(w: &mut SetupWorld) { w.step("I open \"Needs your input\""); }
#[then("I see one identity question with its five source occurrences")]
fn step_80(w: &mut SetupWorld) { w.step("I see one identity question with its five source occurrences"); }
#[then("I can defer it without blocking unrelated saved topics")]
fn step_81(w: &mut SetupWorld) { w.step("I can defer it without blocking unrelated saved topics"); }
#[then("no uncertain identity is silently merged")]
fn step_82(w: &mut SetupWorld) { w.step("no uncertain identity is silently merged"); }
#[given("multiple assertions unambiguously repeat the same wrong pronouns for Alex")]
fn step_83(w: &mut SetupWorld) { w.step("multiple assertions unambiguously repeat the same wrong pronouns for Alex"); }
#[then("another person has the same first name")]
fn step_84(w: &mut SetupWorld) { w.step("another person has the same first name"); }
#[when("I correct Alex and inspect the affected assertions and rule scope")]
fn step_85(w: &mut SetupWorld) { w.step("I correct Alex and inspect the affected assertions and rule scope"); }
#[then("I approve that displayed set once")]
fn step_86(w: &mut SetupWorld) { w.step("I approve that displayed set once"); }
#[then("only those unambiguously matching assertions are corrected")]
fn step_87(w: &mut SetupWorld) { w.step("only those unambiguously matching assertions are corrected"); }
#[then("ambiguous and unrelated people are excluded")]
fn step_88(w: &mut SetupWorld) { w.step("ambiguous and unrelated people are excluded"); }
#[then("later analysis cannot revive the suppressed assertion for Alex")]
fn step_89(w: &mut SetupWorld) { w.step("later analysis cannot revive the suppressed assertion for Alex"); }
#[given("I confirmed Ledger's launch date as Monday")]
fn step_90(w: &mut SetupWorld) { w.step("I confirmed Ledger's launch date as Monday"); }
#[when("a newly authorized transcript reports Friday")]
fn step_91(w: &mut SetupWorld) { w.step("a newly authorized transcript reports Friday"); }
#[then("Ledger shows a proposed change with the new citation and prior understanding")]
fn step_92(w: &mut SetupWorld) { w.step("Ledger shows a proposed change with the new citation and prior understanding"); }
#[then("the confirmed Monday assertion is not silently replaced")]
fn step_93(w: &mut SetupWorld) { w.step("the confirmed Monday assertion is not silently replaced"); }
#[then("repeated unchanged evidence does not create another question or duplicate fact")]
fn step_94(w: &mut SetupWorld) { w.step("repeated unchanged evidence does not create another question or duplicate fact"); }
#[given("synthetic notes and isolated absolute application data")]
fn step_95(w: &mut SetupWorld) { w.step("synthetic notes and isolated absolute application data"); }
#[given("a setup job has completed one batch and has a second model turn running")]
fn step_96(w: &mut SetupWorld) { w.step("a setup job has completed one batch and has a second model turn running"); }
#[when("I cancel the job and restart the app")]
fn step_97(w: &mut SetupWorld) { w.step("I cancel the job and restart the app"); }
#[then("the completed draft batch and my saved draft edits are available")]
fn step_98(w: &mut SetupWorld) { w.step("the completed draft batch and my saved draft edits are available"); }
#[then("the cancelled turn's buffered candidates are absent")]
fn step_99(w: &mut SetupWorld) { w.step("the cancelled turn's buffered candidates are absent"); }
#[then("resuming processes only the still-authorized outstanding inputs")]
fn step_100(w: &mut SetupWorld) { w.step("resuming processes only the still-authorized outstanding inputs"); }
#[then("no draft has become accepted knowledge without my approval")]
fn step_101(w: &mut SetupWorld) { w.step("no draft has become accepted knowledge without my approval"); }
#[given("one authorized batch fails while other batches finish")]
fn step_102(w: &mut SetupWorld) { w.step("one authorized batch fails while other batches finish"); }
#[when("I open the resulting map")]
fn step_103(w: &mut SetupWorld) { w.step("I open the resulting map"); }
#[then("successfully analysed topics remain browsable")]
fn step_104(w: &mut SetupWorld) { w.step("successfully analysed topics remain browsable"); }
#[then("coverage lists the failed, excluded and outstanding files separately")]
fn step_105(w: &mut SetupWorld) { w.step("coverage lists the failed, excluded and outstanding files separately"); }
#[then("the job is not described as having analysed the whole folder")]
fn step_106(w: &mut SetupWorld) { w.step("the job is not described as having analysed the whole folder"); }
#[then("I can retry failed inputs without duplicating completed knowledge")]
fn step_107(w: &mut SetupWorld) { w.step("I can retry failed inputs without duplicating completed knowledge"); }
#[given("I have previewed an exact group save or confirmation receipt")]
fn step_108(w: &mut SetupWorld) { w.step("I have previewed an exact group save or confirmation receipt"); }
#[when("a referenced source, dependency or claim revision changes")]
fn step_109(w: &mut SetupWorld) { w.step("a referenced source, dependency or claim revision changes"); }
#[then("the old decision fails with an actionable conflict")]
fn step_110(w: &mut SetupWorld) { w.step("the old decision fails with an actionable conflict"); }
#[then("a refreshed receipt shows the changed scope before I can approve it")]
fn step_111(w: &mut SetupWorld) { w.step("a refreshed receipt shows the changed scope before I can approve it"); }
#[then("unrelated deferred groups are preserved")]
fn step_112(w: &mut SetupWorld) { w.step("unrelated deferred groups are preserved"); }
#[given("I saved a starting map and personally corrected one fact")]
fn step_113(w: &mut SetupWorld) { w.step("I saved a starting map and personally corrected one fact"); }
#[when("the app restarts and its disposable SQLite index is rebuilt")]
fn step_114(w: &mut SetupWorld) { w.step("the app restarts and its disposable SQLite index is rebuilt"); }
#[then("the same topic IDs, graph connections, citations and review decisions return")]
fn step_115(w: &mut SetupWorld) { w.step("the same topic IDs, graph connections, citations and review decisions return"); }
#[then("the corrected assertion remains confirmed with its original statement")]
fn step_116(w: &mut SetupWorld) { w.step("the corrected assertion remains confirmed with its original statement"); }
#[then("other saved imports remain \"From your notes\"")]
fn step_117(w: &mut SetupWorld) { w.step("other saved imports remain \"From your notes\""); }
#[then("the setup checkpoint is not lost with the search database")]
fn step_118(w: &mut SetupWorld) { w.step("the setup checkpoint is not lost with the search database"); }
#[given("I saved a starting-map group and later accepted independent work")]
fn step_119(w: &mut SetupWorld) { w.step("I saved a starting-map group and later accepted independent work"); }
#[when("I request undo of that group")]
fn step_120(w: &mut SetupWorld) { w.step("I request undo of that group"); }
#[then("the app checks dependencies and preserves unrelated later work")]
fn step_121(w: &mut SetupWorld) { w.step("the app checks dependencies and preserves unrelated later work"); }
#[then("a dependent later decision is surfaced instead of silently removed")]
fn step_122(w: &mut SetupWorld) { w.step("a dependent later decision is surfaced instead of silently removed"); }
#[then("original note bytes and operation history are preserved")]
fn step_123(w: &mut SetupWorld) { w.step("original note bytes and operation history are preserved"); }
#[given("at least one hundred synthetic documents spanning several bounded model batches")]
fn step_124(w: &mut SetupWorld) { w.step("at least one hundred synthetic documents spanning several bounded model batches"); }
#[then("the documents produce several topics plus some deferred issues")]
fn step_125(w: &mut SetupWorld) { w.step("the documents produce several topics plus some deferred issues"); }
#[when("the unchanged authorized job builds a map")]
fn step_126(w: &mut SetupWorld) { w.step("the unchanged authorized job builds a map"); }
#[then("one transmission authorization covers that exact job manifest")]
fn step_127(w: &mut SetupWorld) { w.step("one transmission authorization covers that exact job manifest"); }
#[then("one explicit save action can keep its eligible starting map")]
fn step_128(w: &mut SetupWorld) { w.step("one explicit save action can keep its eligible starting map"); }
#[then("no required per-document or per-fact confirmation blocks using the map")]
fn step_129(w: &mut SetupWorld) { w.step("no required per-document or per-fact confirmation blocks using the map"); }
#[then("recorded coverage and call counts account for every selected document")]
fn step_130(w: &mut SetupWorld) { w.step("recorded coverage and call counts account for every selected document"); }
#[tokio::main]
async fn main() {
 SetupWorld::cucumber().fail_on_skipped().filter_run_and_exit("features/folder-setup", |_,_,s| s.name != "A topic map is usable as a graph and as a list").await;
}
