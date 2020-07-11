import classNames from 'classnames';
import { observer } from 'mobx-react';
import React from 'react';
import Navigation from '../../navigation';
import { Stores, StoresContext } from '../../store/stores';
import Path from '../../utils/path';
import Bootstrap from '../pages/bootstrap/bootstrap';
import { ColumnsConfig } from '../pages/columns/columns-config';
import Columns from '../pages/columns/columns.js';
import Fullscreen from '../pages/fullscreen/fullscreen';
import Home from '../pages/home/home.js';
import NotFound from '../pages/not-found/not-found';
import Settings from '../pages/settings/settings';
import Hamburger from './hamburger/hamburger';
import { Header } from './header/header';
import './layout.less';
import Modal from './modal';

class IProps {
  path: Path;
  modalRenderer?: () => React.ReactNode;
}

@observer
export default class Layout extends React.Component<IProps> {
  static contextType = StoresContext;
  context: Stores;

  constructor(props: IProps) {
    super(props);
  }

  render(): React.ReactNode {
    const classes = classNames({
      'hamburger-open': this.showHamburger,
      'fullscreen': this.isFullscreen,
    });

    const hamburgerMenu = (this.showHamburger) ?
      <Hamburger
        path={this.props.path} /> : null;

    const header = (!this.isFullscreen) ? (
      <Header
        path={this.props.path} />
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

  private get exocoreInitialized(): boolean {
    return this.context.session.exocoreInitialized;
  }

  private get showHamburger(): boolean {
    return !this.isFullscreen && this.context.session.exocoreInitialized;
  }

  private get isFullscreen(): boolean {
    return Navigation.isFullscreenPath(this.props.path);
  }

  private handleColumnsChange(config: ColumnsConfig): void {
    Navigation.navigate(Navigation.pathForColumnsConfig(config));
  }

  private renderModal(): React.ReactNode | null {
    if (this.props.modalRenderer) {
      return <Modal>{this.props.modalRenderer()}</Modal>;
    }
  }

  private renderPath(): React.ReactNode {
    const path = this.props.path;
    if (Navigation.isBootstrapPath(path)) {
      return <Bootstrap />;

    } else if (this.props.path.isRoot() || !this.exocoreInitialized) {
      if (this.exocoreInitialized) {
        Navigation.navigate(Navigation.pathForInbox());
      }
      return <Home />;

    } else if (Navigation.isColumnsPath(path) && this.exocoreInitialized) {
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

