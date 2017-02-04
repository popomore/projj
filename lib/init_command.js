'use strict';

const Command = require('./command');

class InitCommand extends Command {
  * _run() {
    this.logger.info('Set base directory: %s', this.config.base);
  }

  help() {
    return 'initialize configuration';
  }

}

module.exports = InitCommand;
