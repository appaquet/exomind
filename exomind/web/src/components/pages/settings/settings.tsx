
import React from 'react';
import './settings.less';
import Navigation from '../../../navigation';
import { observer } from 'mobx-react';
import { IStores, StoresContext } from '../../../stores/stores';
import { exocore, Exocore, fromProtoTimestamp } from 'exocore';
import DateUtil from '../../../utils/dates';

declare const _EXOMIND_VERSION: string;
declare const _EXOMIND_BUILD_TIME: string;


@observer
export default class Settings extends React.Component {
  static contextType = StoresContext;
  declare context: IStores;

  private buildInfo: exocore.core.BuildInfo;
  private buildTime: Date;

  constructor(props: unknown) {
    super(props);
    this.buildInfo = Exocore.buildInfo();
    this.buildTime = fromProtoTimestamp(this.buildInfo.buildTime);
  }

  render(): React.ReactNode {
    return (
      <div className="settings">
        <ul>
          <li><a href={Navigation.pathForEntity('favorites')}><span className="fa fa-star" /> Edit favorites</a></li>
          <li><a href={Navigation.pathForNodeConfig()}><span className="fa fa-wrench" /> Node config</a></li>
          <li><a href="#" target="local" onClick={this.toggleDarkMode}><span className="fa fa-moon-o" /> Change theme</a></li>
        </ul>

        <div className="version">
          exomind {_EXOMIND_VERSION} ({DateUtil.toLongFormat(new Date(parseInt(_EXOMIND_BUILD_TIME)))})<br />
          exocore {this.buildInfo.version} ({DateUtil.toLongFormat(this.buildTime)})
        </div>
      </div>
    );
  }

  private toggleDarkMode = (e: React.MouseEvent) => {
    e.preventDefault();
    this.context.settings.toggleDarkMode();
    return false;
  };
}