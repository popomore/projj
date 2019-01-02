'use strict';

const mm = require('mm');
const BaseCommand = require('../../lib/base_command');

mm(process, 'platform', 'darwin');

mm(BaseCommand.prototype, 'runScript', function* (cmd) {
  console.log(cmd);
});
