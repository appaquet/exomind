import { observable } from 'mobx';
import { observer } from 'mobx-react';
import React from 'react';
import Navigation from '../../../navigation';
import { ContainerState } from '../../objects/container-state';
import { EntityComponent } from '../../objects/entity-component';
import './fullscreen.less';

interface IProps {
  entityId: string;
}

@observer
export default class Fullscreen extends React.Component<IProps> {
  @observable private containerState: ContainerState = new ContainerState();

  constructor(props: IProps) {
    super(props);

    this.containerState.active = true;
  }

  render(): React.ReactNode {
    document.title = 'Exomind - ' + this.containerState.title;

    if (this.containerState.closed) {
      Navigation.closeWindow();
    }

    return (
      <div className="fullscreen">
        <EntityComponent
          entityId={this.props.entityId}
          containerState={this.containerState} />
      </div>
    );
  }
}
