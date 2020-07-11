
const path = require('path');
const express = require('express');
const cookieParser = require('cookie-parser');

var server = express();
server.set('port', (process.env.PORT || 8000));
server.use(express.static(path.join(__dirname)));
server.use(cookieParser());

server.get('/logout', (req, res) => {
  res.clearCookie('sid');
  res.send('<script language="javascript">window.location.href="/";</script>');
});

server.get('/login', (req, res) => {
  res.redirect('/v1/auth/google/url');
});

server.get('/switch', (req, res) => {
  let cookieOptions = {
    expires: new Date(Date.now() + 86400000*365)
  };
  res.cookie('bid', req.query['b'], cookieOptions);
  res.cookie('fid', req.query['f'], cookieOptions);
  res.send('<script language="javascript">window.location.href="/";</script>');
});

// handle push state
server.get('/*', function (req, res) {
  res.sendFile(path.join(__dirname, 'index.html'));
});

server.listen(server.get('port'), function () {
  if (process.send) {
    process.send('online');
  } else {
    console.log('Server > Running at http://localhost:' + server.get('port'));
  }
});

