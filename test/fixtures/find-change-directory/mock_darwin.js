'use strict';

const mm = require('mm');
const FindCommand = require('../../../lib/command/find');

mm(process, 'platform', 'darwin');

mm(FindCommand.prototype, 'runScript', function* (cmd) {
  console.log(cmd);
});
