'use strict';

const Command = require('./command');

class RunCommand extends Command {

  * _run(cwd, [ hookName ]) {
    if (!hookName || !this.config.hooks[hookName]) {
      throw new Error(`hook "${hookName}" don't exist`);
    }

    for (const key of Object.keys(this.cache)) {
      yield this.runHook(hookName, key);
    }
  }

  help() {
    return 'run hook in every repository';
  }
}

module.exports = RunCommand;
