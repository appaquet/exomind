
import React from 'react';
import './settings.less';
import Navigation from '../../../navigation';

export default function Settings(): JSX.Element {
  return (
    <div className="settings">
      <ul>
        <li><a href={Navigation.pathForEntity('favorites')}><span className="fa fa-star" /> Edit favorites</a></li>
        <li><a href={Navigation.pathForBootstrap()}><span className="fa fa-wrench" /> Node bootstrap</a></li>
      </ul>
    </div>
  );
}