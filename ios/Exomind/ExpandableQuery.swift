import Foundation
import Exocore

class ExpandableQuery {
    private let query: Exocore_Index_EntityQuery
    private let onChange: () -> ();

    private var queries: [Exocore_Index_EntityQuery] = []
    private var queryHandles: [QueryStreamHandle] = []
    private var queryResults: [Exocore_Index_EntityResults] = []

    var results: [Exocore_Index_EntityResult] = [];

    var count: Int {
        get {
            self.results.count
        }
    }

    init(query: Exocore_Index_EntityQuery, onChange: @escaping () -> ()) {
        self.query = query
        self.onChange = onChange

        self.pushQuery(query)
    }

    func expand() {
        let lastQueryIndex = self.queryResults.count - 1
        guard let lastResult = self.queryResults.last,
              var lastQuery = self.queries.last else {
            return
        }

        if lastResult.entities.isEmpty || !lastResult.hasNextPage || lastResult.entities.count < lastQuery.paging.count {
            return
        }

        // push a new query for next page
        var nextQuery = self.query
        nextQuery.paging = lastResult.nextPage
        self.pushQuery(nextQuery)

        // replace last query to fix its page boundary so that it doesn't return
        // results that would be in next page and max out count so that any new
        // results within it are included
        if lastQuery.paging.hasAfterOrderingValue {
            lastQuery.paging.beforeOrderingValue = nextQuery.paging.afterOrderingValue
        } else {
            lastQuery.paging.afterOrderingValue = nextQuery.paging.beforeOrderingValue
        }
        lastQuery.paging.count = 1000

        self.queryHandles[lastQueryIndex] = self.execQuery(query: lastQuery, queryIndex: lastQueryIndex)
    }

    func pushQuery(_ query: Exocore_Index_EntityQuery) {
        let queryIndex = self.queries.count

        self.queries.append(query)
        self.queryResults.append(Exocore_Index_EntityResults())
        self.queryHandles.append(execQuery(query: query, queryIndex: queryIndex))
    }

    private func execQuery(query: Exocore_Index_EntityQuery, queryIndex: Int) -> QueryStreamHandle {
        let handle = ExocoreClient.store.watchedQuery(query: query) { [weak self] status, results in
            guard let this = self,
                  let results = results else {
                return
            }

            this.handleQueryResults(queryIndex: queryIndex, results: results)
        }
        return handle
    }

    private func handleQueryResults(queryIndex: Int, results: Exocore_Index_EntityResults) {
        self.queryResults[queryIndex] = results

        self.results = self.queryResults.reduce(into: []) { (mergedResults, queryResults) in
            for result in queryResults.entities {
                // they may be same results at boundaries
                if let last = mergedResults.last {
                    if result.entity.id == last.entity.id {
                        continue
                    }
                }

                mergedResults.append(result)
            }
        }

        self.onChange()
    }
}
