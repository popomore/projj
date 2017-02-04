'use strict';

const giturl = require('giturl');
const path = require('path');
const fs = require('mz/fs');
const chalk = require('chalk');
const clipboardy = require('clipboardy');
const Command = require('./command');

class AddCommand extends Command {

  * _run(cwd, [ repo ]) {
    const key = url2dir(repo);
    const targetPath = path.join(this.config.base, key);
    this.logger.info('Start adding repository %s', chalk.green(repo));

    if (yield fs.exists(targetPath)) {
      throw new Error(`${targetPath} already exist`);
    }

    yield this.runHook('preadd', targetPath);
    this.logger.info('Cloning into %s', chalk.green(targetPath));
    yield this.runScript(`git clone ${repo} ${targetPath} > /dev/null 2>&1`);
    yield this.runHook('postadd', targetPath);

    // add this repository to cache.json
    yield this.setCache(key);

    try {
      yield clipboardy.write(`cd ${targetPath}`);
      this.logger.info(chalk.green('📋  Copied to clipboard') + ', just use Ctrl+V');
    } catch (e) {
      this.logger.warn('Fail to copy to clipboard, error: %s', e.message);
    }
  }

  help() {
    return 'add repository';
  }

}

module.exports = AddCommand;

// https://github.com/popomore/projj.git
// => $BASE/github.com/popomore/projj
function url2dir(url) {
  url = giturl.parse(url);
  return url.replace(/https?:\/\//, '');
}
