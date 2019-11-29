import "core-js/stable";
import "regenerator-runtime/runtime";

import React from 'react';
import ReactDOM from 'react-dom';
import List from './list.js';

class App extends React.Component {
    constructor(props) {
        super(props);

        this.state = {exocore: null};
        import("../../../clients/wasm/pkg").then(module => {
            // fix issue where not yet connected until we support transport status
            let client = new module.ExocoreClient("ws://127.0.0.1:3340", (status) => {
                if (status === "connected") {
                    this.setState({
                        exocore: client
                    });
                }
            });
        })
    }

    render() {
        if (this.state.exocore) {
            return <List exocore={this.state.exocore}/>;
        } else {
            return this.renderLoading();
        }
    }

    renderLoading() {
        return <div>Loading...</div>;
    }
}

ReactDOM.render(
    <App/>,
    document.getElementById('root')
);

