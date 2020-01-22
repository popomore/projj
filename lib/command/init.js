'use strict';

const BaseCommand = require('../base_command');

class InitCommand extends BaseCommand {
  async _run() {
    console.log(this.config);
    this.logger.info('Set base directory: %s', this.config.base.join(','));
  }

  get description() {
    return 'Initialize configuration';
  }

}

module.exports = InitCommand;
