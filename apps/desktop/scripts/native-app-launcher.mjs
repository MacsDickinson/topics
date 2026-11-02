#!/usr/bin/env node
// Native-test diagnostic parent only; no environment/credentials are logged.
import { spawn } from 'node:child_process';
import { appendFileSync } from 'node:fs';
import { isAbsolute } from 'node:path';

const binary = process.env.TOPICS_TEST_REAL_APP_BINARY;
const log = process.env.TOPICS_TEST_PROCESS_LOG;
if (!binary || !isAbsolute(binary) || !log || !isAbsolute(log)) throw new Error('Native launcher requires explicit app and diagnostic paths');
const record = event => appendFileSync(log, `${JSON.stringify({ time: new Date().toISOString(), ...event })}\n`);
const child = spawn(binary, process.argv.slice(2), { env: process.env, stdio: 'inherit' });
record({ event: 'spawn', pid: child.pid });
for (const signal of ['SIGTERM', 'SIGINT']) process.on(signal, () => { record({ event: 'forward', signal }); child.kill(signal); });
child.on('error', error => { record({ event: 'spawn-error', code: error.code }); process.exitCode = 1; });
child.on('exit', (code, signal) => { record({ event: 'exit', code, signal }); process.exitCode = code ?? (signal === 'SIGTERM' ? 0 : 1); });
