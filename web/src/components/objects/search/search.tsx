import { Exocore, exocore, MutationBuilder, QueryBuilder } from 'exocore';
import React from 'react';
import { exomind } from '../../../protos';
import { EntityTraits } from '../../../utils/entities';
import { ModalStore } from '../../../stores/modal-store';
import { ExpandableQuery } from '../../../stores/queries';
import { CollectionSelector } from '../../modals/collection-selector/collection-selector';
import { ContainerController } from '../container-controller';
import { ButtonAction, EntityActions } from '../entity-list/entity-action';
import { EntityList } from '../entity-list/entity-list';
import { Selection } from '../entity-list/selection';
import { Message } from '../message';
import './search.less';

interface IProps {
  query: string;

  selection?: Selection;
  onSelectionChange?: (sel: Selection) => void;

  containerController?: ContainerController;
}

interface IState {
    entities?: EntityTraits[],
}

export class Search extends React.Component<IProps, IState> {
  private entityQuery?: ExpandableQuery;

  constructor(props: IProps) {
    super(props);

    this.updateContainerTitle(props);
    this.state = {};
  }

  componentDidUpdate(prevProps: IProps): void {
    if (prevProps.query != this.props.query) {
      this.updateContainerTitle(this.props);
      this.query(this.props.query);
      this.setState({
        entities: null,
      });
    }
  }

  componentDidMount(): void {
    this.query(this.props.query);
  }

  componentWillUnmount(): void {
    this.entityQuery?.free();
  }

  render(): React.ReactNode {
    if (this.entityQuery?.hasResults ?? false) {
      return (
        <div className="search">
          <EntityList
            entities={this.state.entities}

            onRequireLoadMore={this.handleLoadMore.bind(this)}

            droppable={false}
            draggable={false}

            selection={this.props.selection}
            onSelectionChange={this.props.onSelectionChange}

            actionsForEntity={this.actionsForEntity.bind(this)}
          />;
        </div>
      );
    } else {
      return <Message text="Loading..." showAfterMs={200} />;
    }
  }

  private query(query: string): void {
    this.entityQuery?.free();

    const childrenQuery = QueryBuilder
      .matches(query)
      .count(30)
      .project(
        new exocore.store.Projection({
          fieldGroupIds: [1],
          package: ["exomind.base"],
        }),
        new exocore.store.Projection({
          skip: true,
        })
      )
      .build();
    this.entityQuery = new ExpandableQuery(childrenQuery, () => {
      const entities = Array.from(this.entityQuery.results()).map((res) => {
          return new EntityTraits(res.entity);
      });

      this.setState({ entities });
    })
  }

  private handleLoadMore(): void {
    this.entityQuery?.expand();
  }

  private actionsForEntity(et: EntityTraits): EntityActions {
    return new EntityActions([
      new ButtonAction('folder-open-o', this.handleEntityMoveCollection.bind(this, et)),
      new ButtonAction('inbox', this.handleEntityMoveInbox.bind(this, et))
    ]);
  }

  private handleEntityMoveCollection(et: EntityTraits) {
    ModalStore.showModal(() => {
      return <CollectionSelector entity={et} />;
    });
  }

  private handleEntityMoveInbox(et: EntityTraits): void {
    const mutation = MutationBuilder
      .updateEntity(et.id)
      .putTrait(new exomind.base.CollectionChild({
        collection: new exocore.store.Reference({
          entityId: 'inbox',
        }),
        weight: new Date().getTime(),
      }), 'child_inbox')
      .returnEntities()
      .build();

    Exocore.store.mutate(mutation);
  }

  private updateContainerTitle(props: IProps): void {
    if (props.containerController) {
      props.containerController.title = `Search ${props.query}`;
      props.containerController.icon = { fa: 'search' };
    }
  }
}
