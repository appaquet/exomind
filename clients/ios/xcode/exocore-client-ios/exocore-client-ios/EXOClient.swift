import UIKit

public enum EXOConfigFormat {
    case protobuf
    case yaml
}

public class EXOClient {
    fileprivate var context: OpaquePointer?

    public init(config: Exocore_Core_LocalNodeConfig) throws {
        let configData = try config.serializedData()

        self.context = try EXOClient.contextFromConfig(configData: configData, format: UInt8(ExocoreConfigFormat_Protobuf.rawValue))
    }

    public init(yamlConfig: String) throws {
        guard let configData = yamlConfig.data(using: .utf8) else {
            print("EXOClient > Couldn't get data from yaml string")
            throw EXOError.initialization
        }

        self.context = try EXOClient.contextFromConfig(configData: configData, format: UInt8(ExocoreConfigFormat_Yaml.rawValue))
    }

    public static func contextFromConfig(configData: Data, format: UInt8) throws -> OpaquePointer {
        try configData.withUnsafeBytes { (ptr) -> OpaquePointer in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            let res = exocore_context_new(addr, UInt(configData.count), format);

            if res.status == UInt8(ExocoreQueryStatus_Success.rawValue) {
                return res.context
            } else {
                throw EXOError.initialization
            }
        }
    }

    public func mutate(mutation: Exocore_Index_EntityMutation, onCompletion: @escaping (EXOMutationStatus, Exocore_Index_MutationResult?) -> Void) -> EXOMutationResult {
        let cb = EXOMutationCallback(cb: onCompletion)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let mutationData = try! mutation.serializedData()
        let handle = mutationData.withUnsafeBytes { (ptr) -> ExocoreMutationHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_mutate(self.context, addr, UInt(mutationData.count), { (status, resultsPtr, resultsSize, observer) in
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

        return EXOMutationResult(mutationHandle: handle, client: self)
    }

    public func query(query: Exocore_Index_EntityQuery, onChange: @escaping (EXOQueryStatus, Exocore_Index_EntityResults?) -> Void) -> EXOQueryHandle {
        let cb = EXOQueryCallback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let queryData = try! query.serializedData()
        let handle = queryData.withUnsafeBytes { (ptr) -> ExocoreQueryHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_query(self.context, addr, UInt(queryData.count), { (status, resultsPtr, resultsSize, observer) in
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

        return EXOQueryHandle(queryHandle: handle, client: self)
    }

    public func watched_query(query: Exocore_Index_EntityQuery, onChange: @escaping (EXOQueryStatus, Exocore_Index_EntityResults?) -> Void) -> EXOQueryStreamHandle {
        let cb = EXOQueryCallback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let queryData = try! query.serializedData()
        let handle = queryData.withUnsafeBytes { (ptr) -> ExocoreQueryStreamHandle in
            let addr = ptr.bindMemory(to: UInt8.self).baseAddress

            return exocore_watched_query(self.context, addr, UInt(queryData.count), { (status, resultsPtr, resultsSize, observer) in
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

        return EXOQueryStreamHandle(queryHandle: handle, client: self)
    }

    deinit {
        if self.context != nil {
            print("EXOClient > Deinit start...")
            // free context, which will trigger all query to fail and get freed
            exocore_context_free(self.context)
            print("EXOClient > Deinit done")
        }
    }
}

public enum EXOError: Error {
    case initialization
}

public class EXOQueryCallback {
    var cb: (EXOQueryStatus, Exocore_Index_EntityResults?) -> Void

    init(cb: @escaping (EXOQueryStatus, Exocore_Index_EntityResults?) -> Void) {
        self.cb = cb
    }

    deinit {
        print("EXOQueryCallback > Deinit")
    }
}

public enum EXOQueryStatus {
    case running
    case done
    case error
}

public class EXOQueryStreamHandle {
    var handle: ExocoreQueryStreamHandle
    weak var client: EXOClient?

    init(queryHandle: ExocoreQueryStreamHandle, client: EXOClient) {
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
    weak var client: EXOClient?

    init(queryHandle: ExocoreQueryHandle, client: EXOClient) {
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
    weak var client: EXOClient?

    init(mutationHandle: ExocoreMutationHandle, client: EXOClient) {
        self.handle = mutationHandle
        self.client = client
    }

    deinit {
        print("EXOMutationResult > Deinit")
    }
}

public func EXOGenerateId(prefix: String? = nil) -> String {
    let idPtr = exocore_generate_id(prefix)
    let idStr = String(cString: idPtr!)
    exocore_free_string(idPtr)
    return idStr
}
