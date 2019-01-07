'use strict';

const path = require('path');
const coffee = require('coffee');
const mm = require('mm');
const rimraf = require('mz-modules/rimraf');
const binfile = path.join(__dirname, '../bin/projj.js');
const fixtures = path.join(__dirname, 'fixtures');
const fs = require('mz/fs');
const home = path.join(fixtures, 'remove');
const runscript = require('runscript');
const assert = require('assert');
const projects = path.join(home, 'projects');
const tempProject = path.join(home, 'temp');
const catchPath = path.join(home, '.projj/cache.json');

describe('test/projj_remove.test.js', () => {
  beforeEach(function* () {
    mm(process.env, 'HOME', home);
    const content = JSON.stringify({
      'github.com/popomore/projj': {},
      'github.com/eggjs/egg': {},
      'github.com/eggjs/egg-core': {},
      'github.com/eggjs/autod-egg': {},
      'gitlab.com/eggjs/egg': {},
      'github.com/DiamondYuan/yuque': {},
    });
    yield runscript(`cp -r ${projects} ${tempProject}`);
    fs.writeFileSync(catchPath, content);
  });

  afterEach(() => {
    afterEach(() => rimraf(tempProject));
    afterEach(() => rimraf(catchPath));
  });


  it('should to prompt if the input is empty', done => {
    coffee.fork(binfile, [ 'remove', '' ])
    .expect('stderr', new RegExp('Please specify the repo name:'))
    .expect('stderr', new RegExp('For example: projj remove example'))
    .expect('code', 0)
    .end(done);
  });

  it('if there are other files in the folder, the folder will not be deleted.', done => {
    coffee.fork(binfile, [ 'remove', 'yuque' ])
    .debug()
    .waitForPrompt()
    .expect('stdout', new RegExp('Do you want to remove the repository github.com/DiamondYuan/yuque'))
    .expect('stdout', new RegExp('Removed repository cannot be restored!'))
    .expect('stdout', new RegExp('Please type in the name of the repository to confirm. DiamondYuan/yuque'))
    .write('DiamondYuan/yuque\n')
    .expect('stdout', new RegExp('Successfully remove repository github.com/DiamondYuan/yuque'))
    .expect('code', 0)
    .end(err => {
      assert.ifError(err);
      assert(fs.existsSync(path.join(tempProject, 'github.com/DiamondYuan')));
      done();
    });
  });

  it('if no other files are in the folder, the folder will be deleted.', done => {
    const folder = path.join(tempProject, 'github.com/popomore');
    coffee.fork(binfile, [ 'remove', 'projj' ])
    .expect('stdout', new RegExp('Do you want to remove the repository github.com/popomore/projj'))
    .expect('stdout', new RegExp('Removed repository cannot be restored!'))
    .expect('stdout', new RegExp('Please type in the name of the repository to confirm. popomore/projj'))
    .write('popomore/projj')
    .expect('stdout', new RegExp(`Successfully remove empty folder ${folder}`))
    .expect('code', 0)
    .end(err => {
      assert.ifError(err);
      assert(fs.existsSync(folder) === false);
      done();
    });
  });

  it('should update cache that do not exist', done => {
    coffee.fork(binfile, [ 'remove', 'autod-egg' ])
    .expect('stdout', new RegExp('Do you want to remove the repository github.com/eggjs/autod-egg'))
    .expect('stdout', new RegExp('Removed repository cannot be restored!'))
    .expect('stdout', new RegExp('Please type in the name of the repository to confirm. eggjs/autod-egg'))
    .write('eggjs/autod-egg')
    .expect('stdout', new RegExp('remove github.com/eggjs/autod-egg that don\'t exist'))
    .expect('code', 0)
    .end(err => {
      assert.ifError(err);
      const cache = fs.readFileSync(catchPath);
      assert(JSON.parse(cache.toString())['github.com/eggjs/autod-egg'] === undefined);
      done();
    });
  });

  it('could retry if the input is incorrect', done => {
    coffee.fork(binfile, [ 'remove', 'autod-egg' ])
    .waitForPrompt()
    .expect('stdout', new RegExp('Do you want to remove the repository github.com/eggjs/autod-egg'))
    .expect('stdout', new RegExp('Removed repository cannot be restored!'))
    .expect('stdout', new RegExp('Please type in the name of the repository to confirm. eggjs/autod-egg'))
    .write('eggjs/egg\n')
    .expect('stdout', new RegExp('Do you want to continue'))
    .write('Y\n')
    .write('eggjs/autod-egg')
    .expect('stdout', new RegExp('remove github.com/eggjs/autod-egg that don\'t exist'))
    .expect('code', 0)
    .end(done);
  });

  it('could cancel if the input is incorrect', done => {
    coffee.fork(binfile, [ 'remove', 'autod-egg' ])
    .waitForPrompt()
    .expect('stdout', new RegExp('Do you want to remove the repository github.com/eggjs/autod-egg'))
    .expect('stdout', new RegExp('Removed repository cannot be restored!'))
    .expect('stdout', new RegExp('Please type in the name of the repository to confirm. eggjs/autod-egg'))
    .write('eggjs/egg\n')
    .expect('stdout', new RegExp('Do you want to continue'))
    .write('\n')
    .expect('stdout', new RegExp('Cancel remove repository {2}github.com/eggjs/autod-egg'))
    .expect('code', 0)
    .end(done);
  });

  it('should find two matchs file with egg', done => {
    coffee.fork(binfile, [ 'remove', 'egg' ])
    .expect('stdout', new RegExp('Please select the correct repo'))
    .write('\n')
    .expect('stdout', new RegExp('Do you want to remove the repository github.com/eggjs/egg'))
    .expect('code', 0)
    .end(done);
  });
});
