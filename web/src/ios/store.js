'use strict';

exomind.timeLogic = require('../logic/time-logic').default;
exomind.emailsLogic = require('../logic/emails-logic').default;

exomind.arrayLength = (ar) => {
  if (ar) {
    return ar.length;
  } else {
    return 0;
  }
};
