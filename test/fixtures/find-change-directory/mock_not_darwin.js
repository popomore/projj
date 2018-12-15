'use strict';

const mm = require('mm');
const clipboardy = require('clipboardy');


mm(process, 'platform', 'not_darwin');
mm(clipboardy, 'write', function* () {
  return;
});
