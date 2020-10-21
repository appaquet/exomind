
import React from 'react';
import ReactDOM from 'react-dom';
import ReactBridge from './react-bridge.js';
import './hybrid.less';

// used in some places to detect if we're on iOS or not
window.isHybridExomind = true;

// communication to iOS app
window.toIosDataId = 0;
window.toIosData = {};

window.sendIos = (data) => {
  let id = window.toIosDataId++;
  window.toIosData[id] = data;
  window.location = 'exomind://' + id;
};

// For some reason, React needs to be imported first
// This prevents unused warning
window.noop = React;

window.getData = (dataId) => {
  let data = window.toIosData[dataId];
  delete window.toIosData[dataId];
  return JSON.stringify(data);
};

window._startComponent = () => {
  try {
    ReactDOM.render(<ReactBridge />, document.getElementById('body'));
  } catch (e) {
    document.getElementById('body').innerHTML = e;
    alert(e);
    throw(e);
  }
};

// run the application when both DOM is ready and page content is loaded
Promise.all([
  new Promise((resolve) => {
    if (window.addEventListener) {
      window.addEventListener('DOMContentLoaded', resolve);
    } else {
      window.attachEvent('onload', resolve);
    }
  })
]).then(() => {
  // window.component = 'html-editor';
  window.sendIos('ready');
  window._startComponent();
});

