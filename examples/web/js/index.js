import "core-js/stable";
import "regenerator-runtime/runtime";
import { getClient } from "exocore";
import { Entity, Trait, TestMessage } from "exocore";

import React from 'react';
import ReactDOM from 'react-dom';
import List from './list.js';

class App extends React.Component {
    constructor(props) {
        super(props);

        this.state = {exocore: null};

        console.log('Connecting...');
        let msg = new TestMessage();
        msg.string1 = "bob";

        getClient().then(module => {
            // fix issue where not yet connected until we support transport status
            let client = new module.ExocoreClient("ws://127.0.0.1:3340", (status) => {
                console.log('Status ' + status);
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
            return (<div>
                <button onClick={this.disconnect.bind(this)}>Disconnect</button>

                <List exocore={this.state.exocore}/>
            </div>);
        } else {
            return this.renderLoading();
        }
    }

    renderLoading() {
        return <div>Loading...</div>;
    }

    disconnect() {
        this.setState({exocore: null});
    }
}

ReactDOM.render(
    <App/>,
    document.getElementById('root')
);

