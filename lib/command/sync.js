'use strict';

const BaseCommand = require('../base_command');

class SyncCommand extends BaseCommand {

  * _run() {
    // a
  }

  help() {
    return 'Run hook in every repository';
  }
}

module.exports = SyncCommand;
