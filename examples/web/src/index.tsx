import { Discovery, Exocore, exocore, LocalNode } from "exocore";
import React, { ChangeEvent, useState } from 'react';
import ReactDOM from 'react-dom';
import List from './list';

interface IAppProps { }

interface IAppState {
    status: string;
    join_pin?: string;
    node?: LocalNode;
}

class App extends React.Component<IAppProps, IAppState> {
    private disco: Discovery = null;

    constructor(props: IAppProps) {
        super(props);

        let state: IAppState = {
            status: 'disconnected',
        };

        Exocore.ensureLoaded().then(() => {
            this.configure();
        });

        this.state = state;
    }

    render() {
        if (this.state.join_pin) {
            return <JoinView pin={this.state.join_pin} onConfigChanged={this.setConfig} />;
        }

        if (this.state.status === 'connected') {
            return (
                <div>
                    <button onClick={this.rejoin}>Rejoin</button>

                    <List />
                </div>
            );
        } else {
            return this.renderLoading();
        }
    }

    renderLoading() {
        return (<div>
            <h3>Connecting...</h3>

            <button onClick={this.rejoin}>Rejoin</button>
        </div>);
    }

    private setConfig = async (yamlConfig: string) => {
        const node = Exocore.node.from_yaml(yamlConfig);
        node.save_to_storage(localStorage);
        await this.createInstance(node);
    };

    private async configure(rejoin?: boolean) {
        let node: LocalNode;
        try {
            node = Exocore.node.from_storage(localStorage)
        } catch { }

        if (!node) {
            node = Exocore.node.generate();
            node.save_to_storage(localStorage);
        }

        if (rejoin || !node.has_configured_cell) {
            this.disco?.free();
            this.disco = Exocore.discovery.create();

            try {
                node = await this.disco.join_cell(node, (pin: string) => {
                    this.setState({
                        join_pin: pin,
                    })
                });
                node.save_to_storage(localStorage);
            } finally {
                this.disco?.free();
                this.disco = null;
            }
        }

        await this.createInstance(node);
    }

    private async createInstance(node: LocalNode) {
        const instance = await Exocore.initialize(node);
        instance.registry.registerMessage(exocore.test.TestMessage, 'exocore.test.TestMessage');
        instance.registry.registerMessage(exocore.test.TestMessage2, 'exocore.test.TestMessage2');
        instance.onChange = () => {
            this.setState({ status: instance.status });
        }

        // this.disco?.free();
        // this.disco = null;

        this.setState({
            node,
            join_pin: undefined,
        });
    }

    private rejoin = () => {
        this.setState({ node: null });
        this.configure(true);
    }

    componentWillUnmount() {
        if (this.state.node) {
            this.state.node.free();
        }
    }
}

function JoinView(props: { pin: string, onConfigChanged: (config: string) => void }) {
    return (
        <div>
            <JoinPin pin={props.pin} />

            <h4>or enter yaml config:</h4>

            <NodeConfig onConfigChanged={props.onConfigChanged} />
        </div>
    )

}

function JoinPin(props: { pin: string }) {
    return (
        <div>
            <h3>Discovery PIN: {props.pin}</h3>
            <h4>Enter this pin on host node (see exo cell node add --help)</h4>
        </div>
    )
}

function NodeConfig(props: { onConfigChanged: (config: string) => void }) {
    const [config, setConfig] = useState('');

    return (
        <div>
            <div>
                <textarea
                    onChange={(e) => setConfig(e.target.value)}
                    style={{ width: '500px', height: '200px' }}
                    id="config"
                    value={config}
                />
            </div>

            <div><button id="config-save" onClick={() => props.onConfigChanged(config)}>Save</button></div>
        </div>
    );
}

ReactDOM.render(
    <App />,

    document.getElementById('root')
);

