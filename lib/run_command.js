'use strict';

const Command = require('./command');

class RunCommand extends Command {

  * _run(cwd, [ hookName ]) {
    if (!hookName || !this.config.hooks[hookName]) {
      throw new Error(`Hook "${hookName}" don't exist`);
    }

    yield this.runHook(hookName, cwd);
  }

  help() {
    return 'run hook in current directory';
  }
}

module.exports = RunCommand;
