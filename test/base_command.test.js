'use strict';

const assert = require('assert');
const path = require('path');

const BaseCommand = require('../lib/base_command');

class TestCommand extends BaseCommand {
  constructor() {
    super([]);
    this.cache = {
      async set() {
        return undefined;
      },
      async dump() {
        return undefined;
      },
    };
  }
}

describe('test/base_command.test.js', () => {

  it('should build git clone command with progress enabled', () => {
    const command = new TestCommand();
    const repo = 'https://github.com/example/repo.git';
    const targetPath = '/tmp/example/repo';

    assert.strictEqual(command.buildCloneCommand(repo, targetPath), `git clone --progress ${repo} ${targetPath}`);
  });

  it('should normalize proxy env for git from npm config and uppercase env', () => {
    const command = new TestCommand();
    const env = command.buildGitEnv({
      HOME: process.env.HOME,
      HTTP_PROXY: 'http://127.0.0.1:7890',
      npm_config_https_proxy: 'http://127.0.0.1:7891',
      NO_PROXY: 'localhost,127.0.0.1',
    });

    assert.strictEqual(env.GIT_SSH, path.join(__dirname, '../lib/ssh.js'));
    assert.strictEqual(env.http_proxy, 'http://127.0.0.1:7890');
    assert.strictEqual(env.HTTP_PROXY, 'http://127.0.0.1:7890');
    assert.strictEqual(env.https_proxy, 'http://127.0.0.1:7891');
    assert.strictEqual(env.HTTPS_PROXY, 'http://127.0.0.1:7891');
    assert.strictEqual(env.no_proxy, 'localhost,127.0.0.1');
    assert.strictEqual(env.NO_PROXY, 'localhost,127.0.0.1');
  });

  it('should prepend hook directory to PATH with the provided delimiter', () => {
    const command = new TestCommand();
    const env = command.buildHookEnv('custom', {
      PATH: 'C:\\Windows\\System32',
    }, ';');

    assert.strictEqual(env.PATH, `${path.join(process.env.HOME, '.projj/hooks')};C:\\Windows\\System32`);
    assert.strictEqual(env.PROJJ_HOOK_NAME, 'custom');
  });

  it('should match repo keys relative to base directories across separators', () => {
    const command = new TestCommand();
    const base = String.raw`D:\a\projj\projj\test\fixtures\remove\temp`;
    const keys = [
      String.raw`D:\a\projj\projj\test\fixtures\remove\temp\github.com\popomore\projj`,
      String.raw`D:\a\projj\projj\test\fixtures\remove\temp\github.com\eggjs\egg`,
    ];
    command.config = {
      base: [ base ],
    };

    assert.deepStrictEqual(command.matchRepoKeys(keys, 'projj'), [ keys[0] ]);
    assert.deepStrictEqual(command.matchRepoKeys(keys, '/projj'), [ keys[0] ]);
  });

  it('should derive repo label from windows cache key', () => {
    const command = new TestCommand();
    command.config = {
      base: [ String.raw`D:\a\projj\projj\test\fixtures\remove\temp` ],
    };

    assert.strictEqual(
      command.getRepoLabel(String.raw`D:\a\projj\projj\test\fixtures\remove\temp\github.com\eggjs\autod-egg`),
      'eggjs/autod-egg'
    );
  });

  it('should forward buffered child process output to child logger streams', () => {
    const command = new TestCommand();
    const writes = {
      stdout: [],
      stderr: [],
    };

    command.childLogger = {
      log(message) {
        writes.stdout.push(message);
      },
      error(message) {
        writes.stderr.push(message);
      },
    };

    command.forwardScriptOutput({
      stdout: Buffer.from('hook stdout\n'),
      stderr: Buffer.from('hook stderr\n'),
    });

    assert.deepStrictEqual(writes.stdout, [ 'hook stdout' ]);
    assert.deepStrictEqual(writes.stderr, [ 'hook stderr' ]);
  });

});
