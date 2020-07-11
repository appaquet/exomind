
import PropTypes from 'prop-types';
import React from 'react';
import Layout from './components/layout/layout';
import { ModalStore } from './store/modal-store.js';
import { StoresInstance, StoresContext } from './store/stores';

export default class App extends React.Component {
  static propTypes = {
    path: PropTypes.object.isRequired
  };

  constructor(props) {
    super(props);

    this.state = {
      modalRenderer: null
    };

    ModalStore.onChange(this.onModalChange, this);
  }

  onModalChange() {
    this.setState({
      modalRenderer: ModalStore.currentRenderer
    });
  }

  render() {
    return (
      <StoresContext.Provider value={StoresInstance}>
        <Layout
          path={this.props.path}
          modalRenderer={this.state.modalRenderer} />
      </StoresContext.Provider>
    );
  }
}

