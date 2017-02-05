'use strict';

const fs = require('fs');
const path = require('path');
const BaseProgram = require('common-bin').Program;

class Program extends BaseProgram {
  constructor() {
    super();
    this.version = require('../package.json').version;
    this.loadCommand();
  }

  loadCommand() {
    const commandPath = path.join(__dirname, 'command');
    const commands = fs.readdirSync(commandPath);
    for (const command of commands) {
      this.addCommand(command.replace(/\.js$/, ''), path.join(commandPath, command));
    }
  }
}

module.exports = Program;
