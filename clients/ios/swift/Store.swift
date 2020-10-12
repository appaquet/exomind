import Foundation

public class Store {
    weak var client: ClientInstance?

    init(client: ClientInstance) {
        self.client = client
    }

    public func mutate(mutation: Exocore_Store_MutationRequest, onCompletion: ((MutationStatus, Exocore_Store_MutationResult?) -> Void)? = nil) {
        let callback = onCompletion ?? { status, result in }
        let cb = MutationCallback(cb: callback)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let mutationData = try! mutation.serializedData()
        let _ = mutationData.withUnsafeBytes { (ptr) -> ExocoreMutationHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_mutate(self.client!.context, addr, UInt(mutationData.count), { (status, resultsPtr, resultsSize, observer) in
                let cb = Unmanaged<MutationCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr

                if status == UInt8(ExocoreMutationStatus_Error.rawValue) {
                    cb.cb(.error, nil)
                } else {
                    let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
                    if let results = try? Exocore_Store_MutationResult(serializedData: resultsData) {
                        cb.cb(.done, results)
                    } else {
                        cb.cb(.error, nil)
                    }
                }
            }, observer)
        }
    }

    public func query(query: Exocore_Store_EntityQuery, onChange: @escaping (QueryStatus, Exocore_Store_EntityResults?) -> Void) -> QueryHandle {
        let cb = QueryCallback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let queryData = try! query.serializedData()
        let handle = queryData.withUnsafeBytes { (ptr) -> ExocoreQueryHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_query(self.client!.context, addr, UInt(queryData.count), { (status, resultsPtr, resultsSize, observer) in
                let cb = Unmanaged<QueryCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr

                if status == UInt8(ExocoreQueryStatus_Error.rawValue) {
                    cb.cb(.error, nil)
                } else {
                    let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
                    if let results = try? Exocore_Store_EntityResults(serializedData: resultsData) {
                        cb.cb(.done, results)
                    } else {
                        cb.cb(.error, nil)
                    }
                }
            }, observer)
        }

        return QueryHandle(queryHandle: handle, client: self.client!)
    }

    public func watchedQuery(query: Exocore_Store_EntityQuery, onChange: @escaping (QueryStatus, Exocore_Store_EntityResults?) -> Void) -> QueryStreamHandle {
        let cb = QueryCallback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let queryData = try! query.serializedData()
        let handle = queryData.withUnsafeBytes { (ptr) -> ExocoreQueryStreamHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_watched_query(self.client!.context, addr, UInt(queryData.count), { (status, resultsPtr, resultsSize, observer) in
                if status == UInt8(ExocoreQueryStreamStatus_Done.rawValue) {
                    let cb = Unmanaged<QueryCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr
                    cb.cb(.done, nil)
                    return
                } else if status == UInt8(ExocoreQueryStreamStatus_Error.rawValue) {
                    let cb = Unmanaged<QueryCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr
                    cb.cb(.error, nil)
                    return
                }

                let cb = Unmanaged<QueryCallback>.fromOpaque(observer!).takeUnretainedValue() // don't consume the ptr
                let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
                if let results = try? Exocore_Store_EntityResults(serializedData: resultsData) {
                    cb.cb(.running, results)
                } else {
                    cb.cb(.error, nil)
                }
            }, observer)
        }

        return QueryStreamHandle(queryHandle: handle, client: self.client!)
    }
}

public enum QueryStatus {
    case running
    case done
    case error
}

public class QueryStreamHandle {
    var handle: ExocoreQueryStreamHandle
    weak var client: ClientInstance?

    init(queryHandle: ExocoreQueryStreamHandle, client: ClientInstance) {
        self.handle = queryHandle
        self.client = client
    }

    deinit {
        if let client = self.client {
            exocore_watched_query_cancel(client.context, self.handle)
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
            exocore_query_cancel(client.context, self.handle)
        }
    }
}

public class QueryCallback {
    var cb: (QueryStatus, Exocore_Store_EntityResults?) -> Void

    init(cb: @escaping (QueryStatus, Exocore_Store_EntityResults?) -> Void) {
        self.cb = cb
    }
}

public class MutationCallback {
    var cb: (MutationStatus, Exocore_Store_MutationResult?) -> Void

    init(cb: @escaping (MutationStatus, Exocore_Store_MutationResult?) -> Void) {
        self.cb = cb
    }
}

public enum MutationStatus {
    case done
    case error
}
