'use strict';

const BaseCommand = require('../base_command');

class RunCommand extends BaseCommand {

  * _run(cwd, [ hookName ]) {
    if (!hookName || !this.config.hooks[hookName]) {
      throw new Error(`hook "${hookName}" don't exist`);
    }

    for (const key of Object.keys(this.cache)) {
      try {
        yield this.runHook(hookName, key);
      } catch (err) {
        this.childLogger.error(err.message);
      }
    }
  }

  help() {
    return 'Run hook in every repository';
  }
}

module.exports = RunCommand;
