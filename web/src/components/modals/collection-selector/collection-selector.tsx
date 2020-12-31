import classNames from 'classnames';
import { Exocore, exocore, MutationBuilder, QueryBuilder, TraitQueryBuilder, WatchedQueryWrapper } from 'exocore';
import * as _ from 'lodash';
import React from 'react';
import { exomind } from '../../../protos';
import { EntityTraits } from '../../../store/entities';
import { ExpandableQuery } from '../../../store/queries';
import Debouncer from '../../../utils/debouncer';
import Scrollable from '../../interaction/scrollable/scrollable';
import { Message } from '../../objects/message';
import './collection-selector.less';

interface IProps {
    entity: exocore.store.IEntity;
}

interface IState {
    entity?: EntityTraits;
    entityParentsIds?: string[],
    entityParents?: exocore.store.IEntityResult[],
    keywords: string;
    debouncedKeywords?: string;
}

export class CollectionSelector extends React.Component<IProps, IState> {
    private searchDebouncer: Debouncer;

    private entityQuery: WatchedQueryWrapper;
    private entityParentsQuery: WatchedQueryWrapper;
    private entityParentsQueryIds?: string[];
    private collectionsQuery?: ExpandableQuery;
    private collectionsQueryKeywords?: string;
    private filterInputRef: React.RefObject<HTMLInputElement>;

    constructor(props: IProps) {
        super(props);

        this.searchDebouncer = new Debouncer(200);

        const entityQuery = QueryBuilder.withIds(props.entity.id).build();
        this.entityQuery = Exocore.store
            .watchedQuery(entityQuery)
            .onChange((res) => {
                const entity = new EntityTraits(res.entities[0].entity);
                this.setState({
                    entity: entity,
                    entityParentsIds: this.entityParents(entity),
                })
            });

        this.state = {
            keywords: '',
        };

        this.filterInputRef = React.createRef();
    }

    componentWillUnmount(): void {
        this.entityQuery.free();
        this.collectionsQuery?.free();
        this.entityParentsQuery?.free();
    }

    componentDidMount(): void {
        this.maybeRefreshQueries();
        this.filterInputRef.current.focus();
    }

    render(): React.ReactNode {
        this.maybeRefreshQueries();

        return (
            <div className="collection-selector" onMouseOver={this.handlePreventDefault.bind(this)}>
                <div className="collection-selector-header">Add to collections...</div>
                <div className="filter">
                    <input type="text" ref={this.filterInputRef} value={this.state.keywords}
                        onChange={this.handleFilterChange.bind(this)} placeholder="Filter..." />
                </div>

                {this.renderInner()}
            </div>
        );
    }

    private renderInner(): React.ReactNode {
        if (!(this.collectionsQuery?.hasResults ?? false) || !this.state.entity) {
            return <Message text="Loading..." showAfterMs={200} />
        }

        const collectionsResults = Array
            .from(this.state.entityParents ?? [])
            .concat(Array.from(this.collectionsQuery?.results()));

        return (
            <Scrollable loadMoreItems={15} onNeedMore={this.handleLoadMore.bind(this)} nbItems={collectionsResults.length}>
                <ul onClick={this.handlePreventDefault.bind(this)}>
                    {this.renderCollections(collectionsResults)}
                </ul>
            </Scrollable>
        );
    }

    private renderCollections(collectionResults: exocore.store.IEntityResult[]): React.ReactNode {
        if (collectionResults.length > 0) {
            const parentsIds = _.keyBy(this.state.entityParentsIds ?? [])
            return _.chain(collectionResults)
                .uniqBy(col => col.entity.id)
                .map((colResult) => {
                    const et = new EntityTraits(colResult.entity);
                    const colTrait = et.traitOfType<exomind.base.ICollection>(exomind.base.Collection);

                    const iconClasses = classNames({
                        'icon': true,
                        'fa': true,
                        ['fa-' + colTrait.constants.icon]: true
                    });

                    const checked = _.includes(parentsIds, et.id);
                    return <li key={colResult.entity.id} onClick={this.handleItemCheck.bind(this, et, colResult)}>
                        <input type="checkbox" checked={checked} onChange={this.handleItemCheck.bind(this, et, colResult)} />
                        <span className={iconClasses} />
                        {colTrait.displayName}
                    </li>
                })
                .value();
        }
    }

    private entityParents(entity: EntityTraits): string[] {
        return entity
            .traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild)
            .flatMap((cc) => cc.message.collection.entityId);
    }

    private maybeRefreshQueries(): void {
        // Get collections
        if (this.collectionsQueryKeywords != this.state.debouncedKeywords || !this.collectionsQuery) {
            this.collectionsQuery?.free();

            const traitQuery = (this.state.debouncedKeywords) ? TraitQueryBuilder.matches(this.state.debouncedKeywords).build() : null;
            const query = QueryBuilder
                .withTrait(exomind.base.Collection, traitQuery)
                .count(30)
                .build();
            this.collectionsQuery = new ExpandableQuery(query, () => {
                this.setState({});
            })
            this.collectionsQueryKeywords = this.state.debouncedKeywords;
        }

        // Get entity current parrents
        if (this.state.entityParentsIds && this.entityParentsQueryIds != this.state.entityParentsIds) {
            this.entityParentsQuery?.free();

            const query = QueryBuilder
                .withIds(this.state.entityParentsIds)
                .count(this.state.entityParentsIds.length)
                .build();
            this.entityParentsQuery = Exocore.store.watchedQuery(query);
            this.entityParentsQuery.onChange((res) => {
                this.setState({
                    entityParents: res.entities,
                });
            });
            this.entityParentsQueryIds = this.state.entityParentsIds;
        }
    }

    private handlePreventDefault(e: MouseEvent): void {
        // prevent browser from bubbling the click and mouseover under the list
        e.stopPropagation();
    }

    private handleFilterChange(event: React.ChangeEvent<HTMLInputElement>): void {
        const value = event.target.value;
        this.setState({
            keywords: value
        });

        this.searchDebouncer.debounce(() => {
            this.setState({
                debouncedKeywords: value
            });
        });
    }

    private handleItemCheck(collectionEntity: EntityTraits, collection: exomind.base.ICollection, event: MouseEvent): void {
        const currentChildTrait = this.state.entity
            .traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild)
            .find((c) => c.message.collection.entityId == collectionEntity.id);

        if (!currentChildTrait) {
            const mutation = MutationBuilder
                .updateEntity(this.state.entity.id)
                .putTrait(new exomind.base.CollectionChild({
                    collection: new exocore.store.Reference({
                        entityId: collectionEntity.id,
                    }),
                    weight: new Date().getTime(),
                }), `child_${collectionEntity.id}`)
                .build();
            Exocore.store.mutate(mutation);

        } else {
            const mutation = MutationBuilder
                .updateEntity(this.state.entity.id)
                .deleteTrait(currentChildTrait.id)
                .build();
            Exocore.store.mutate(mutation);
        }

        event.stopPropagation(); // since we are bound on click of the li too, we stop propagation to prevent double
    }

    private handleLoadMore(): void {
        this.collectionsQuery?.expand();
    }
}

