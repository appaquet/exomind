import classNames from 'classnames';
import { observer } from 'mobx-react';
import React from 'react';
import Navigation from '../../navigation';
import { IStores, StoresContext } from '../../stores/stores';
import NodeConfig from '../pages/node-config/node-config';
import { ColumnConfigs } from '../pages/columns/columns-config';
import Columns from '../pages/columns/columns';
import Fullscreen from '../pages/fullscreen/fullscreen';
import Home from '../pages/home/home.js';
import NotFound from '../pages/not-found/not-found';
import Settings from '../pages/settings/settings';
import Hamburger from './hamburger/hamburger';
import { Header } from './header/header';
import './layout.less';
import Modal from './modal';

@observer
export default class Layout extends React.Component {
  static contextType = StoresContext;
  declare context: IStores;

  render(): React.ReactNode {
    const classes = classNames({
      'hamburger-open': this.showHamburger,
      'fullscreen': this.isFullscreen,
      'dark-mode': this.context.settings.darkMode,
    });

    const hamburgerMenu = (this.showHamburger) ?
      <Hamburger
        path={this.context.session.currentPath} /> : null;

    const header = (!this.isFullscreen) ? (
      <Header
        path={this.context.session.currentPath} />
    ) : null;

    return (
      <div id="layout" className={classes}>
        {header}

        {hamburgerMenu}

        {this.renderModal()}

        <div id="content">
          {this.renderPath()}
        </div>
      </div>
    );
  }

  private get showHamburger(): boolean {
    return !this.isFullscreen && this.context.session.cellInitialized;
  }

  private get isFullscreen(): boolean {
    return Navigation.isFullscreenPath(this.context.session.currentPath);
  }

  private handleColumnsChange(config: ColumnConfigs): void {
    Navigation.navigate(Navigation.pathForColumnsConfig(config));
  }

  private renderModal(): React.ReactNode | null {
    if (this.context.session.currentModal) {
      return <Modal>{this.context.session.currentModal()}</Modal>;
    }
  }

  private renderPath(): React.ReactNode {
    const path = this.context.session.currentPath;
    if (Navigation.isNodeConfigPath(path) || this.context.session.showDiscovery) {
      return <NodeConfig />;

    } else if (this.context.session.currentPath.isRoot()) {
      if (this.context.session.cellInitialized) {
        Navigation.navigate(Navigation.pathForInbox());
      }
      return <Home />;

    } else if (Navigation.isColumnsPath(path) && this.context.session.cellInitialized) {
      const config = path.drop(1).toString();
      return <Columns
        config={config}
        onConfigChange={this.handleColumnsChange} />;

    } else if (Navigation.isFullscreenPath(path)) {
      return <Fullscreen entityId={path.drop(1).take(1).toString()} />

    } else if (Navigation.isSettingsPath(path)) {
      return <Settings />;

    } else {
      return <NotFound />;
    }
  }
}

