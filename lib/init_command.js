'use strict';

const Command = require('./command');

class InitCommand extends Command {
  * _run() {
    yield this.init();
    this.logger.info('base directory: %s', this.config.base);
  }

  help() {
    return 'initialize configuration';
  }

}

module.exports = InitCommand;
