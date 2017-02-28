'use strict';

const path = require('path');
const coffee = require('coffee');
const mm = require('mm');
const rimraf = require('mz-modules/rimraf');
const fs = require('mz/fs');


const binfile = path.join(__dirname, '../bin/projj.js');
const fixtures = path.join(__dirname, 'fixtures');
const tmp = path.join(fixtures, 'tmp');


describe('test/projj_runall.test.js', () => {
  const home = path.join(fixtures, 'hook');
  const content = JSON.stringify({
    'github.com/popomore/test1': {},
    'github.com/popomore/test2': {},
  });

  before(() => fs.writeFile(path.join(home, '.projj/cache.json'), content));
  afterEach(mm.restore);
  afterEach(() => rimraf(tmp));

  it('should run hook that do not exist', done => {
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'runall', 'noexist' ])
    // .debug()
    .expect('stderr', /hook "noexist" don't exist/)
    .expect('code', 1)
    .end(done);
  });

  it('should run hook in every repo', done => {
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'runall', 'custom' ])
    // .debug()
    .expect('stdout', /Run hook custom for github.com\/popomore\/test1/)
    .expect('stdout', /Run hook custom for github.com\/popomore\/test2/)
    .expect('stdout', new RegExp(`get package name test1 from ${home}/github.com/popomore/test1`))
    .expect('stdout', new RegExp(`get package name test2 from ${home}/github.com/popomore/test2`))
    .expect('code', 0)
    .end(done);
  });

  it('should run all hooks if one has error', done => {
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'runall', 'error' ])
    // .debug()
    .expect('stdout', /Run hook error for github.com\/popomore\/test1/)
    .expect('stdout', /Run hook error for github.com\/popomore\/test2/)
    .expect('stderr', /Run "sh -c exit 1" error, exit code 1/)
    .expect('code', 0)
    .end(done);
  });
});
