import { Exocore, exocore, LocalNode } from "exocore";
import React, { ChangeEvent } from 'react';
import ReactDOM from 'react-dom';
import List from './list';

interface IAppProps { }

interface IAppState {
    status: string;
    join_pin?: string;
    node?: LocalNode;
}

class App extends React.Component<IAppProps, IAppState> {
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
            return <JoinPin pin={this.state.join_pin} />;
        }

        if (this.state.status === 'connected') {
            return (
                <div>
                    <button onClick={this.rejoin.bind(this)}>Rejoin</button>

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

            <button onClick={this.rejoin.bind(this)}>Rejoin</button>
        </div>);
    }

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
            const disco = Exocore.discovery.create();

            try {
                node = await disco.join_cell(node, (pin: string) => {
                    this.setState({
                        join_pin: pin,
                    })
                });
                node.save_to_storage(localStorage);
            } finally {
                disco.free();
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

        this.setState({
            node,
            join_pin: undefined,
        });
    }

    private rejoin() {
        this.setState({ node: null });
        this.configure(true);
    }

    componentWillUnmount() {
        if (this.state.node) {
            this.state.node.free();
        }
    }
}

interface IJoinPinProps {
    pin: string;
}

class JoinPin extends React.Component<IJoinPinProps, {}> {
    constructor(props: IJoinPinProps) {
        super(props);
    }

    render() {
        return (
            <div>
                <h3>Discovery PIN: {this.props.pin}</h3>
                <h4>Enter this pin on host node (see exo cell node add --help)</h4>
            </div>
        )
    }
}


ReactDOM.render(
    <App />,

    document.getElementById('root')
);

