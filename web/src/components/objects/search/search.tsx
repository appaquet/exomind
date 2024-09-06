import { exocore, QueryBuilder } from 'exocore';
import React from 'react';
import { EntityTraits } from '../../../utils/entities';
import { ManagedQuery } from '../../../stores/queries';
import { ContainerState } from '../container-state';
import { ListEntityActions } from '../entity-list/actions';
import { EntityList } from '../entity-list/entity-list';
import { Selection } from '../entity-list/selection';
import { runInAction } from 'mobx';
import { IStores, StoresContext } from '../../../stores/stores';
import { Actions } from '../../../utils/actions';
import './search.less';
import { BottomMenu, IActionShortcut } from '../../interaction/bottom-menu/bottom-menu';

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

  private entityQuery?: ManagedQuery;

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

          onLoadMore={this.handleLoadMore}

          droppable={false}
          draggable={false}

          selection={this.props.selection}
          onSelectionChange={this.props.onSelectionChange}

          actionsForEntity={this.actionsForEntity}
          containerState={this.props.containerState}
        />;

        {this.renderBottomMenu()}
      </div>
    );
  }

  private renderBottomMenu(): React.ReactNode {
    if (!this.props.selection || this.props.selection.isEmpty || !(this.props.containerState.active ?? false)) {
      return null;
    }

    const selectedEntities = this.props.selection.filterSelectedEntities(this.state.entities);
    const items = Actions.forSelectedEntities(selectedEntities, { section: 'search' });
    const actionShortcuts: IActionShortcut[] = [
      {
        shortcutKey: 'z',
        disabledContexts: ['input', 'modal'],
        actionKey: 'snooze',
      },
      {
        shortcutKey: 'c',
        disabledContexts: ['input', 'modal'],
        actionKey: 'select-entity-collections',
      },
    ];

    return (
      <BottomMenu
        items={items}
        shortcuts={actionShortcuts}
      />
    );
  }

  private query(query: string): void {
    this.entityQuery?.free();

    const childrenQuery = QueryBuilder
      .fromQueryString(query)
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

    this.entityQuery = new ManagedQuery(childrenQuery, () => {
      const entities = Array.from(this.entityQuery.results()).map((res) => {
        return new EntityTraits(res.entity);
      });

      this.setState({ entities });
    });
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
  };

  private actionsForEntity = (et: EntityTraits): ListEntityActions => {
    const actions = Actions.forEntity(et, { section: 'search' });
    return ListEntityActions.fromActions(actions);
  };
}
