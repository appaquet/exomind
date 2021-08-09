import Foundation
import Exocore

class ManagedQuery {
    let query: Exocore_Store_EntityQuery

    private let onChange: () -> ();
    private let autoReconnect: Bool;

    private var isDirty: Bool = false
    private var queries: [Exocore_Store_EntityQuery] = []
    private var queryHandles: [QueryStreamHandle?] = []
    private var queryResults: [Exocore_Store_EntityResults] = []

    private var refreshRetry: Retry?

    private var inhibitTimer: Timer?
    private var inhibitEnabled = false

     var results: [Exocore_Store_EntityResult] = []

    init(query: Exocore_Store_EntityQuery, onChange: @escaping () -> (), autoReconnect: Bool = true) {
        self.query = query
        self.onChange = onChange

        self.autoReconnect = true
        self.pushQuery(query)

        if self.autoReconnect {
            NotificationCenter.default.addObserver(self, selector: #selector(appWillEnterForeground), name: UIApplication.willEnterForegroundNotification, object: nil)
        }
    }

    var count: Int {
        get {
            self.results.count
        }
    }

    var canExpand: Bool {
        get {
            guard let lastResult = self.queryResults.last,
                  let lastQuery = self.queries.last else {
                return false
            }

            if lastResult.entities.isEmpty || !lastResult.hasNextPage || lastResult.entities.count < lastQuery.paging.count {
                return false
            }

            return true
        }
    }

    func expand() {
        let lastQueryIndex = self.queryResults.count - 1
        guard let lastResult = self.queryResults.last,
              var lastQuery = self.queries.last else {
            return
        }

        if !self.canExpand {
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

    func ensureFresh() {
        if !self.isDirty {
            return
        }

        // TODO: Should only retry if Exocore connection is open

        if self.refreshRetry == nil {
            self.refreshRetry = Retry(minimumInterval: 2.0) { [weak self] in
                self?.requeryFailed()
            }
        }
        self.refreshRetry?.trigger()
    }

    func pushQuery(_ query: Exocore_Store_EntityQuery) {
        let queryIndex = self.queries.count

        self.queries.append(query)
        self.queryResults.append(Exocore_Store_EntityResults())
        self.queryHandles.append(execQuery(query: query, queryIndex: queryIndex))
    }

    func inhibitChanges(forDelay: TimeInterval = 2.0) {
        print("ManagedQuery > Inhibiting for \(forDelay)")
        self.inhibitEnabled = true
        inhibitTimer?.invalidate()
        inhibitTimer = Timer.scheduledTimer(withTimeInterval: forDelay, repeats: false, block: { [weak self] timer in
            print("ManagedQuery > Inhibit done")
            self?.inhibitEnabled = false
            self?.aggregateAndTrigger()
        })
    }

    private func requeryFailed() {
        for i in 0..<self.queries.count {
            if self.queryHandles[i] == nil {
                print("ManagedQuery > Refreshing query \(i)")
                self.queryHandles[i] = self.execQuery(query: self.queries[i], queryIndex: i)
            }
        }
    }

    @objc private func appWillEnterForeground() {
        self.ensureFresh()
    }

    private func execQuery(query: Exocore_Store_EntityQuery, queryIndex: Int) -> QueryStreamHandle {
        print("ManagedQuery> Executing new query \(queryIndex)")

        var handle: QueryStreamHandle?
        handle = ExocoreClient.store.watchedQuery(query: query) { [weak self] status, results in
            guard let this = self else {
                return
            }

            if handle !== this.queryHandles[queryIndex] {
                // if handle changed, query got replaced by another one and this isn't valid anymore
                return
            }

            if status == .error || status == .done {
                this.handleQueryError(queryIndex: queryIndex)
                return
            }

            if let results = results {
                this.handleQueryResults(queryIndex: queryIndex, results: results)
            }
        }

        return handle!
    }

    private func handleQueryResults(queryIndex: Int, results: Exocore_Store_EntityResults) {
        self.queryResults[queryIndex] = results
        self.aggregateAndTrigger()
    }

    private func handleQueryError(queryIndex: Int) {
        print("ManagedQuery> Query \(queryIndex) failed")
        self.queryHandles[queryIndex] = nil
        self.aggregateAndTrigger()

        if self.autoReconnect {
            self.ensureFresh()
        }
    }

    private func aggregateAndTrigger() {
        if self.inhibitEnabled {
            print("ManagedQuery > New results inhibited")
            return
        }

        self.isDirty = false

        var idx = 0
        self.results = self.queryResults.reduce(into: []) { (mergedResults, queryResults) in
            if self.queryHandles[idx] == nil {
                self.isDirty = true
            }

            for result in queryResults.entities {
                // they may be same results at boundaries
                if let last = mergedResults.last {
                    if result.entity.id == last.entity.id {
                        continue
                    }
                }

                mergedResults.append(result)
            }

            idx += 1
        }

        self.onChange()
    }

    deinit {
        NotificationCenter.default.removeObserver(self)
        print("ManagedQuery > Deinit")
    }
}
