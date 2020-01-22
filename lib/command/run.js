'use strict';

const BaseCommand = require('../base_command');

class RunCommand extends BaseCommand {

  async _run(cwd, [ hookName ]) {
    if (!hookName || !this.config.hooks[hookName]) {
      throw new Error(`Hook "${hookName}" don't exist`);
    }

    await this.runHook(hookName, cwd);
  }

  get description() {
    return 'Run hook in current directory';
  }
}

module.exports = RunCommand;
