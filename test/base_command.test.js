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

});
