'use strict';

const BaseCommand = require('../base_command');

class InitCommand extends BaseCommand {
  * _run() {
    this.logger.info('Set base directory: %s', this.config.base);
  }

  help() {
    return 'Initialize configuration';
  }

}

module.exports = InitCommand;
