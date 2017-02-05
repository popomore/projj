'use strict';

const BaseCommand = require('../base_command');

class RunCommand extends BaseCommand {

  * _run(cwd, [ hookName ]) {
    if (!hookName || !this.config.hooks[hookName]) {
      throw new Error(`Hook "${hookName}" don't exist`);
    }

    yield this.runHook(hookName, cwd);
  }

  help() {
    return 'Run hook in current directory';
  }
}

module.exports = RunCommand;
