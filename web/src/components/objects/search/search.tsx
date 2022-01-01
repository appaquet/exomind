import { Exocore, exocore, MutationBuilder, QueryBuilder } from 'exocore';
import React from 'react';
import { exomind } from '../../../protos';
import { EntityTraits } from '../../../utils/entities';
import { ExpandableQuery } from '../../../stores/queries';
import { CollectionSelector } from '../../modals/collection-selector/collection-selector';
import { ContainerState } from '../container-state';
import { ButtonAction, EntityActions } from '../entity-list/entity-action';
import { EntityList } from '../entity-list/entity-list';
import { Selection } from '../entity-list/selection';
import './search.less';
import { runInAction } from 'mobx';
import { IStores, StoresContext } from '../../../stores/stores';

interface IProps {
  query: string;

  selection?: Selection;
  onSelectionChange?: (sel: Selection) => void;

  containerState?: ContainerState;
}

interface IState {
  entities: EntityTraits[],
}

export class Search extends React.Component<IProps, IState> {
  static contextType = StoresContext;
  declare context: IStores;

  private entityQuery?: ExpandableQuery;

  constructor(props: IProps) {
    super(props);

    this.updateContainerTitle(props);
    this.state = {
      entities: [],
    };
  }

  componentDidUpdate(prevProps: IProps): void {
    if (prevProps.query != this.props.query) {
      this.updateContainerTitle(this.props);
      this.query(this.props.query);
    }
  }

  componentDidMount(): void {
    this.query(this.props.query);
  }

  componentWillUnmount(): void {
    this.entityQuery?.free();
  }

  render(): React.ReactNode {
    return (
      <div className="search">
        <EntityList
          entities={this.state.entities}

          onRequireLoadMore={this.handleLoadMore}

          droppable={false}
          draggable={false}

          selection={this.props.selection}
          onSelectionChange={this.props.onSelectionChange}

          actionsForEntity={this.actionsForEntity}
          containerState={this.props.containerState}
        />;
      </div>
    );
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

  private updateContainerTitle(props: IProps): void {
    if (props.containerState) {
      runInAction(() => {
        props.containerState.title = `Search '${props.query}'`;
        props.containerState.icon = { fa: 'search' };
      });
    }
  }

  private handleLoadMore = (): void => {
    this.entityQuery?.expand();
  }

  private actionsForEntity = (et: EntityTraits): EntityActions => {
    return new EntityActions([
      new ButtonAction('folder-open-o', () => this.handleEntityMoveCollection(et)),
      new ButtonAction('inbox', () => this.handleEntityMoveInbox(et))
    ]);
  }

  private handleEntityMoveCollection(et: EntityTraits) {
    this.context.session.showModal(() => {
      return <CollectionSelector entity={et} />;
    });
  }

  private handleEntityMoveInbox(et: EntityTraits): void {
    const mutation = MutationBuilder
      .updateEntity(et.id)
      .putTrait(new exomind.base.v1.CollectionChild({
        collection: new exocore.store.Reference({
          entityId: 'inbox',
        }),
        weight: new Date().getTime(),
      }), 'child_inbox')
      .returnEntities()
      .build();

    Exocore.store.mutate(mutation);
  }
}
