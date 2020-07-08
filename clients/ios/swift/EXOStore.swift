
import Foundation

public class EXOStore {
    var client: ExocoreClient

    init(client: ExocoreClient) {
        self.client = client
    }

    public func mutate(mutation: Exocore_Index_MutationRequest, onCompletion: @escaping (EXOMutationStatus, Exocore_Index_MutationResult?) -> Void) -> EXOMutationResult {
        let cb = EXOMutationCallback(cb: onCompletion)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let mutationData = try! mutation.serializedData()
        let handle = mutationData.withUnsafeBytes { (ptr) -> ExocoreMutationHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_mutate(self.client.context, addr, UInt(mutationData.count), { (status, resultsPtr, resultsSize, observer) in
                let cb = Unmanaged<EXOMutationCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr

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

        return EXOMutationResult(mutationHandle: handle, client: self.client)
    }

    public func query(query: Exocore_Index_EntityQuery, onChange: @escaping (EXOQueryStatus, Exocore_Index_EntityResults?) -> Void) -> EXOQueryHandle {
        let cb = EXOQueryCallback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let queryData = try! query.serializedData()
        let handle = queryData.withUnsafeBytes { (ptr) -> ExocoreQueryHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_query(self.client.context, addr, UInt(queryData.count), { (status, resultsPtr, resultsSize, observer) in
                let cb = Unmanaged<EXOQueryCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr

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

        return EXOQueryHandle(queryHandle: handle, client: self.client)
    }

    public func watchedQuery(query: Exocore_Index_EntityQuery, onChange: @escaping (EXOQueryStatus, Exocore_Index_EntityResults?) -> Void) -> EXOQueryStreamHandle {
        let cb = EXOQueryCallback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let queryData = try! query.serializedData()
        let handle = queryData.withUnsafeBytes { (ptr) -> ExocoreQueryStreamHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_watched_query(self.client.context, addr, UInt(queryData.count), { (status, resultsPtr, resultsSize, observer) in
                if status == UInt8(ExocoreQueryStreamStatus_Done.rawValue) {
                    let cb = Unmanaged<EXOQueryCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr
                    cb.cb(.done, nil)
                    return
                } else if status == UInt8(ExocoreQueryStreamStatus_Error.rawValue) {
                    let cb = Unmanaged<EXOQueryCallback>.fromOpaque(observer!).takeRetainedValue() // consume ptr
                    cb.cb(.error, nil)
                    return
                }

                let cb = Unmanaged<EXOQueryCallback>.fromOpaque(observer!).takeUnretainedValue() // don't consume the ptr
                let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
                if let results = try? Exocore_Index_EntityResults(serializedData: resultsData) {
                    cb.cb(.running, results)
                } else {
                    cb.cb(.error, nil)
                }
            }, observer)
        }

        return EXOQueryStreamHandle(queryHandle: handle, client: self.client)
    }
}

public enum EXOQueryStatus {
    case running
    case done
    case error
}

public class EXOQueryStreamHandle {
    var handle: ExocoreQueryStreamHandle
    weak var client: ExocoreClient?

    init(queryHandle: ExocoreQueryStreamHandle, client: ExocoreClient) {
        self.handle = queryHandle
        self.client = client
    }

    deinit {
        print("EXOResultStreamHandle > Deinit")
        if let client = self.client {
            exocore_watched_query_cancel(client.context, self.handle)
        }
    }
}

public class EXOQueryHandle {
    var handle: ExocoreQueryHandle
    weak var client: ExocoreClient?

    init(queryHandle: ExocoreQueryHandle, client: ExocoreClient) {
        self.handle = queryHandle
        self.client = client
    }

    deinit {
        print("EXOQueryHandle > Deinit")
        if let client = self.client {
            exocore_query_cancel(client.context, self.handle)
        }
    }
}

public class EXOMutationCallback {
    var cb: (EXOMutationStatus, Exocore_Index_MutationResult?) -> Void

    init(cb: @escaping (EXOMutationStatus, Exocore_Index_MutationResult?) -> Void) {
        self.cb = cb
    }

    deinit {
        print("EXOMutationCallback > Deinit")
    }
}

public enum EXOMutationStatus {
    case done
    case error
}

public class EXOMutationResult {
    var handle: ExocoreMutationHandle
    weak var client: ExocoreClient?

    init(mutationHandle: ExocoreMutationHandle, client: ExocoreClient) {
        self.handle = mutationHandle
        self.client = client
    }

    deinit {
        print("EXOMutationResult > Deinit")
    }
}
