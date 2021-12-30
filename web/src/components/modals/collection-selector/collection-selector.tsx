import { Exocore, exocore, MutationBuilder, QueryBuilder, TraitQueryBuilder, WatchedQueryWrapper } from 'exocore';
import { memoize } from 'lodash';
import React, { KeyboardEvent } from 'react';
import { exomind } from '../../../protos';
import { EntityTraits } from '../../../utils/entities';
import { ExpandableQuery } from '../../../stores/queries';
import Debouncer from '../../../utils/debouncer';
import { EntitySelector } from '../../interaction/entity-selector/entity-selector';
import { CancellableEvent } from '../../../utils/events';

import './collection-selector.less';

interface IProps {
    entity: EntityTraits;
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
    private filterInputRef: React.RefObject<HTMLInputElement> = React.createRef();

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

        const loading = !(this.collectionsQuery?.hasResults ?? false) || !this.state.entity;

        const entities: EntityTraits[] = [];
        const entityIds = new Set<string>();
        if (!loading) {
            for (const entity of this.state.entityParents ?? []) {
                if (entityIds.has(entity.entity.id)) {
                    continue;
                }
                entities.push(this.wrapEntityTraits(entity.entity));
                entityIds.add(entity.entity.id);
            }

            for (const entity of this.collectionsQuery?.results() ?? []) {
                if (entityIds.has(entity.entity.id)) {
                    continue;
                }
                entities.push(this.wrapEntityTraits(entity.entity));
                entityIds.add(entity.entity.id);
            }
        }
        const selectedIds: string[] = this.state.entityParentsIds;

        return (
            <div className="collection-selector">
                <div className="collection-selector-header">Add to collections...</div>
                <div className="filter">
                    <input type="text"
                        ref={this.filterInputRef}
                        value={this.state.keywords}
                        onChange={this.handleFilterChange}
                        onKeyDown={this.handleFilterKeyDown}
                        placeholder="Filter..." />
                </div>

                <EntitySelector
                    multi={true}
                    entities={entities}
                    selectedIds={selectedIds}
                    loading={loading}
                    onSelect={this.handleItemCheck}
                    onUnselect={this.handleItemCheck}
                    onNeedMore={this.handleLoadMore}
                    onHoverUnderflow={this.handleHoverUnderflow}
                />
            </div>
        );
    }

    private wrapEntityTraits = memoize((entity: exocore.store.IEntity) => new EntityTraits(entity));

    private entityParents(entity: EntityTraits): string[] {
        return entity
            .traitsOfType<exomind.base.v1.ICollectionChild>(exomind.base.v1.CollectionChild)
            .flatMap((cc) => cc.message.collection.entityId);
    }

    private maybeRefreshQueries(): void {
        // Get collections
        if (this.collectionsQueryKeywords != this.state.debouncedKeywords || !this.collectionsQuery) {
            this.collectionsQuery?.free();

            const traitQuery = (this.state.debouncedKeywords) ? TraitQueryBuilder.matches(this.state.debouncedKeywords).build() : null;
            const query = QueryBuilder
                .withTrait(exomind.base.v1.Collection, traitQuery)
                .count(30)
                .build();
            this.collectionsQuery = new ExpandableQuery(query, () => {
                this.setState({});
            })
            this.collectionsQueryKeywords = this.state.debouncedKeywords;
        }

        // Get entity current parents
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

    private handleFilterChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
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

    private handleFilterKeyDown = (event: KeyboardEvent): void => {
        if (event.key == 'ArrowUp' || event.key == 'ArrowDown') {
            this.filterInputRef.current?.blur();
        }
    }

    private handleItemCheck = (collectionEntity: EntityTraits, event?: CancellableEvent): void => {
        const currentChildTrait = this.state.entity
            .traitsOfType<exomind.base.v1.ICollectionChild>(exomind.base.v1.CollectionChild)
            .find((c) => c.message.collection.entityId == collectionEntity.id);

        if (!currentChildTrait) {
            const mutation = MutationBuilder
                .updateEntity(this.state.entity.id)
                .putTrait(new exomind.base.v1.CollectionChild({
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

        event?.stopPropagation(); // since we are bound on click of the li too, we stop propagation to prevent double
    }

    private handleLoadMore = (): void => {
        this.collectionsQuery?.expand();
    }

    private handleHoverUnderflow = (): void => {
        this.filterInputRef.current?.focus();
    }
}

