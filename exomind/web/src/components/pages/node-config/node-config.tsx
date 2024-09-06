
import React, { ChangeEvent } from 'react';
import { IStores, StoresContext, Stores } from '../../../stores/stores';
import { observer } from 'mobx-react';
import { Discovery, Exocore } from 'exocore';
import { bootNode, initNode, resetNode } from '../../../exocore';
import './node-config.less';
import Navigation from '../../../navigation';

type IProps = Record<string, unknown>;

interface IState {
  join_pin?: string;
  error?: string;
  config: string;
}

@observer
export default class NodeConfig extends React.Component<IProps, IState> {
  static contextType = StoresContext;
  declare context: IStores;

  private mounted = true;
  private disco: Discovery;

  constructor(props: IProps) {
    super(props);

    this.startDiscovery();

    this.state = {
      config: window.localStorage.node_config
    };
  }

  render(): React.ReactNode {
    return (
      <div className="node-config">
        <h2>Node config</h2>

        <div className="body">
          {this.renderError()}

          <div className="discovery">
            <span className="text">Discovery PIN: </span>
            <span className="pin">{this.state.join_pin}</span>
          </div>

          <div className="config">
            <span className="title">Config</span>
            <textarea onChange={this.handleConfigChange} value={this.state.config} />
          </div>

          <div className="actions">
            <button onClick={this.handleNodeReset}>Reset node</button>
            <button onClick={this.handleConfigSave}>Save</button>
          </div>
        </div>
      </div>
    );
  }

  private renderError(): React.ReactNode | null {
    if (this.state.error) {
      return (
        <span className="error">{this.state.error}</span>
      );
    }
  }

  private startDiscovery() {
    const sessionStore = Stores.session;

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
      console.error('Error in discovery', err);
      if (this.mounted) {
        this.setState({
          error: err.message,
        });
      }
    });
  }

  private handleNodeReset = async (): Promise<void> => {
    await resetNode();
    this.startDiscovery();
    this.setState({
      config: window.localStorage.node_config,
    });
  };

  private handleConfigSave = (): void => {
    window.localStorage.node_config = this.state.config;
    initNode();
  };

  private handleConfigChange = (e: ChangeEvent<HTMLTextAreaElement>) => {
    this.setState({
      config: e.target.value,
    });
  };

  componentWillUnmount(): void {
    this.mounted = false;
    this.disco.free();
  }
}

