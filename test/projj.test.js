'use strict';

const path = require('path');
const coffee = require('coffee');
const binfile = path.join(__dirname, '../bin/projj.js');


describe('test/projj.test.js', () => {

  it('should show help info', done => {
    coffee.fork(binfile, [])
    // .debug()
    .expect('stdout', /Usage: {2}\[command] \[options]/)
    .expect('code', 1)
    .end(done);
  });

  it('should show version', done => {
    coffee.fork(binfile, [ '-V' ])
    // .debug()
    .expect('stdout', require('../package.json').version + '\n')
    .expect('code', 0)
    .end(done);
  });

});
