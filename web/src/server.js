/* eslint-env node */

import path from 'path';
import express from 'express';
import cookieParser from 'cookie-parser';

var server = express();
server.set('port', (process.env.PORT || 8080));
server.use(express.static(path.join(__dirname)));
server.use(cookieParser());

// handle push state
server.get('/*', function (_req, res) {
  res.sendFile(path.join(__dirname, 'index.html'));
});

server.listen(server.get('port'), function () {
  if (process.send) {
    process.send('online');
  } else {
    console.log('Server > Running at http://localhost:' + server.get('port'));
  }
});

