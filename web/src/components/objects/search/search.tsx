import { Exocore, exocore, MutationBuilder, QueryBuilder } from 'exocore';
import React from 'react';
import { exomind } from '../../../protos';
import { EntityTraits } from '../../../store/entities';
import { ModalStore } from '../../../store/modal-store';
import { ExpandableQuery } from '../../../store/queries';
import { CollectionSelector } from '../../popups/collection-selector/collection-selector';
import { ContainerController } from '../container-controller';
import EntityAction from '../entity-list/entity-action';
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

export class Search extends React.Component<IProps> {
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
        results: null,
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
      const entities = Array.from(this.entityQuery.results()).map((entityResult) => {
        return entityResult.entity;
      });

      return (
        <div className="search">
          <EntityList
            entities={entities}

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
        new exocore.index.Projection({
          fieldGroupIds: [1],
          package: ["exomind.base"],
        }),
        new exocore.index.Projection({
          skip: true,
        })
      )
      .build();
    this.entityQuery = new ExpandableQuery(childrenQuery, () => {
      this.setState({});
    })
  }

  private handleLoadMore(): void {
    this.entityQuery?.expand();
  }

  private actionsForEntity(et: EntityTraits): EntityAction[] {
    return [
      new EntityAction('folder-open-o', this.handleEntityMoveCollection.bind(this, et)),
      new EntityAction('inbox', this.handleEntityMoveInbox.bind(this, et))
    ];
  }

  private handleEntityMoveCollection(et: EntityTraits) {
    ModalStore.showModal(this.showCollectionsSelector.bind(this, et));
  }

  private showCollectionsSelector(et: EntityTraits) {
    return <CollectionSelector entity={et.entity} />;
  }

  private handleEntityMoveInbox(et: EntityTraits): void {
    const mutation = MutationBuilder
      .updateEntity(et.id)
      .putTrait(new exomind.base.CollectionChild({
        collection: new exocore.index.Reference({
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
      props.containerController.icon = 'search';
    }
  }
}
