'use strict';

const path = require('path');
const Command = require('common-bin');

class Program extends Command {
  constructor(rawArgv) {
    super(rawArgv);
    this.yargs.scriptName('projj');
    this.usage = 'Usage: [command] [options]';
    this.version = require('../package.json').version;
    this.load(path.join(__dirname, 'command'));
  }
}

module.exports = Program;
