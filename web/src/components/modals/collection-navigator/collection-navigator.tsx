import { exocore, QueryBuilder, TraitQueryBuilder } from 'exocore';
import { memoize } from 'lodash';
import React, { KeyboardEvent } from 'react';
import { exomind } from '../../../protos';
import { EntityTraits } from '../../../utils/entities';
import { ManagedQuery } from '../../../stores/queries';
import Debouncer from '../../../utils/debouncer';
import { EntitySelector } from '../../interaction/entity-selector/entity-selector';
import { CancellableEvent } from '../../../utils/events';

import './collection-navigator.less';

interface IProps {
    onSelect: (entity: EntityTraits) => void;
}

interface IState {
    keywords: string;
    debouncedKeywords?: string;
}

export class CollectionNavigator extends React.Component<IProps, IState> {
    private searchDebouncer: Debouncer;

    private collectionsQuery?: ManagedQuery;
    private collectionsQueryKeywords?: string;
    private filterInputRef: React.RefObject<HTMLInputElement> = React.createRef();

    constructor(props: IProps) {
        super(props);

        this.searchDebouncer = new Debouncer(200);
        this.state = {
            keywords: '',
        };

        this.maybeRefreshQueries();
    }

    componentWillUnmount(): void {
        this.collectionsQuery?.free();
    }

    componentDidMount(): void {
        this.filterInputRef.current.focus();
    }

    render(): React.ReactNode {
        const loading = !(this.collectionsQuery?.hasResults ?? false);
        const entities = Array.from(this.collectionsQuery?.results() ?? []).map((res) => this.wrapEntityTraits(res.entity));

        return (
            <div className="collection-selector">
                <div className="filter">
                    <input type="text"
                        ref={this.filterInputRef}
                        value={this.state.keywords}
                        onChange={this.handleFilterChange}
                        onKeyDown={this.handleFilterKeyDown}
                        placeholder="Filter..." />
                </div>

                <EntitySelector
                    multi={false}
                    entities={entities}
                    loading={loading}
                    onSelect={this.handleItemCheck}
                    onUnselect={this.handleItemCheck}
                    onNeedMore={this.handleLoadMore}
                    onBlur={this.handleSelectorBlur}
                />
            </div>
        );
    }

    private wrapEntityTraits = memoize((entity: exocore.store.IEntity) => new EntityTraits(entity));

    private maybeRefreshQueries(): void {
        if (this.collectionsQueryKeywords != this.state.debouncedKeywords || !this.collectionsQuery) {
            this.collectionsQuery?.free();

            const traitQuery = (this.state.debouncedKeywords) ? TraitQueryBuilder.matches(this.state.debouncedKeywords).build() : null;
            const query = QueryBuilder
                .withTrait(exomind.base.v1.Collection, traitQuery)
                .count(30)
                .build();
            this.collectionsQuery = new ManagedQuery(query, () => {
                this.setState({});
            });
            this.collectionsQueryKeywords = this.state.debouncedKeywords;
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
            this.maybeRefreshQueries();
        });
    };

    private handleFilterKeyDown = (event: KeyboardEvent): void => {
        if (event.key == 'ArrowUp' || event.key == 'ArrowDown') {
            this.filterInputRef.current?.blur();
        }
    };

    private handleItemCheck = (entity: EntityTraits, event?: CancellableEvent): void => {
        this.props.onSelect(entity);
        event?.stopPropagation(); // since we are bound on click of the li too, we stop propagation to prevent double
    };

    private handleLoadMore = (): void => {
        this.collectionsQuery?.expand();
    };

    private handleSelectorBlur = (): void => {
        this.filterInputRef.current?.focus();
    };
}

