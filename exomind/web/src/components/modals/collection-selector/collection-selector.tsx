import { Exocore, QueryBuilder, TraitQueryBuilder, WatchedQueryWrapper } from 'exocore';
import React, { KeyboardEvent } from 'react';
import { exomind } from '../../../protos';
import { EntityTraits } from '../../../utils/entities';
import { ManagedQuery } from '../../../stores/queries';
import Debouncer from '../../../utils/debouncer';
import { EntitySelector } from '../../interaction/entity-selector/entity-selector';
import { CancellableEvent } from '../../../utils/events';
import { getEntityParentRelation } from '../../../stores/collections';
import { Commands } from '../../../utils/commands';
import './collection-selector.less';

interface IProps {
    entities: EntityTraits | EntityTraits[];
}

interface IState {
    entities?: EntityTraits[];
    entityParentsIds?: string[],
    entityParents?: EntityTraits[],
    collectionEntities?: EntityTraits[],
    keywords: string;
    debouncedKeywords?: string;
}

export class CollectionSelector extends React.Component<IProps, IState> {
    private searchDebouncer: Debouncer;

    private entityQuery: WatchedQueryWrapper;
    private entityParentsQuery: WatchedQueryWrapper;
    private collectionsQuery?: ManagedQuery;
    private filterInputRef: React.RefObject<HTMLInputElement> = React.createRef();

    constructor(props: IProps) {
        super(props);

        this.searchDebouncer = new Debouncer(200);
        this.state = {
            keywords: '',
        };
    }

    componentDidMount(): void {
        this.queryEntities(this.props);
        this.queryCollections();
        this.filterInputRef.current.focus();
    }

    componentWillUnmount(): void {
        this.entityQuery.free();
        this.collectionsQuery?.free();
        this.entityParentsQuery?.free();
    }

    componentDidUpdate(prevProps: Readonly<IProps>, prevState: Readonly<IState>): void {
        if (this.state.debouncedKeywords !== prevState.debouncedKeywords || this.state.entityParentsIds !== prevState.entityParentsIds) {
            this.queryCollections();
        }
    }

    render(): React.ReactNode {
        const loading = this.state.entities === undefined || this.state.entityParents === undefined || this.state.collectionEntities === undefined;
        const entities: EntityTraits[] = [];
        const entityIds = new Set<string>();
        if (!loading) {
            for (const entity of this.state.entityParents ?? []) {
                if (entityIds.has(entity.entity.id)) {
                    continue;
                }
                entities.push(entity);
                entityIds.add(entity.entity.id);
            }

            for (const entity of this.state.collectionEntities ?? []) {
                if (entityIds.has(entity.entity.id)) {
                    continue;
                }
                entities.push(entity);
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
                    onBlur={this.handleSelectorBlur}
                />
            </div>
        );
    }

    private queryEntities(props: IProps) {
        let entities = props.entities;
        if (!Array.isArray(entities)) {
            entities = [entities];
        }

        const entityIds = new Set(entities.map((entity) => entity.id));
        const entityQuery = QueryBuilder.withIds(Array.from(entityIds)).build();
        this.entityQuery = Exocore.store
            .watchedQuery(entityQuery)
            .onChange((res) => {
                const entities = res.entities.map((r) => new EntityTraits(r.entity));
                const entityParentsIds = this.entityParentsIntersection(entities);
                this.setState({ entities, entityParentsIds, });
            });
    }

    private entityParentsIntersection(entities: EntityTraits[]): string[] {
        const parentCount: { [key: string]: number } = {};

        for (const entity of entities) {
            const parentIds = entity
                .traitsOfType<exomind.base.v1.ICollectionChild>(exomind.base.v1.CollectionChild)
                .flatMap((cc) => cc.message.collection.entityId);
            for (const parentId of parentIds) {
                if (parentCount[parentId] === undefined) {
                    parentCount[parentId] = 0;
                }
                parentCount[parentId]++;
            }
        }

        return Object.keys(parentCount).filter((id) => parentCount[id] === entities.length);
    }

    private queryCollections(): void {
        // Get collections
        const traitQuery = (this.state.debouncedKeywords) ? TraitQueryBuilder.matches(this.state.debouncedKeywords).build() : null;
        const collectionQuery = QueryBuilder
            .withTrait(exomind.base.v1.Collection, traitQuery)
            .count(30)
            .build();
        this.collectionsQuery?.free();
        this.collectionsQuery = new ManagedQuery(collectionQuery, () => {
            this.setState({
                collectionEntities: Array.from(this.collectionsQuery.results()).map((r) => new EntityTraits(r.entity)),
            });
        });

        // Get entities parents
        if (this.state.entityParentsIds) {
            const parentQuery = QueryBuilder
                .withIds(this.state.entityParentsIds)
                .count(this.state.entityParentsIds.length)
                .build();
            this.entityParentsQuery?.free();
            this.entityParentsQuery = Exocore.store.watchedQuery(parentQuery);
            this.entityParentsQuery.onChange((res) => {
                this.setState({
                    entityParents: res.entities.map((r) => new EntityTraits(r.entity)),
                });
            });
        }
    }

    private handleFilterChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
        this.setState({
            keywords: event.target.value
        });

        this.searchDebouncer.debounce(() => {
            this.setState({
                debouncedKeywords: this.state.keywords
            });
        });
    };

    private handleFilterKeyDown = (event: KeyboardEvent): void => {
        if (event.key == 'ArrowUp' || event.key == 'ArrowDown') {
            this.filterInputRef.current?.blur();
        }
    };

    private handleItemCheck = (collectionEntity: EntityTraits, event?: CancellableEvent): void => {
        const parentRel = getEntityParentRelation(this.state.entities[0], collectionEntity.id); // assumes all entities are the same
        if (!parentRel) {
            Commands.addToParent(this.state.entities, collectionEntity.id);
        } else {
            Commands.removeFromParent(this.state.entities, collectionEntity.id);
        }

        event?.stopPropagation(); // since we are bound on click of the li too, we stop propagation to prevent double
    };

    private handleLoadMore = (): void => {
        this.collectionsQuery?.expand();
    };

    private handleSelectorBlur = (): void => {
        this.filterInputRef.current?.focus();
    };
}

