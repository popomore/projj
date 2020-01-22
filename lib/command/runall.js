'use strict';

const BaseCommand = require('../base_command');

class RunCommand extends BaseCommand {

  * _run(cwd, [ hookName ]) {
    if (!hookName || !this.config.hooks[hookName]) {
      throw new Error(`hook "${hookName}" don't exist`);
    }

    const keys = yield this.cache.getKeys();
    for (const key of keys) {
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
