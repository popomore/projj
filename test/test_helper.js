'use strict';

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function literalPattern(value) {
  return new RegExp(escapeRegExp(value));
}

function pathPattern(value) {
  return new RegExp(escapeRegExp(value).replace(/\//g, '[/\\\\]'));
}

module.exports = {
  escapeRegExp,
  literalPattern,
  pathPattern,
};
