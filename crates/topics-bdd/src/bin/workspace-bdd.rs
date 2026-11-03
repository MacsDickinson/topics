//! Initial explicit RED scaffolding against f3e1ed0; replaced with service steps after recording RED.
use cucumber::{World, given, then, when};
#[derive(Debug, World)]
#[world(init = Self::new)]
struct Workspace {
    _temp: tempfile::TempDir,
}
impl Workspace {
    fn new() -> Self {
        Self {
            _temp: tempfile::tempdir().unwrap(),
        }
    }
}
#[given("a synthetic notes root with absolute isolated application data")]
fn step_0(world: &mut Workspace) {
    assert!(world._temp.path().is_absolute());
}
#[given("no notes root is configured")]
fn step_1(_: &mut Workspace) {
    panic!("Missing workspace behavior: no notes root is configured");
}
#[when("I create an empty notes root from the desktop")]
fn step_2(_: &mut Workspace) {
    panic!("Missing workspace behavior: I create an empty notes root from the desktop");
}
#[then("the root is selected and an empty-state explains how to add notes")]
fn step_3(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the root is selected and an empty-state explains how to add notes"
    );
}
#[given(
    "dated notes \"Meetings/one.md\" on \"2026-11-01\" and \"Meetings/two.md\" on \"2026-11-03\""
)]
fn step_4(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: dated notes \"Meetings/one.md\" on \"2026-11-01\" and \"Meetings/two.md\" on \"2026-11-03\""
    );
}
#[given("an undated note \"Meetings/undated.md\"")]
fn step_5(_: &mut Workspace) {
    panic!("Missing workspace behavior: an undated note \"Meetings/undated.md\"");
}
#[when("I preview folder \"Meetings\" for dates \"2026-11-01\" through \"2026-11-02\"")]
fn step_6(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: I preview folder \"Meetings\" for dates \"2026-11-01\" through \"2026-11-02\""
    );
}
#[then(
    "the preview includes \"Meetings/one.md\" and excludes \"Meetings/two.md\" and \"Meetings/undated.md\""
)]
fn step_7(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the preview includes \"Meetings/one.md\" and excludes \"Meetings/two.md\" and \"Meetings/undated.md\""
    );
}
#[then("no note bytes have been sent to a model")]
fn step_8(_: &mut Workspace) {
    panic!("Missing workspace behavior: no note bytes have been sent to a model");
}
#[given("\"Meetings/one.md\" is unknown and \"Meetings/private.md\" is local-only")]
fn step_9(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: \"Meetings/one.md\" is unknown and \"Meetings/private.md\" is local-only"
    );
}
#[when("I preview both files and confirm unknown sources for this run")]
fn step_10(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: I preview both files and confirm unknown sources for this run"
    );
}
#[then("the model can receive \"Meetings/one.md\"")]
fn step_11(_: &mut Workspace) {
    panic!("Missing workspace behavior: the model can receive \"Meetings/one.md\"");
}
#[then("all model tools deny \"Meetings/private.md\" and its derived topic context")]
fn step_12(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: all model tools deny \"Meetings/private.md\" and its derived topic context"
    );
}
#[given("I preview \"Meetings/one.md\"")]
fn step_13(_: &mut Workspace) {
    panic!("Missing workspace behavior: I preview \"Meetings/one.md\"");
}
#[when("another program changes \"Meetings/one.md\" before analysis starts")]
fn step_14(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: another program changes \"Meetings/one.md\" before analysis starts"
    );
}
#[then("analysis fails with \"stale_scope\"")]
fn step_15(_: &mut Workspace) {
    panic!("Missing workspace behavior: analysis fails with \"stale_scope\"");
}
#[then("no note bytes are sent to a model")]
fn step_16(_: &mut Workspace) {
    panic!("Missing workspace behavior: no note bytes are sent to a model");
}
#[given("a selected synthetic meeting says \"Alex leads Ledger\"")]
fn step_17(_: &mut Workspace) {
    panic!("Missing workspace behavior: a selected synthetic meeting says \"Alex leads Ledger\"");
}
#[given(
    "the scripted model proposes person \"Alex\", project \"Ledger\" and a cited \"leads\" connection"
)]
fn step_18(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the scripted model proposes person \"Alex\", project \"Ledger\" and a cited \"leads\" connection"
    );
}
#[when("analysis completes successfully")]
fn step_19(_: &mut Workspace) {
    panic!("Missing workspace behavior: analysis completes successfully");
}
#[then("the proposals show the passage and its source locator")]
fn step_20(_: &mut Workspace) {
    panic!("Missing workspace behavior: the proposals show the passage and its source locator");
}
#[then("no topic, claim or connection is accepted")]
fn step_21(_: &mut Workspace) {
    panic!("Missing workspace behavior: no topic, claim or connection is accepted");
}
#[then("the meeting bytes are unchanged")]
fn step_22(_: &mut Workspace) {
    panic!("Missing workspace behavior: the meeting bytes are unchanged");
}
#[given("two accepted people named \"Joe\" have different IDs")]
fn step_23(_: &mut Workspace) {
    panic!("Missing workspace behavior: two accepted people named \"Joe\" have different IDs");
}
#[given("a selected meeting mentions \"Joe\" without distinguishing them")]
fn step_24(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: a selected meeting mentions \"Joe\" without distinguishing them"
    );
}
#[when("the scripted model proposes an identity for that mention")]
fn step_25(_: &mut Workspace) {
    panic!("Missing workspace behavior: the scripted model proposes an identity for that mention");
}
#[then("the review shows both candidates as unresolved")]
fn step_26(_: &mut Workspace) {
    panic!("Missing workspace behavior: the review shows both candidates as unresolved");
}
#[then("no connection to either person is accepted")]
fn step_27(_: &mut Workspace) {
    panic!("Missing workspace behavior: no connection to either person is accepted");
}
#[given("a cited claim from \"Meetings/one.md\" was accepted")]
fn step_28(_: &mut Workspace) {
    panic!("Missing workspace behavior: a cited claim from \"Meetings/one.md\" was accepted");
}
#[when("the same unchanged meeting produces the same claim in a later analysis")]
fn step_29(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the same unchanged meeting produces the same claim in a later analysis"
    );
}
#[then("the existing decision is shown as a duplicate")]
fn step_30(_: &mut Workspace) {
    panic!("Missing workspace behavior: the existing decision is shown as a duplicate");
}
#[then("there is still one accepted claim")]
fn step_31(_: &mut Workspace) {
    panic!("Missing workspace behavior: there is still one accepted claim");
}
#[given("Codex is unavailable")]
fn step_32(_: &mut Workspace) {
    panic!("Missing workspace behavior: Codex is unavailable");
}
#[when("I open the workspace")]
fn step_33(_: &mut Workspace) {
    panic!("Missing workspace behavior: I open the workspace");
}
#[then("I see how to install or configure the required runtime")]
fn step_34(_: &mut Workspace) {
    panic!("Missing workspace behavior: I see how to install or configure the required runtime");
}
#[then("I can browse accepted topics and create a manual proposal")]
fn step_35(_: &mut Workspace) {
    panic!("Missing workspace behavior: I can browse accepted topics and create a manual proposal");
}
#[given("an accepted topic, claim, connection and correction rule")]
fn step_36(_: &mut Workspace) {
    panic!("Missing workspace behavior: an accepted topic, claim, connection and correction rule");
}
#[when("the derived database is deleted and the app restarts")]
fn step_37(_: &mut Workspace) {
    panic!("Missing workspace behavior: the derived database is deleted and the app restarts");
}
#[then("the same stable IDs, current claim, history and citation are recovered")]
fn step_38(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the same stable IDs, current claim, history and citation are recovered"
    );
}
#[given("a pending update with an expected record hash")]
fn step_39(_: &mut Workspace) {
    panic!("Missing workspace behavior: a pending update with an expected record hash");
}
#[when("another program changes that record before approval")]
fn step_40(_: &mut Workspace) {
    panic!("Missing workspace behavior: another program changes that record before approval");
}
#[then("approval reports a conflict and preserves the external bytes and proposed bytes")]
fn step_41(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: approval reports a conflict and preserves the external bytes and proposed bytes"
    );
}
#[then("no dependent record becomes visible")]
fn step_42(_: &mut Workspace) {
    panic!("Missing workspace behavior: no dependent record becomes visible");
}
#[given("approval of a topic and dependent claim is interrupted after the first file")]
fn step_43(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: approval of a topic and dependent claim is interrupted after the first file"
    );
}
#[when("the app restarts")]
fn step_44(_: &mut Workspace) {
    panic!("Missing workspace behavior: the app restarts");
}
#[then("recovery completes the operation if remaining expected hashes match")]
fn step_45(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: recovery completes the operation if remaining expected hashes match"
    );
}
#[then("the topic and claim become visible together")]
fn step_46(_: &mut Workspace) {
    panic!("Missing workspace behavior: the topic and claim become visible together");
}
#[given("approval is interrupted after its first file")]
fn step_47(_: &mut Workspace) {
    panic!("Missing workspace behavior: approval is interrupted after its first file");
}
#[given("another program changes a remaining target")]
fn step_48(_: &mut Workspace) {
    panic!("Missing workspace behavior: another program changes a remaining target");
}
#[then("recovery reports a conflict and preserves old, staged and current versions")]
fn step_49(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: recovery reports a conflict and preserves old, staged and current versions"
    );
}
#[then("incomplete records are not visible as accepted")]
fn step_50(_: &mut Workspace) {
    panic!("Missing workspace behavior: incomplete records are not visible as accepted");
}
#[given("a topic was created and a later claim depends on it")]
fn step_51(_: &mut Workspace) {
    panic!("Missing workspace behavior: a topic was created and a later claim depends on it");
}
#[when("I undo the topic creation")]
fn step_52(_: &mut Workspace) {
    panic!("Missing workspace behavior: I undo the topic creation");
}
#[then("undo is refused with the dependent claim identified")]
fn step_53(_: &mut Workspace) {
    panic!("Missing workspace behavior: undo is refused with the dependent claim identified");
}
#[when("I undo the claim and then undo the topic creation")]
fn step_54(_: &mut Workspace) {
    panic!("Missing workspace behavior: I undo the claim and then undo the topic creation");
}
#[then("both are absent from current knowledge and remain visible in history")]
fn step_55(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: both are absent from current knowledge and remain visible in history"
    );
}
#[given("root \"A\" has an accepted topic and pending proposal")]
fn step_56(_: &mut Workspace) {
    panic!("Missing workspace behavior: root \"A\" has an accepted topic and pending proposal");
}
#[when("I switch to root \"B\"")]
fn step_57(_: &mut Workspace) {
    panic!("Missing workspace behavior: I switch to root \"B\"");
}
#[then("neither root A's topic nor its proposal appears")]
fn step_58(_: &mut Workspace) {
    panic!("Missing workspace behavior: neither root A's topic nor its proposal appears");
}
#[then("an old root A approval request is rejected")]
fn step_59(_: &mut Workspace) {
    panic!("Missing workspace behavior: an old root A approval request is rejected");
}
#[given("a pending claim says \"Ledger launch is Friday\" with a meeting citation")]
fn step_60(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: a pending claim says \"Ledger launch is Friday\" with a meeting citation"
    );
}
#[when("I edit the claim to \"Ledger launch is Monday\" and approve it")]
fn step_61(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: I edit the claim to \"Ledger launch is Monday\" and approve it"
    );
}
#[then("the topic shows the accepted Monday claim with its citation and uncertainty")]
fn step_62(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the topic shows the accepted Monday claim with its citation and uncertainty"
    );
}
#[then("the source meeting bytes are unchanged")]
fn step_63(_: &mut Workspace) {
    panic!("Missing workspace behavior: the source meeting bytes are unchanged");
}
#[given("a pending claim says \"Ledger is cancelled\"")]
fn step_64(_: &mut Workspace) {
    panic!("Missing workspace behavior: a pending claim says \"Ledger is cancelled\"");
}
#[when("I reject the proposal")]
fn step_65(_: &mut Workspace) {
    panic!("Missing workspace behavior: I reject the proposal");
}
#[then("no accepted claim says \"Ledger is cancelled\"")]
fn step_66(_: &mut Workspace) {
    panic!("Missing workspace behavior: no accepted claim says \"Ledger is cancelled\"");
}
#[then("the proposal is marked rejected")]
fn step_67(_: &mut Workspace) {
    panic!("Missing workspace behavior: the proposal is marked rejected");
}
#[given("a proposal contains a new person and a connection to that person")]
fn step_68(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: a proposal contains a new person and a connection to that person"
    );
}
#[when("I approve only the connection")]
fn step_69(_: &mut Workspace) {
    panic!("Missing workspace behavior: I approve only the connection");
}
#[then("approval fails with a dependency error")]
fn step_70(_: &mut Workspace) {
    panic!("Missing workspace behavior: approval fails with a dependency error");
}
#[then("neither record is accepted")]
fn step_71(_: &mut Workspace) {
    panic!("Missing workspace behavior: neither record is accepted");
}
#[given("an accepted claim says person \"Alex\" uses \"he/him\" with a transcript citation")]
fn step_72(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: an accepted claim says person \"Alex\" uses \"he/him\" with a transcript citation"
    );
}
#[given("topic chat proposes \"they/them\" and a rule scoped to Alex's pronouns")]
fn step_73(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: topic chat proposes \"they/them\" and a rule scoped to Alex's pronouns"
    );
}
#[when("I approve the correction")]
fn step_74(_: &mut Workspace) {
    panic!("Missing workspace behavior: I approve the correction");
}
#[then("Alex's current claim says \"they/them\" and cites my persisted chat statement")]
fn step_75(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: Alex's current claim says \"they/them\" and cites my persisted chat statement"
    );
}
#[then("the prior claim is superseded but visible in history")]
fn step_76(_: &mut Workspace) {
    panic!("Missing workspace behavior: the prior claim is superseded but visible in history");
}
#[when("the same transcript is analyzed again and the app restarts")]
fn step_77(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the same transcript is analyzed again and the app restarts"
    );
}
#[then("\"he/him\" is not restored as current knowledge")]
fn step_78(_: &mut Workspace) {
    panic!("Missing workspace behavior: \"he/him\" is not restored as current knowledge");
}
#[then("the rule does not change another person's pronouns")]
fn step_79(_: &mut Workspace) {
    panic!("Missing workspace behavior: the rule does not change another person's pronouns");
}
#[then("the transcript bytes are unchanged")]
fn step_80(_: &mut Workspace) {
    panic!("Missing workspace behavior: the transcript bytes are unchanged");
}
#[given("a pending claim cites a passage in \"Meetings/one.md\"")]
fn step_81(_: &mut Workspace) {
    panic!("Missing workspace behavior: a pending claim cites a passage in \"Meetings/one.md\"");
}
#[when("another program changes that passage before I approve it")]
fn step_82(_: &mut Workspace) {
    panic!("Missing workspace behavior: another program changes that passage before I approve it");
}
#[then("approval fails with \"stale_evidence\"")]
fn step_83(_: &mut Workspace) {
    panic!("Missing workspace behavior: approval fails with \"stale_evidence\"");
}
#[then("the proposed claim is not accepted")]
fn step_84(_: &mut Workspace) {
    panic!("Missing workspace behavior: the proposed claim is not accepted");
}
#[given("two corrections supersede the same accepted claim revision")]
fn step_85(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: two corrections supersede the same accepted claim revision"
    );
}
#[when("the first correction is approved and the second is approved from its stale view")]
fn step_86(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the first correction is approved and the second is approved from its stale view"
    );
}
#[then("the second approval reports a conflict")]
fn step_87(_: &mut Workspace) {
    panic!("Missing workspace behavior: the second approval reports a conflict");
}
#[then("neither correction silently overwrites the other")]
fn step_88(_: &mut Workspace) {
    panic!("Missing workspace behavior: neither correction silently overwrites the other");
}
#[given("the run is authorized only for \"Meetings/one.md\"")]
fn step_89(_: &mut Workspace) {
    panic!("Missing workspace behavior: the run is authorized only for \"Meetings/one.md\"");
}
#[given("\"Meetings/private.md\" is local-only")]
fn step_90(_: &mut Workspace) {
    panic!("Missing workspace behavior: \"Meetings/private.md\" is local-only");
}
#[when("the model requests \"Meetings/private.md\" or an unselected source by ID")]
fn step_91(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the model requests \"Meetings/private.md\" or an unselected source by ID"
    );
}
#[then("the tool returns no source content or derived private context")]
fn step_92(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the tool returns no source content or derived private context"
    );
}
#[given("runtime preflight detects an unexpected tool or failed confinement check")]
fn step_93(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: runtime preflight detects an unexpected tool or failed confinement check"
    );
}
#[when("I start analysis")]
fn step_94(_: &mut Workspace) {
    panic!("Missing workspace behavior: I start analysis");
}
#[then("the runtime reports an actionable isolation error")]
fn step_95(_: &mut Workspace) {
    panic!("Missing workspace behavior: the runtime reports an actionable isolation error");
}
#[then("no selected note bytes are sent")]
fn step_96(_: &mut Workspace) {
    panic!("Missing workspace behavior: no selected note bytes are sent");
}
#[given("a model run has emitted a partial proposed change")]
fn step_97(_: &mut Workspace) {
    panic!("Missing workspace behavior: a model run has emitted a partial proposed change");
}
#[when("I cancel the run and a late proposal event arrives")]
fn step_98(_: &mut Workspace) {
    panic!("Missing workspace behavior: I cancel the run and a late proposal event arrives");
}
#[then("the run ends cancelled and no proposal or accepted record is created")]
fn step_99(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: the run ends cancelled and no proposal or accepted record is created"
    );
}
#[given("a model run returns a proposal with an invalid source locator")]
fn step_100(_: &mut Workspace) {
    panic!(
        "Missing workspace behavior: a model run returns a proposal with an invalid source locator"
    );
}
#[when("the run completes")]
fn step_101(_: &mut Workspace) {
    panic!("Missing workspace behavior: the run completes");
}
#[then("the proposal is rejected with an actionable error")]
fn step_102(_: &mut Workspace) {
    panic!("Missing workspace behavior: the proposal is rejected with an actionable error");
}
#[then("no accepted record or source file changes")]
fn step_103(_: &mut Workspace) {
    panic!("Missing workspace behavior: no accepted record or source file changes");
}
#[tokio::main]
async fn main() {
    Workspace::cucumber()
        .fail_on_skipped()
        .run_and_exit("features/topic-workspace")
        .await;
}
