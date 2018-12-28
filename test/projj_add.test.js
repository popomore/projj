'use strict';

const fs = require('fs');
const path = require('path');
const coffee = require('coffee');
const mm = require('mm');
const rimraf = require('mz-modules/rimraf');
const assert = require('assert');
const mkdirp = require('mkdirp');

const binfile = path.join(__dirname, '../bin/projj.js');
const fixtures = path.join(__dirname, 'fixtures');
const tmp = path.join(fixtures, 'tmp');


describe('test/projj_add.test.js', () => {

  afterEach(mm.restore);
  afterEach(() => rimraf(tmp));

  it('should add a git repo', done => {
    const home = path.join(fixtures, 'base-tmp');
    const cachePath = path.join(home, '.projj/cache.json');
    const repo = 'https://github.com/popomore/projj.git';
    const target = path.join(tmp, 'github.com/popomore/projj');
    mm(process.env, 'HOME', home);

    fs.writeFileSync(cachePath, JSON.stringify({
      'github.com/popomore/test1': {},
      'github.com/popomore/test2': { repo: 'https://github.com/popomore/projj.git' },
    }));

    coffee.fork(binfile, [ 'add', repo ])
    // .debug()
      .expect('stdout', new RegExp(`Start adding repository ${repo}`))
      .expect('stdout', new RegExp(`Cloning into ${target}`))
      .expect('code', 0)
      .end(err => {
        assert.ifError(err);
        assert(fs.existsSync(path.join(target, 'package.json')));

        const cache = JSON.parse(fs.readFileSync(cachePath));
        assert(cache['github.com/popomore/projj']);
        assert(cache['github.com/popomore/projj'].repo === 'https://github.com/popomore/projj.git');
        assert(cache['github.com/popomore/test1'].repo === 'git@github.com:popomore/test1.git');
        assert(cache['github.com/popomore/test2'].repo === 'https://github.com/popomore/projj.git');
        done();
      });
  });

  it('should throw when target exists', function* () {
    const home = path.join(fixtures, 'base-tmp');
    const repo = 'https://github.com/popomore/projj.git';
    const target = path.join(tmp, 'github.com/popomore/projj');
    mm(process.env, 'HOME', home);
    yield mkdir(target);

    yield coffee.fork(binfile, [ 'add', repo ])
    // .debug()
      .expect('stderr', new RegExp(`${target} already exist`))
      .expect('code', 1)
      .end();
  });

  it('should run hook', done => {
    const home = path.join(fixtures, 'hook-add');
    const repo = 'https://github.com/popomore/test.git';
    const target = path.join(tmp, 'github.com/popomore/test');
    mm(process.env, 'HOME', home);
    coffee.fork(binfile, [ 'add', repo ])
    // .debug()
      .expect('stdout', new RegExp(`pre hook, cwd ${process.cwd()}`))
      .expect('stdout', new RegExp(`post hook, cwd ${target}`))
      .expect('stdout', /pre hook, get package name projj/)
      .expect('stdout', /post hook, get package name spm-bump/)
      .expect('code', 0)
      .end(done);
  });

});

function mkdir(file) {
  return new Promise((resolve, reject) => {
    mkdirp(file, err => {
      err ? reject(err) : resolve();
    });
  });
}
