
import React, { ChangeEvent } from 'react';
import './bootstrap.less';
import { StoresInstance } from '../../../store/stores';

type IProps = Record<string, unknown>;
interface IState {
  config: string;
  error?: string;
}

export default class Loading extends React.Component<IProps, IState> {
  constructor(props: IProps) {
    super(props);

    const config = StoresInstance.settings.exocoreConfig ?? {};
    this.state = {
      config: JSON.stringify(config, null, '  '),
    }
  }

  render(): React.ReactNode {
    return (
      <div className="node-bootstrap">
        <h2>Node boostrap</h2>

        <div className="input">
          <textarea value={this.state.config} onChange={this.handleChange.bind(this)}/>
        </div>

        <div className="save">
          {this.renderError()}

          <button onClick={this.handleSave.bind(this)} disabled={this.state.error ? true : false}>Save</button>
        </div>
      </div>
    );
  }

  private renderError(): React.ReactNode | null {
    if (this.state.error) {
      return (
        <span className="error">{this.state.error}</span>
      )
    } else {
      return null;
    }
  }

  private handleChange(e: ChangeEvent<HTMLTextAreaElement>): void {
    let error;
    try {
      JSON.parse(e.target.value);
    } catch (e) {
      error = e.toString();
    }

    this.setState({
      config: e.target.value,
      error: error,
    });
  }

  private handleSave(): void {
    StoresInstance.settings.exocoreConfig = JSON.parse(this.state.config);
  }
}

