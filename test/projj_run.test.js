'use strict';

const path = require('path');
const coffee = require('coffee');
const mm = require('mm');
const rimraf = require('rimraf');

const binfile = path.join(__dirname, '../bin/projj.js');
const fixtures = path.join(__dirname, 'fixtures');
const tmp = path.join(fixtures, 'tmp');


describe('test/projj_run.test.js', () => {

  afterEach(mm.restore);
  afterEach(done => rimraf(tmp, done));

  it('should run hook that do not exist', done => {
    const home = path.join(fixtures, 'hook');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'run', 'noexist' ])
    // .debug()
    .expect('stderr', /hook "noexist" don't exist/)
    .expect('code', 1)
    .end(done);
  });

  it('should run hook in cwd', done => {
    const home = path.join(fixtures, 'hook');
    const cwd = path.join(home, 'github.com/popomore/test1');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'run', 'custom' ], { cwd })
    // .debug()
    .expect('stdout', new RegExp(`get package name test1 from ${home}/github.com/popomore/test1`))
    .expect('code', 0)
    .end(done);
  });

  it('should using buildin hook when has same name', done => {
    const home = path.join(fixtures, 'hook');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'run', 'ls' ])
    // .debug()
    .expect('stdout', /buildin ls/)
    .expect('code', 0)
    .end(done);
  });

  it('should get hook config', done => {
    const home = path.join(fixtures, 'hook');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'run', 'run_config' ])
    // .debug()
    .expect('stdout', /get config from env true/)
    .expect('code', 0)
    .end(done);
  });

});
