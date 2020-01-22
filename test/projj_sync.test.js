'use strict';

const path = require('path');
const coffee = require('coffee');
const mm = require('mm');
const rimraf = require('mz-modules/rimraf');
const fs = require('mz/fs');

const binfile = path.join(__dirname, '../bin/projj.js');
const fixtures = path.join(__dirname, 'fixtures');
const tmp = path.join(fixtures, 'tmp');


describe('test/projj_sync.test.js', () => {

  afterEach(mm.restore);
  afterEach(() => rimraf(tmp));

  it('should run hook that do not exist', function* () {
    const home = path.join(fixtures, 'hook');
    mm(process.env, 'HOME', home);

    const content = JSON.stringify({
      [path.join(tmp, 'github.com/popomore/projj')]: {},
    });
    yield fs.writeFile(path.join(home, '.projj/cache.json'), content);

    yield coffee.fork(binfile, [ 'sync' ])
    // .debug()
      .expect('stdout', new RegExp(`Remove ${tmp}/github.com/popomore/projj that don't exist`))
      .expect('code', 0)
      .end();
  });

});
