import { observer } from 'mobx-react';
import React from 'react';
import { ContainerController } from '../../objects/container-controller';
import { EntityComponent } from '../../objects/entity-component';
import './fullscreen.less';

interface IProps {
  entityId: string;
}

@observer
export default class Fullscreen extends React.Component<IProps> {
  private containerController: ContainerController;

  constructor(props: IProps) {
    super(props);
    this.containerController = new ContainerController();
  }

  render(): React.ReactNode {
    document.title = 'Exomind - ' + this.containerController.title;

    return (
      <div className="fullscreen">
        <EntityComponent
          entityId={this.props.entityId}
          containerController={this.containerController}/>
      </div>
    );
  }
}
