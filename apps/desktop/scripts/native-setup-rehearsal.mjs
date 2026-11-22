// Scripted-only native rehearsal: no installed provider, credentials or personal notes.
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, dirname, isAbsolute } from 'node:path';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { prepareNativeEnvironment, assertNoTopicsFallback } from './native-environment.mjs';
if (process.platform !== 'darwin' || process.env.TOPICS_SCRIPTED_REHEARSAL !== '1') throw new Error('Opt in with TOPICS_SCRIPTED_REHEARSAL=1 on macOS; this is a scripted rehearsal.');
const desktop=resolve(import.meta.dirname,'..'), target=process.env.CARGO_TARGET_DIR;
if (!target || !isAbsolute(target)) throw new Error('Use an absolute exclusive CARGO_TARGET_DIR');
const fixture=mkdtempSync(join(tmpdir(),'topics-scripted-rehearsal-')), notes=join(fixture,'notes');
mkdirSync(join(notes,'profiles'),{recursive:true}); mkdirSync(join(notes,'private'));
const files=[[join(notes,'profiles','alex.md'),'# Synthetic Alex A42\n\nAlex A42 is a person. Alex A42 uses he/him pronouns. Alex A42 leads the Ledger project.\n'],[join(notes,'private','private.md'),'# Local only\n\nPRIVATE_SYNTHETIC_ONLY must stay local.\n']];
for(const [path,content] of files) writeFileSync(path,content);
const hashes=files.map(([path,content])=>[path,createHash('sha256').update(content).digest('hex')]);
const session=prepareNativeEnvironment(fixture), codexHome=join(session.HOME,'.codex'); mkdirSync(codexHome);
const locator=spawnSync('rustup',['which','cargo'],{encoding:'utf8'}); if(locator.status!==0) throw Error('cargo unavailable');
const registry=join(process.env.CARGO_HOME || join(process.env.HOME,'.cargo'),'registry'),cargoHome=join(fixture,'cargo-home'); mkdirSync(cargoHome); symlinkSync(registry,join(cargoHome,'registry'),'dir');
const output=join(fixture,'provider'); mkdirSync(output);
const evidence=join(desktop,'evidence/scripted-setup-rehearsal'); mkdirSync(evidence,{recursive:true});
const env={...process.env,HOME:session.HOME,CODEX_HOME:codexHome,XDG_DATA_HOME:session.XDG_DATA_HOME,CARGO_HOME:cargoHome,CARGO_NET_OFFLINE:'true',CARGO_INCREMENTAL:'0',CARGO_PROFILE_DEV_DEBUG:'0',CARGO_PROFILE_TEST_DEBUG:'0',CARGO_TARGET_DIR:target,TOPICS_APP_DATA_DIR:join(fixture,'app-data'),TOPICS_TEST_HOME:session.HOME,TOPICS_TEST_CODEX_HOME:codexHome,TOPICS_TEST_XDG_DATA:session.XDG_DATA_HOME,TOPICS_TEST_SANDBOX:fixture,TOPICS_TEST_WORKSPACE_ROOT:notes,TOPICS_TEST_LIVE_SETUP:'1',TOPICS_TEST_REHEARSAL:'1',TOPICS_TEST_EXPECTED:join(fixture,'expected.json'),TOPICS_TEST_OBSERVATION:join(evidence,'observation.json'),TOPICS_TEST_ARTIFACT_PREFIX:join(evidence,'scripted'),VITE_NATIVE_TEST:'1',TOPICS_TEST_REAL_APP_BINARY:join(target,'debug/topics-desktop'),TOPICS_TEST_APP_LAUNCHER:join(desktop,'scripts/native-app-launcher.mjs'),PATH:`${join(desktop,'node_modules/.bin')}:${dirname(locator.stdout.trim())}:${process.env.PATH}`};
function run(command,args,capture=false){const result=spawnSync(command,args,{cwd:desktop,env,encoding:capture?'utf8':undefined,stdio:capture?['ignore','pipe','inherit']:'inherit',timeout:600000});if(result.error)throw result.error;if(result.status!==0)throw Error(`${command} exited ${result.status}`);return result.stdout;}
try {
  console.log('SCRIPTED REHEARSAL: local fixture provider, not genuine Codex');
  const rendered=JSON.parse(run('cargo',['run','-q','-p','topics-setup','--example','render_fixture','--',output,'rehearsal_claim'],true));
  env.TOPICS_TEST_CODEX=rendered.executable; env.TOPICS_CODEX_PATH=rendered.executable;
  run('npm',['run','tauri','--','build','--debug','--no-bundle','--features','native-test','--config','src-tauri/tauri.native-test.conf.json']);
  run('npm',['exec','--','wdio','run','wdio.conf.ts']);
  for(const file of ['index.sqlite3','index.sqlite3-wal','index.sqlite3-shm','index.sqlite3-journal']) rmSync(join(env.TOPICS_APP_DATA_DIR,file),{force:true});
  env.TOPICS_TEST_SETUP_RESTART='1'; run('npm',['exec','--','wdio','run','wdio.conf.ts']);
  assertNoTopicsFallback(fixture);
} finally {
  for(const [path,hash] of hashes) if(createHash('sha256').update(readFileSync(path)).digest('hex')!==hash) throw Error('Synthetic source hash changed');
  rmSync(fixture,{recursive:true,force:true});
}
