import "core-js/stable";
import "regenerator-runtime/runtime";

import App from '../../app';
import Navigation, { setupLinkClickNavigation } from "../../navigation";
import React from 'react';
import ReactDOM from 'react-dom';
import { initNode } from '../../exocore';
import Path from "../../utils/path";

// the require added to index.html by electron-webpack isn't working in isolation mode. 
window.exoElectron.installSourceMap();

Navigation.initialize({
    initialPath: new Path('/'),

    openPopup: (path) => {
        window.exoElectron.openPopup(path.toString());
    },

    pushHistory: (/*_path, _replace*/) => { 
        // not supported (yet?)
    },

    openExternal: (url) => {
        window.exoElectron.openExternal(url);
    }
});

window.exoElectron.onNavigate((_event, path) => {
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
        window.exoElectron.openExternal(el.href);
    });
});

