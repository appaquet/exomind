import { observer } from 'mobx-react';
import React from 'react';
import { ContainerState } from '../../objects/container-state';
import { EntityComponent } from '../../objects/entity-component';
import './fullscreen.less';

interface IProps {
  entityId: string;
}

@observer
export default class Fullscreen extends React.Component<IProps> {
  private containerState: ContainerState;

  constructor(props: IProps) {
    super(props);

    this.containerState = new ContainerState();
    this.containerState.active = true;
  }

  render(): React.ReactNode {
    document.title = 'Exomind - ' + this.containerState.title;

    return (
      <div className="fullscreen">
        <EntityComponent
          entityId={this.props.entityId}
          containerState={this.containerState}/>
      </div>
    );
  }
}
