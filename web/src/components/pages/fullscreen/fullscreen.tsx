import React from 'react';
import { ContainerController } from '../../objects/container-controller';
import { EntityComponent } from '../../objects/entity-component';
import './fullscreen.less';

interface IProps {
  entityId: string;
}

export default class Fullscreen extends React.Component<IProps> {
  constructor(props: IProps) {
    super(props);
  }

  render(): React.ReactNode {
    const containerController = new ContainerController();
    containerController.onChange(() => {
      document.title = 'Exomind - ' + containerController.title;
    }, this);

    return (
      <div className="fullscreen">
        <EntityComponent
          entityId={this.props.entityId}
          containerController={containerController}/>
      </div>
    );
  }
}
