import "core-js/stable";
import "regenerator-runtime/runtime";

import App from './app';
import React from 'react';
import ReactDOM from 'react-dom';
import { initNode } from './exocore';
import Navigation, { setupLinkClickNavigation } from "./navigation";
import Constants from "./constants";
import Path from "./utils/path";


function getBrowserCurrentPath() {
    return new Path(window.location.pathname.replace(Constants.basePath, ''));
}

Navigation.initialize({
    initialPath: getBrowserCurrentPath(),

    openPopup: (path) => {
        let url = Constants.webUrl + Constants.basePath + path.toString();
        window.open(url, '_blank', 'menubar=no,location=no,status=no,titlebar=no,toolbar=no');
    },

    pushHistory: (path, replace) => {
        if (!window.history) {
            return;
        }

        if (replace) {
            window.history.replaceState({}, null, Constants.basePath + path.toString());
        } else {
            window.history.pushState({}, null, Constants.basePath + path.toString());
        }
    },

    openExternal: (url) => {
        window.open(url, '_blank');
    }
});

window.onpopstate = () => {
    Navigation.currentPath = getBrowserCurrentPath();
    Navigation.notifyChange();
};

Promise.all([
    new Promise((resolve) => {
        if (window.addEventListener) {
            window.addEventListener('DOMContentLoaded', resolve);
        } else {
            window.attachEvent('onload', resolve);
        }
    }),
    initNode()
]).then(() => {
    ReactDOM.render(<App />, document.getElementById('body'));
    setupLinkClickNavigation();
});