import "core-js/stable";
import "regenerator-runtime/runtime";

import App from './app';
import Navigation from './navigation';
import React from 'react';
import ReactDOM from 'react-dom';
import { ensureLoaded } from './exocore';

class ClientApp extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            path: Navigation.currentPath
        };

        Navigation.onNavigate(this.onNavigate, this);
    }

    onNavigate() {
        this.setState({
            path: Navigation.currentPath
        });
    }

    render() {
        return <App path={this.state.path} />;
    }
}


// Run the application when both DOM is ready and page content is loaded
Promise.all([
    new Promise((resolve) => {
        if (window.addEventListener) {
            window.addEventListener('DOMContentLoaded', resolve);
        } else {
            window.attachEvent('onload', resolve);
        }
    }),
    ensureLoaded()
]).then(() => {
    ReactDOM.render(<ClientApp />, document.getElementById('body'));

    document.addEventListener('click', (e) => {
        var el = e.target;

        // if tagname is not a link, try to go up into the parenthood up 5 levels
        for (var i = 0; el.tagName !== 'A' && i < 5; i++) {
            if (el.parentNode) {
                el = el.parentNode;
            }
        }

        if (el.tagName === 'A') {
            let url = el.getAttribute('href');

            // if it's a local URL, we catch it and send it to navigation
            if (url.startsWith('/') || url.startsWith(window.location.origin) && !el.getAttribute('target')) {
                Navigation.navigate(url);
                e.preventDefault();
                e.stopPropagation();
                return false;
            }
        }
    });
})