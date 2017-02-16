'use strict';

const path = require('path');
const coffee = require('coffee');
const mm = require('mm');
const rimraf = require('rimraf');

const binfile = path.join(__dirname, '../bin/projj.js');
const fixtures = path.join(__dirname, 'fixtures');
const tmp = path.join(fixtures, 'tmp');


describe('test/projj_find.test.js', () => {

  afterEach(mm.restore);
  afterEach(done => rimraf(tmp, done));

  it('should find endsWith egg', done => {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'find', 'egg' ])
    // .debug()
    .expect('stdout', new RegExp(`find repo egg's location: ${home}/github.com/eggjs/egg`))
    .expect('code', 0)
    .end(done);
  });

  it('should find endsWith /egg', done => {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'find', '/egg' ])
    // .debug()
    .expect('stdout', new RegExp(`find repo /egg's location: ${home}/github.com/eggjs/egg`))
    .expect('code', 0)
    .end(done);
  });

  it('should find match eggjs/autod', done => {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'find', 'eggjs/autod' ])
    // .debug()
    .expect('stdout', new RegExp(`find repo eggjs/autod's location: ${home}/gitlab.com/eggjs/autod-egg`))
    .expect('code', 0)
    .end(done);
  });

  it('should find tow matchs file with egg-core', done => {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'find', 'egg-core' ])
    .write('\n')
    // .debug()
    .expect('stdout', new RegExp('Please select the correct repo'))
    .expect('stdout', new RegExp(`find repo egg-core's location: ${home}/github.com/eggjs/egg-core`))
    .expect('code', 0)
    .end(done);
  });

  it('should find nothing with eggggg', done => {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'find', 'eggggg' ])
    // .debug()
    .expect('stderr', new RegExp('Can not find repo eggggg'))
    .expect('code', 0)
    .end(done);
  });
});
