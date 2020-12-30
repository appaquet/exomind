
import React from 'react';
import Layout from './components/layout/layout';
import Navigation from './navigation';
import { ModalStore } from './store/modal-store.js';
import { StoresInstance, StoresContext } from './store/stores';

export default class App extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      path: Navigation.currentPath,
      modalRenderer: null
    };

    ModalStore.onChange(this.onModalChange, this);
    Navigation.onNavigate(this.onNavigate, this);
  }

  onModalChange() {
    this.setState({
      modalRenderer: ModalStore.currentRenderer
    });
  }

  onNavigate() {
    this.setState({
      path: Navigation.currentPath
    });
  }

  render() {
    return (
      <StoresContext.Provider value={StoresInstance}>
        <Layout
          path={this.state.path}
          modalRenderer={this.state.modalRenderer} />
      </StoresContext.Provider>
    );
  }
}

