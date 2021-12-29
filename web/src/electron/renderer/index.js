import "core-js/stable";
import "regenerator-runtime/runtime";

import App from '../../app';
import Navigation, { InMemoryHistory, setupLinkClickNavigation } from "../../navigation";
import React from 'react';
import ReactDOM from 'react-dom';
import { initNode } from '../../exocore';

// the require added to index.html by electron-webpack isn't working in isolation mode. 
window.exoElectron.installSourceMap();

Navigation.initialize({
    history: new InMemoryHistory(),

    openPopup: (path) => {
        window.exoElectron.openPopup(path.toString());
    },

    openExternal: (url) => {
        window.exoElectron.openExternal(url);
    }
});

// TODO: move to global shortcut manager
document.addEventListener('keydown', (e) => {
    if (e.key == 'ArrowLeft' && e.metaKey) {
        Navigation.navigateBack();
        e.stopPropagation();
    } else if (e.key == 'ArrowRight' && e.metaKey) {
        Navigation.navigateForward();
        e.stopPropagation();
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

