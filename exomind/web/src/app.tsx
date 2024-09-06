
import React from 'react';
import Layout from './components/layout/layout';
import { Stores, StoresContext } from './stores/stores';

export default class App extends React.Component {
  render(): React.ReactNode {
    return (
      <StoresContext.Provider value={Stores}>
        <Layout />
      </StoresContext.Provider>
    );
  }
}
