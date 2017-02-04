'use strict';

const path = require('path');
const Command = require('./command');

class RunCommand extends Command {

  * _run(cwd, [ hookName ]) {
    if (!hookName || !this.config.hooks[hookName]) {
      throw new Error(`hook "${hookName}" don't exist`);
    }

    for (const key of Object.keys(this.cache)) {
      yield this.runHook(hookName, path.join(this.config.base, key));
    }
  }

  help() {
    return 'run a hook';
  }
}

module.exports = RunCommand;
