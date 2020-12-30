
import React from 'react';
import { Stores, StoresContext, StoresInstance } from '../../../store/stores';
import { observer } from 'mobx-react';
import { Discovery, Exocore } from 'exocore';
import { bootNode, resetNode } from '../../../exocore';
import './node-config.less';
import Navigation from '../../../navigation';

type IProps = Record<string, unknown>;
interface IState {
  join_pin?: string;
  error?: string;
}

@observer
export default class NodeConfig extends React.Component<IProps, IState> {
  private mounted = true;
  private disco: Discovery;

  static contextType = StoresContext;
  context: Stores;

  constructor(props: IProps) {
    super(props);

    this.startDiscovery();

    this.state = {};
  }

  render(): React.ReactNode {
    return (
      <div className="node-config">
        <h2>Node config</h2>

        {this.renderError()}

        <div className="discovery">
          <h3>Join cell</h3>
          <span className="text">Discovery PIN: </span>
          <span className="pin">{this.state.join_pin}</span>
        </div>

        <div className="actions">
          <h3>Actions</h3>
          <button onClick={this.handleNodeReset.bind(this)}>Reset node</button>
        </div>
      </div>
    );
  }

  private renderError(): React.ReactNode | null {
    if (this.state.error) {
      return (
        <span className="error">{this.state.error}</span>
      )
    }
  }

  private startDiscovery() {
    const sessionStore = StoresInstance.session;

    if (this.disco) {
      this.disco.free();
    }

    this.disco = Exocore.discovery.create();
    this.disco.join_cell(sessionStore.node, (pin: string) => {
      if (this.mounted) {
        this.setState({
          join_pin: pin,
        });
      }
    }).then((node) => {
      node.save_to_storage(localStorage);
      sessionStore.node = node;

      bootNode();

      Navigation.navigate(Navigation.pathForInbox());
    }).catch((err) => {
      console.log('Error in discovery', err);
      if (this.mounted) {
        this.setState({
          error: err.message,
        })
      }
    });
  }

  private async handleNodeReset(): Promise<void> {
    await resetNode();
    this.startDiscovery();
  }

  componentWillUnmount(): void {
    this.mounted = false;
    this.disco.free();
  }
}

