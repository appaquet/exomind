
import UIKit

public class Client {
    var context: OpaquePointer?

    public init() {
        let res = exocore_context_new();
        if res.status == UInt8(ExocoreQueryStatus_Success.rawValue) {
            self.context = res.context
        }
    }

    public func mutate(mutation: Exocore_Index_EntityMutation, onCompletion: @escaping (MutationStatus, Exocore_Index_MutationResult?) -> Void) -> MutationResult {
        // See https://www.mikeash.com/pyblog/friday-qa-2017-08-11-swiftunmanaged.html
        let cb = MutationCallback(cb: onCompletion)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let mutationData = try! mutation.serializedData()
        let handle = mutationData.withUnsafeBytes { (ptr) -> ExocoreMutationHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_mutate(self.context, addr, UInt(mutationData.count), { (status, resultsPtr, resultsSize, observer) in
                let cb = Unmanaged<MutationCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr

                if status == UInt8(ExocoreMutationStatus_Error.rawValue) {
                    cb.cb(.error, nil)
                } else {
                    let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
                    if let results = try? Exocore_Index_MutationResult(serializedData: resultsData) {
                        cb.cb(.done, results)
                    } else {
                        cb.cb(.error, nil)
                    }
                }
            }, observer)
        }

        return MutationResult(mutationHandle: handle, client: self)
    }

    public func query(query: Exocore_Index_EntityQuery, onChange: @escaping (QueryStatus, Exocore_Index_EntityResults?) -> Void) -> QueryHandle {
        // See https://www.mikeash.com/pyblog/friday-qa-2017-08-11-swiftunmanaged.html
        let cb = QueryCallback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let queryData = try! query.serializedData()
        let handle = queryData.withUnsafeBytes { (ptr) -> ExocoreQueryHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_query(self.context, addr, UInt(queryData.count), { (status, resultsPtr, resultsSize, observer) in
                let cb = Unmanaged<QueryCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr

                if status == UInt8(ExocoreQueryStatus_Error.rawValue) {
                    cb.cb(.error, nil)
                } else {
                    let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
                    if let results = try? Exocore_Index_EntityResults(serializedData: resultsData) {
                        cb.cb(.done, results)
                    } else {
                        cb.cb(.error, nil)
                    }
                }
            }, observer)
        }

        return QueryHandle(queryHandle: handle, client: self)
    }

    public func watched_query(query: Exocore_Index_EntityQuery, onChange: @escaping (QueryStatus, Exocore_Index_EntityResults?) -> Void) -> QueryStreamHandle {
        // See https://www.mikeash.com/pyblog/friday-qa-2017-08-11-swiftunmanaged.html
        let cb = QueryCallback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let queryData = try! query.serializedData()
        let handle = queryData.withUnsafeBytes { (ptr) -> ExocoreQueryStreamHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_watched_query(self.context, addr, UInt(queryData.count), { (status, resultsPtr, resultsSize, observer) in
                if status == UInt8(ExocoreQueryStreamStatus_Done.rawValue) {
                    let cb = Unmanaged<QueryCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr
                    cb.cb(.done, nil)
                    return
                } else if  status == UInt8(ExocoreQueryStreamStatus_Error.rawValue) {
                    let cb = Unmanaged<QueryCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr
                    cb.cb(.error, nil)
                    return
                }

                let cb = Unmanaged<QueryCallback>.fromOpaque(observer!).takeUnretainedValue() // don't consume the ptr
                let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
                if let results = try? Exocore_Index_EntityResults(serializedData: resultsData) {
                    cb.cb(.running, results)
                } else {
                    cb.cb(.error, nil)
                }
            }, observer)
        }

        return QueryStreamHandle(queryHandle: handle, client: self)
    }

    deinit {
        print("Client > Deinit start...")
        // free context, which will trigger all query to fail and get freed
        exocore_context_free(self.context)
        print("Client > Deinit done")
    }
}

class QueryCallback {
    var cb: (QueryStatus, Exocore_Index_EntityResults?) -> Void

    init(cb: @escaping (QueryStatus, Exocore_Index_EntityResults?) -> Void) {
        self.cb = cb
    }

    deinit {
        print("QueryCallback > Deinit")
    }
}

public enum QueryStatus {
    case running
    case done
    case error
}

public class QueryStreamHandle {
    var handle: ExocoreQueryStreamHandle
    weak var client: Client?

    init(queryHandle: ExocoreQueryStreamHandle, client: Client) {
        self.handle = queryHandle
        self.client = client
    }

    deinit {
        print("ResultStream > Deinit")
        if let client = self.client {
            exocore_watched_query_cancel(client.context, self.handle)
        }
    }
}

public class QueryHandle {
    var handle: ExocoreQueryHandle
    weak var client: Client?

    init(queryHandle: ExocoreQueryHandle, client: Client) {
        self.handle = queryHandle
        self.client = client
    }

    deinit {
        print("ResultFuture > Deinit")
        if let client = self.client {
            exocore_query_cancel(client.context, self.handle)
        }
    }
}

class MutationCallback {
    var cb: (MutationStatus, Exocore_Index_MutationResult?) -> Void

    init(cb: @escaping (MutationStatus, Exocore_Index_MutationResult?) -> Void) {
        self.cb = cb
    }

    deinit {
        print("MutationCallback > Deinit")
    }
}

public enum MutationStatus {
    case done
    case error
}

public class MutationResult {
    var handle: ExocoreMutationHandle
    weak var client: Client?

    init(mutationHandle: ExocoreMutationHandle, client: Client) {
        self.handle = mutationHandle
        self.client = client
    }

    deinit {
        print("MutationResult > Deinit")
    }
}

public func GenerateId(prefix: String? = nil) -> String {
    let idPtr = exocore_generate_id(prefix)
    let idStr = String(cString: idPtr!)
    exocore_free_string(idPtr)
    return idStr
}
