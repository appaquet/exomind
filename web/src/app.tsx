
import React from 'react';
import Layout from './components/layout/layout';
import { StoresInstance, StoresContext } from './stores/stores';


export default class App extends React.Component {
  render(): React.ReactNode {
    return (
      <StoresContext.Provider value={StoresInstance}>
        <Layout />
      </StoresContext.Provider>
    );
  }
}

