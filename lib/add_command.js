'use strict';

const giturl = require('giturl');
const path = require('path');
const fs = require('mz/fs');
const Command = require('./command');

class AddCommand extends Command {

  * _run(cwd, [ repo ]) {
    yield this.init();

    const key = url2dir(repo);
    const targetPath = path.join(this.config.base, key);
    this.logger.info('add repo %s to %s', repo, targetPath);

    if (yield fs.exists(targetPath)) {
      this.logger.error(`${targetPath} already exist`);
      process.exit(1);
    }

    yield this.runHook('preadd', targetPath);
    yield this.spawn('git', [ 'clone', repo, targetPath ]);
    yield this.runHook('postadd', targetPath);
    yield this.setCache(key);
    this.logger.info('done');
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
