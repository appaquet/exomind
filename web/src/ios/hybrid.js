
import React from 'react';
import ReactDOM from 'react-dom';
import ReactBridge from './react-bridge.js';
import './hybrid.less';

// used in some places to detect if we're on iOS or not
window.isHybridExomind = true;

// callback to iOS
export const sendIos = (data) => {
  // eslint-disable-next-line no-undef
  webkit.messageHandlers['onMessage'].postMessage(data);
};
window.sendIos = sendIos;

// For some reason, React needs to be imported first
// This prevents unused warning
window.noop = React;

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

