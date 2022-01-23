
import { Exocore, exocore, WatchedQueryWrapper } from 'exocore';

export class ManagedQuery {
    query: exocore.store.IEntityQuery;
    onChange: () => void;
    hasResults: boolean;

    private queries: exocore.store.IEntityQuery[] = [];
    private watched_queries: WatchedQueryWrapper[] = [];
    private queries_results: exocore.store.IEntityResults[] = [];
    private freed = false;

    constructor(query: exocore.store.IEntityQuery, onChange: () => void) {
        this.query = query;
        this.onChange = onChange;

        this.pushQuery(query);
    }

    *results(): IterableIterator<exocore.store.IEntityResult> {
        let lastEntity = null;

        for (const query_results of this.queries_results) {
            if (!query_results) {
                continue;
            }

            for (const entity of query_results.entities) {
                // they may be same results at boundaries
                if (lastEntity?.entity.id == entity.entity.id) {
                    continue;
                }

                lastEntity = entity;
                yield entity;
            }
        }
    }

    pushQuery(query: exocore.store.IEntityQuery): void {
        const queryIndex = this.watched_queries.length;

        const watchedQuery = Exocore.store.watchedQuery(query);
        watchedQuery.onChange((results: exocore.store.EntityResults) => {
            this.queries_results[queryIndex] = results;
            this.hasResults = true;
            this.triggerChanged();
        });

        this.queries.push(query);
        this.watched_queries.push(watchedQuery);
        this.queries_results.push(null);
    }

    expand(): void {
        const lastIndex = this.queries_results.length - 1;
        const lastResults = this.queries_results[lastIndex];
        const lastQueryRequestCount = this.queries[lastIndex].paging?.count ?? 0;
        if (!lastResults || !lastResults.nextPage || lastResults.entities.length < lastQueryRequestCount) {
            return;
        }

        // push a new query for next page
        const nextQuery = new exocore.store.EntityQuery(this.query);
        nextQuery.paging = lastResults.nextPage;
        this.pushQuery(nextQuery);

        // replace last query to fix its page boundary so that it doesn't return
        // results that would be in next page and max out count so that any new
        // results within it are included
        this.watched_queries[lastIndex].free();
        const query = this.queries[lastIndex];
        if (nextQuery.paging.afterOrderingValue) {
            query.paging.beforeOrderingValue = nextQuery.paging.afterOrderingValue;
        } else {
            query.paging.afterOrderingValue = nextQuery.paging.beforeOrderingValue;
        }
        query.paging.count = 1000;

        const watchedQuery = Exocore.store.watchedQuery(query);
        watchedQuery.onChange((results: exocore.store.EntityResults) => {
            this.queries_results[lastIndex] = results;
            this.hasResults = true;
            this.triggerChanged();
        });
        this.watched_queries[lastIndex] = watchedQuery;
    }

    triggerChanged(): void {
        if (!this.freed) {
            this.onChange();
        }
    }

    free(): void {
        for (const query of this.watched_queries) {
            query.free();
        }
        this.freed = true;
    }
}