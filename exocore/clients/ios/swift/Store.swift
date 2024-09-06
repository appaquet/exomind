import Foundation

public class Store {
    weak var client: ClientInstance?

    init(client: ClientInstance) {
        self.client = client
    }

    public func mutate(mutation: Exocore_Store_MutationRequest, onCompletion: ((MutationStatus, Exocore_Store_MutationResult?) -> Void)? = nil) {
        let callback = onCompletion ?? { status, result in }
        let cbCtx = MutationCallbackContext(cb: callback)
        let cbCtxPtr = UnsafeRawPointer(Unmanaged.passRetained(cbCtx).toOpaque())

        let mutationData = try! mutation.serializedData()
        let _ = mutationData.withUnsafeBytes { (dataPtr) -> ExocoreMutationHandle in
            let dataAddr = dataPtr.bindMemory(to: UInt8.self).baseAddress

            return exocore_store_mutate(self.client!.client, dataAddr, UInt(mutationData.count), { (status, resultsPtr, resultsSize, cbCtxPtr) in
                let cbCtx = Unmanaged<MutationCallbackContext>.fromOpaque(cbCtxPtr!).takeRetainedValue() // consume ptr

                if status == UInt8(ExocoreMutationStatus_Error.rawValue) {
                    cbCtx.cb(.error, nil)
                } else {
                    let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
                    if let results = try? Exocore_Store_MutationResult(serializedData: resultsData) {
                        cbCtx.cb(.done, results)
                    } else {
                        cbCtx.cb(.error, nil)
                    }
                }
            }, cbCtxPtr)
        }
    }

    public func query(query: Exocore_Store_EntityQuery, onChange: @escaping (QueryStatus, Exocore_Store_EntityResults?) -> Void) -> QueryHandle {
        let cbCtx = QueryCallbackContext(cb: onChange)
        let cbCtxPtr = UnsafeRawPointer(Unmanaged.passRetained(cbCtx).toOpaque())

        let queryData = try! query.serializedData()
        let handle = queryData.withUnsafeBytes { (dataPtr) -> ExocoreQueryHandle in
            let dataAddr = dataPtr.bindMemory(to: UInt8.self).baseAddress

            return exocore_store_query(self.client!.client, dataAddr, UInt(queryData.count), { (status, resultsPtr, resultsSize, cbCtxPtr) in
                let cbCtx = Unmanaged<QueryCallbackContext>.fromOpaque(cbCtxPtr!).takeRetainedValue() // consume ptr

                if status == UInt8(ExocoreQueryStatus_Error.rawValue) {
                    cbCtx.cb(.error, nil)
                } else {
                    let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
                    if let results = try? Exocore_Store_EntityResults(serializedData: resultsData) {
                        cbCtx.cb(.done, results)
                    } else {
                        cbCtx.cb(.error, nil)
                    }
                }
            }, cbCtxPtr)
        }

        return QueryHandle(queryHandle: handle, client: self.client!)
    }

    public func watchedQuery(query: Exocore_Store_EntityQuery, onChange: @escaping (QueryStatus, Exocore_Store_EntityResults?) -> Void) -> QueryStreamHandle {
        let cbCtx = QueryCallbackContext(cb: onChange)
        let cbCtxPtr = UnsafeRawPointer(Unmanaged.passRetained(cbCtx).toOpaque())

        let queryData = try! query.serializedData()
        let handle = queryData.withUnsafeBytes { (dataPtr) -> ExocoreWatchedQueryHandle in
            let addr = dataPtr.bindMemory(to: UInt8.self).baseAddress

            return exocore_store_watched_query(self.client!.client, addr, UInt(queryData.count), { (status, resultsPtr, resultsSize, cbCtxPtr) in
                if status == UInt8(ExocoreWatchedQueryStatus_Done.rawValue) {
                    let cbCtxPtr = Unmanaged<QueryCallbackContext>.fromOpaque(cbCtxPtr!).takeRetainedValue() // consume ptr
                    cbCtxPtr.cb(.done, nil)
                    return
                } else if status == UInt8(ExocoreWatchedQueryStatus_Error.rawValue) {
                    let cbCtxPtr = Unmanaged<QueryCallbackContext>.fromOpaque(cbCtxPtr!).takeRetainedValue() // consume ptr
                    cbCtxPtr.cb(.error, nil)
                    return
                }

                let cbCtxPtr = Unmanaged<QueryCallbackContext>.fromOpaque(cbCtxPtr!).takeUnretainedValue() // don't consume the ptr
                let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
                if let results = try? Exocore_Store_EntityResults(serializedData: resultsData) {
                    cbCtxPtr.cb(.running, results)
                } else {
                    cbCtxPtr.cb(.error, nil)
                }
            }, cbCtxPtr)
        }

        return QueryStreamHandle(queryHandle: handle, client: self.client!)
    }

    public func httpEndpoints() -> [String] {
        guard let context = self.client?.client else { return [] }

        let endpointsPtr = exocore_store_http_endpoints(context)
        let endpointsStr = String(cString: endpointsPtr!)
        exocore_free_string(endpointsPtr)

        return endpointsStr.split(separator: ";").map { String($0) }
    }
}

public enum QueryStatus {
    case running
    case done
    case error
}

public class QueryStreamHandle {
    var handle: ExocoreWatchedQueryHandle
    weak var client: ClientInstance?

    init(queryHandle: ExocoreWatchedQueryHandle, client: ClientInstance) {
        self.handle = queryHandle
        self.client = client
    }

    deinit {
        if let client = self.client {
            exocore_store_watched_query_cancel(client.client, self.handle)
        }
    }
}

public class QueryHandle {
    var handle: ExocoreQueryHandle
    weak var client: ClientInstance?

    init(queryHandle: ExocoreQueryHandle, client: ClientInstance) {
        self.handle = queryHandle
        self.client = client
    }

    deinit {
        if let client = self.client {
            exocore_store_query_cancel(client.client, self.handle)
        }
    }
}

class QueryCallbackContext {
    var cb: (QueryStatus, Exocore_Store_EntityResults?) -> Void

    init(cb: @escaping (QueryStatus, Exocore_Store_EntityResults?) -> Void) {
        self.cb = cb
    }
}

class MutationCallbackContext {
    var cb: (MutationStatus, Exocore_Store_MutationResult?) -> Void

    init(cb: @escaping (MutationStatus, Exocore_Store_MutationResult?) -> Void) {
        self.cb = cb
    }
}

public enum MutationStatus {
    case done
    case error
}
