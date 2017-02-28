'use strict';

const fs = require('fs');
const path = require('path');
const coffee = require('coffee');
const mm = require('mm');
const rimraf = require('mz-modules/rimraf');
const assert = require('assert');

const binfile = path.join(__dirname, '../bin/projj.js');
const fixtures = path.join(__dirname, 'fixtures');
const tmp = path.join(fixtures, 'tmp');


describe('test/projj_init.test.js', () => {

  afterEach(mm.restore);
  afterEach(() => rimraf(tmp));

  it('should get base directory with relative path', done => {
    const home = path.join(fixtures, 'base-relative');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'init' ])
    // .debug()
    .expect('stdout', new RegExp(`Set base directory: ${home}\n`))
    .expect('code', 0)
    .end(done);
  });

  it('should get base directory with tilde', done => {
    const home = path.join(fixtures, 'base-tilde');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'init' ])
    // .debug()
    .expect('stdout', new RegExp(`Set base directory: ${home}/code\n`))
    .expect('code', 0)
    .end(done);
  });

  it('should set base when config don\'t exist', done => {
    mm(process.env, 'HOME', tmp);
    coffee.fork(binfile, [ 'init' ])
    // .debug()
    .expect('stdout', /Set base directory: /)
    .expect('stdout', /Set base directory: \/home\n/)
    .expect('code', 0)
    .write('/home')
    .end(err => {
      assert.ifError(err);
      assert(fs.existsSync(path.join(tmp, '.projj/config.json')));
      const content = fs.readFileSync(path.join(tmp, '.projj/config.json'), 'utf8');
      assert(content === '{\n  "base": "/home",\n  "hooks": {}\n}');
      done();
    });
  });

  it('should set base with relative path', done => {
    mm(process.env, 'HOME', tmp);
    coffee.fork(binfile, [ 'init' ])
    // .debug()
    .expect('stdout', new RegExp(`Set base directory: ${process.cwd()}/code\n`))
    .expect('code', 0)
    .write('code')
    .end(done);
  });
});
