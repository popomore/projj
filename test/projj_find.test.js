'use strict';

const path = require('path');
const coffee = require('coffee');
const mm = require('mm');
const fs = require('mz/fs');
const rimraf = require('mz-modules/rimraf');

const binfile = path.join(__dirname, '../bin/projj.js');
const fixtures = path.join(__dirname, 'fixtures');
const tmp = path.join(fixtures, 'tmp');


describe('test/projj_find.test.js', () => {

  afterEach(mm.restore);
  afterEach(() => rimraf(tmp));

  it('should run script when changeDirectory is true and platform is darwin', function* () {
    const home = path.join(fixtures, 'find-change-directory');
    mm(process.env, 'HOME', home);
    yield makeConfig(home);

    yield coffee.fork(binfile, [ 'find', 'egg' ])
      // .debug()
      .beforeScript(path.join(__dirname, 'fixtures/mock_darwin.js'))
      .expect('stdout', new RegExp(`Change directory to ${home}/github.com/eggjs/egg`))
      // .expect('stdout', new RegExp(`cd ${home}/github.com/eggjs/egg`))
      .expect('code', 0)
      .end();
  });

  it('should show warn when changeDirectory is true and platform is not darwin', function* () {
    const home = path.join(fixtures, 'find-change-directory');
    mm(process.env, 'HOME', home);
    yield makeConfig(home);

    yield coffee.fork(binfile, [ 'find', 'egg' ])
      .beforeScript(path.join(__dirname, 'fixtures/mock_not_darwin.js'))
      .expect('stderr', new RegExp('Change directory only supported in darwin'))
      .expect('stdout', new RegExp(`find repo egg's location: ${home}/github.com/eggjs/egg`))
      .expect('code', 0)
      .end();
  });

  it('should to prompt if the input is empty', function* () {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);

    yield coffee.fork(binfile, [ 'find', '' ])
      .expect('stderr', new RegExp('Please specify the repo name:'))
      .expect('stderr', new RegExp('For example: projj find example'))
      .expect('code', 0)
      .end();
  });

  it('should find endsWith egg', function* () {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);
    yield makeConfig(home);

    yield coffee.fork(binfile, [ 'find', 'egg' ])
      // .debug()
      .expect('stdout', new RegExp(`find repo egg's location: ${home}/github.com/eggjs/egg`))
      .expect('code', 0)
      .end();
  });

  it('should find endsWith /egg', function* () {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);
    yield makeConfig(home);

    yield coffee.fork(binfile, [ 'find', '/egg' ])
      // .debug()
      .expect('stdout', new RegExp(`find repo /egg's location: ${home}/github.com/eggjs/egg`))
      .expect('code', 0)
      .end();
  });

  it('should find match eggjs/autod', function* () {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);
    yield makeConfig(home);

    yield coffee.fork(binfile, [ 'find', 'eggjs/autod' ])
      // .debug()
      .expect('stdout', new RegExp(`find repo eggjs/autod's location: ${home}/gitlab.com/eggjs/autod-egg`))
      .expect('code', 0)
      .end();
  });

  it('should find two matching file with egg-core', function* () {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);
    yield makeConfig(home);

    yield coffee.fork(binfile, [ 'find', 'egg-core' ])
      .write('\n')
      // .debug()
      .expect('stdout', new RegExp('Please select the correct repo'))
      .expect('stdout', new RegExp(`find repo egg-core's location: ${home}/github.com/eggjs/egg-core`))
      .expect('code', 0)
      .end();
  });

  it('should find nothing with eggggg', function* () {
    const home = path.join(fixtures, 'find');
    mm(process.env, 'HOME', home);
    yield makeConfig(home);

    yield coffee.fork(binfile, [ 'find', 'eggggg' ])
      // .debug()
      .expect('stderr', new RegExp('Can not find repo eggggg'))
      .expect('code', 0)
      .end();
  });
});

function* makeConfig(cwd) {
  const config = {
    [path.join(cwd, 'github.com/eggjs/egg')]: {
      repo: 'git@github.com:eggjs/egg.git',
    },
    [path.join(cwd, 'github.com/eggjs/egg-core')]: {
      repo: 'git@github.com:eggjs/egg-core.git',
    },
    [path.join(cwd, 'gitlab.com/eggjs/egg-core')]: {
      repo: 'git@gitlab.com:eggjs/egg-core.git',
    },
    [path.join(cwd, 'gitlab.com/eggjs/autod-egg')]: {
      repo: 'git@gitlab.com:eggjs/autod-egg.git',
    },
    version: 'v1',
  };
  yield fs.writeFile(path.join(cwd, '.projj/cache.json'), JSON.stringify(config));
}
