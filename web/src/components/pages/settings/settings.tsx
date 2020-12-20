
import React from 'react';
import './settings.less';
import Navigation from '../../../navigation';

export default function Settings(): JSX.Element {
  return (
    <div className="settings">
      <ul>
        <li><a href={Navigation.pathForEntity('favorites')}><span className="fa fa-star" /> Edit favorites</a></li>
        <li><a href={Navigation.pathForNodeConfig()}><span className="fa fa-wrench" /> Node config</a></li>
      </ul>
    </div>
  );
}