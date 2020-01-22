'use strict';

const path = require('path');
const coffee = require('coffee');
const binfile = path.join(__dirname, '../bin/projj.js');


describe('test/projj.test.js', () => {

  it('should show help info', done => {
    coffee.fork(binfile, [])
      // .debug()
      .expect('stdout', /Usage: \[command] \[options]/)
      .expect('stdout', /init\ +Initialize configuration/)
      .expect('stdout', /add\ +Add repository/)
      .expect('stdout', /run\ +Run hook in current directory/)
      .expect('stdout', /runall\ +Run hook in every repository/)
      .expect('stdout', /import\ +Import repositories from existing directory/)
      .expect('code', 0)
      .end(done);
  });

  it('should show version', done => {
    coffee.fork(binfile, [ '-v' ])
      // .debug()
      .expect('stdout', require('../package.json').version + '\n')
      .expect('code', 0)
      .end(done);
  });

});
