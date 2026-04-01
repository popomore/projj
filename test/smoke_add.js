'use strict';

const assert = require('assert');
const fs = require('fs');
const os = require('os');
const path = require('path');
const cp = require('child_process');

const repo = process.argv[2] || 'https://github.com/yiliang114/claude-code.git';
const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'projj-smoke-'));
const home = path.join(tmpRoot, 'home');
const base = path.join(tmpRoot, 'workspace');
const configDir = path.join(home, '.projj');
const useLinkedBinary = process.env.PROJJ_SMOKE_USE_LINKED === '1';
const command = process.env.PROJJ_BIN || (useLinkedBinary ? 'projj' : 'node');
const args = useLinkedBinary ? [ 'add', repo ] : [ path.join(__dirname, '../bin/projj.js'), 'add', repo ];
const env = Object.assign({}, process.env, { HOME: home });

fs.mkdirSync(configDir, { recursive: true });
fs.mkdirSync(base, { recursive: true });
fs.writeFileSync(path.join(configDir, 'config.json'), JSON.stringify({
  base,
  hooks: {},
  alias: {
    'github://': 'https://github.com/',
  },
}, null, 2));

console.log(`[smoke] repo: ${repo}`);
console.log(`[smoke] tmp root: ${tmpRoot}`);
console.log(`[smoke] command: ${command} ${args.join(' ')}`);
console.log(`[smoke] HTTP_PROXY=${mask(process.env.HTTP_PROXY || process.env.http_proxy || process.env.npm_config_proxy)}`);
console.log(`[smoke] HTTPS_PROXY=${mask(process.env.HTTPS_PROXY || process.env.https_proxy || process.env.npm_config_https_proxy || process.env.npm_config_proxy)}`);

const child = cp.spawn(command, args, {
  env,
  stdio: [ 'ignore', 'pipe', 'pipe' ],
});

let combined = '';
child.stdout.on('data', chunk => {
  const text = chunk.toString();
  combined += text;
  process.stdout.write(text);
});
child.stderr.on('data', chunk => {
  const text = chunk.toString();
  combined += text;
  process.stderr.write(text);
});

child.on('exit', code => {
  try {
    assert.strictEqual(code, 0, `smoke add exited with code ${code}`);

    const progressPattern = /Receiving objects:|Resolving deltas:|remote: Enumerating objects:|remote: Counting objects:|Updating files:/;
    assert(progressPattern.test(combined), 'expected git clone progress output in terminal');

    const hasProxyEnv = Boolean(
      process.env.HTTP_PROXY ||
      process.env.http_proxy ||
      process.env.HTTPS_PROXY ||
      process.env.https_proxy ||
      process.env.ALL_PROXY ||
      process.env.all_proxy ||
      process.env.npm_config_proxy ||
      process.env.npm_config_https_proxy
    );

    if (hasProxyEnv) {
      assert(combined.includes('Detected git proxy settings:'), 'expected proxy diagnostics in output');
    } else {
      console.log('[smoke] proxy env not set, skipping proxy diagnostics assertion');
    }

    console.log('[smoke] assertions passed');
    console.log(`[smoke] cloned into: ${base}`);
  } catch (err) {
    console.error(`[smoke] failed: ${err.message}`);
    process.exitCode = 1;
  }
});

function mask(value) {
  if (!value) return '<unset>';
  return value.replace(/\/\/[^@]+@/, '//***:***@');
}
