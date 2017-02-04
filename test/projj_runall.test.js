'use strict';

const path = require('path');
const coffee = require('coffee');
const mm = require('mm');
const rimraf = require('rimraf');

const binfile = path.join(__dirname, '../bin/projj.js');
const fixtures = path.join(__dirname, 'fixtures');
const tmp = path.join(fixtures, 'tmp');


describe('test/projj_runall.test.js', () => {

  afterEach(mm.restore);
  afterEach(done => rimraf(tmp, done));

  it('should run hook that do not exist', done => {
    const home = path.join(fixtures, 'hook');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'runall', 'noexist' ])
    // .debug()
    .expect('stderr', /hook "noexist" don't exist/)
    .expect('code', 1)
    .end(done);
  });

  it('should run hook in every repo', done => {
    const home = path.join(fixtures, 'hook');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'runall', 'custom' ])
    // .debug()
    .expect('stdout', new RegExp(`get package name test1 from ${home}/github.com/popomore/test1`))
    .expect('stdout', new RegExp(`get package name test2 from ${home}/github.com/popomore/test2`))
    .expect('code', 0)
    .end(done);
  });

});
