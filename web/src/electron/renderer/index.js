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

if (window.location.hash) {
    window.location.hash = '';
}
const initialUrl = window.location.toString().replace('#', '');

Navigation.initialize({
    initialPath: new Path('/'),

    openPopup: (path) => {
        window.exoElectron.openPopup(path.toString());
    },

    pushHistory: (path, replace) => { 
        if (replace) {
            window.history.replaceState({}, null, initialUrl + '#' + path.toString());
        } else {
            window.history.pushState({}, null, initialUrl + '#' + path.toString());
        }
    },

    openExternal: (url) => {
        window.exoElectron.openExternal(url);
    }
});

window.onpopstate = () => {
    Navigation.currentPath = new Path(window.location.hash.replace('#', ''));
};

document.addEventListener('keydown', (e) => {
    if (e.key == 'ArrowLeft' && e.metaKey && e.shiftKey) {
        window.history.back();
        e.stopPropagation();
        e.preventDefault();
    } else if (e.key == 'ArrowRight' && e.metaKey && e.shiftKey) {
        window.history.forward();
        e.stopPropagation();
        e.preventDefault();
    }
}, false);


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

