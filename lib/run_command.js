'use strict';

const Command = require('./command');

class RunCommand extends Command {

  * _run(cwd, [ hookName ]) {
    yield this.init();
    yield this.runHook(hookName);
  }
}

module.exports = RunCommand;
