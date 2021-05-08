
import React from 'react';
import Layout from './components/layout/layout';
import Navigation from './navigation';
import { ModalRenderer, ModalStore } from './stores/modal-store';
import { StoresInstance, StoresContext } from './stores/stores';
import Path from './utils/path';

type IProps = Record<string, unknown>;

interface IState {
  path?: Path;
  modalRenderer?: ModalRenderer;
}

export default class App extends React.Component<IProps, IState> {
  constructor(props: IProps) {
    super(props);

    this.state = {
      path: Navigation.currentPath,
      modalRenderer: null
    };

    ModalStore.onChange(this.onModalChange, this);
    Navigation.onNavigate(this.onNavigate, this);
  }

  render(): React.ReactNode {
    return (
      <StoresContext.Provider value={StoresInstance}>
        <Layout
          path={this.state.path}
          modalRenderer={this.state.modalRenderer} />
      </StoresContext.Provider>
    );
  }

  private onModalChange(): void {
    this.setState({
      modalRenderer: ModalStore.currentRenderer
    });
  }

  private onNavigate(): void {
    this.setState({
      path: Navigation.currentPath
    });
  }
}

