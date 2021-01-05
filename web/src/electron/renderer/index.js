import "core-js/stable";
import "regenerator-runtime/runtime";

import App from '../../app';
import Navigation from '../../navigation';
import React from 'react';
import ReactDOM from 'react-dom';
import { initNode } from '../../exocore';
import { setupLinkClickNavigation } from '../../utils';
import * as electron from 'electron';
import Path from "../../utils/path";

Navigation.initialize({
    initialPath: new Path('/'),

    openPopup: (path) => {
        electron.ipcRenderer.send('open-popup', path.toString());
    },

    pushHistory: (_path, _replace) => { 
        // not supported (yet?)
    },

    openExternal: (url) => {
        electron.shell.openExternal(url);
    }
});

electron.ipcRenderer.on('navigate', (event, path) => {
    Navigation.navigate(path);
});

Promise.all([
    new Promise((resolve) => {
        window.addEventListener('DOMContentLoaded', resolve);
    }),
    initNode()
]).then(() => {
    ReactDOM.render(<App />, document.getElementById('body'));

    setupLinkClickNavigation((e, el) => {
        e.preventDefault();
        e.stopPropagation();
        electron.shell.openExternal(el.href);
    });
});

