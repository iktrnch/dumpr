#!/usr/bin/env node
'use strict';
const { spawnSync } = require('node:child_process');
const { join } = require('node:path');
const packages = {
  'linux-x64': '@iktrnch/dumpr-linux-x64', 'linux-arm64': '@iktrnch/dumpr-linux-arm64',
  'darwin-x64': '@iktrnch/dumpr-darwin-x64', 'darwin-arm64': '@iktrnch/dumpr-darwin-arm64',
  'win32-x64': '@iktrnch/dumpr-win32-x64',
};
const packageName = packages[`${process.platform}-${process.arch}`];
if (!packageName) { console.error(`dumpr does not provide a native binary for ${process.platform}-${process.arch}`); process.exit(1); }
let executable;
try { executable = join(require.resolve(`${packageName}/package.json`), '..', 'bin', process.platform === 'win32' ? 'dumpr.exe' : 'dumpr'); }
catch (_) { console.error(`dumpr's optional native package (${packageName}) was not installed. Reinstall without --omit=optional.`); process.exit(1); }
const result = spawnSync(executable, process.argv.slice(2), { stdio: 'inherit' });
if (result.error) throw result.error;
process.exit(result.status ?? 1);
