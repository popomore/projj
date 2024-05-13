import * as path from 'path';
import Command from 'common-bin';

class Program extends Command {
  constructor(rawArgv: string[]) {
    super(rawArgv);
    this.yargs.scriptName('projj');
    this.usage = 'Usage: [command] [options]';
    this.version = require('../package.json').version;
    this.load(path.join(__dirname, 'command'));
  }
}

export = Program;
