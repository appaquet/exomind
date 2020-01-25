
import UIKit


class Client {
    var context: OpaquePointer?

    init() {
        let res = exocore_context_new();
        if res.status == UInt8(ExocoreQueryStatus_Success.rawValue) {
            self.context = res.context
        }
    }

    func query(onChange: @escaping (QueryStatus, Exocore_Index_EntityResults?) -> Void) -> ResultFuture {
        // See https://www.mikeash.com/pyblog/friday-qa-2017-08-11-swiftunmanaged.html
        let cb = Callback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let handle = exocore_query(self.context, "hello world", { (status, resultsPtr, resultsSize, observer) in
            if status == UInt8(ExocoreQueryStreamStatus_Done.rawValue) {
                let cb = Unmanaged<Callback>.fromOpaque(observer!).takeRetainedValue() // consume ptr
                cb.cb(.done, nil)
                return
            } else if status == UInt8(ExocoreQueryStreamStatus_Error.rawValue) {
                let cb = Unmanaged<Callback>.fromOpaque(observer!).takeRetainedValue() // consume ptr
                cb.cb(.error, nil)
                return
            }


            let cb = Unmanaged<Callback>.fromOpaque(observer!).takeUnretainedValue() // don't consume the ptr
            let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
            if let results = try? Exocore_Index_EntityResults(serializedData: resultsData) {
                cb.cb(.running, results)
            } else {
                cb.cb(.error, nil)
            }
        }, observer)

        return ResultFuture(queryHandle: handle, client: self)
    }

    func watched_query(onChange: @escaping (QueryStatus, Exocore_Index_EntityResults?) -> Void) -> ResultStream {
        // See https://www.mikeash.com/pyblog/friday-qa-2017-08-11-swiftunmanaged.html
        let cb = Callback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let handle = exocore_watched_query(self.context, "hello world", { (status, resultsPtr, resultsSize, observer) in
            if status == UInt8(ExocoreQueryStreamStatus_Done.rawValue) {
                let cb = Unmanaged<Callback>.fromOpaque(observer!).takeRetainedValue() // consume ptr
                cb.cb(.done, nil)
                return
            } else if  status == UInt8(ExocoreQueryStreamStatus_Error.rawValue) {
                let cb = Unmanaged<Callback>.fromOpaque(observer!).takeRetainedValue() // consume ptr
                cb.cb(.error, nil)
                return
            }

            let cb = Unmanaged<Callback>.fromOpaque(observer!).takeUnretainedValue() // don't consume the ptr
            let resultsData = Data(bytes: resultsPtr!, count: Int(resultsSize))
            if let results = try? Exocore_Index_EntityResults(serializedData: resultsData) {
                cb.cb(.running, results)
            } else {
                cb.cb(.error, nil)
            }
        }, observer)

        return ResultStream(queryHandle: handle, client: self)
    }

    deinit {
        print("Client > Deinit start...")
        // free context, which will trigger all query to fail and get freed
        exocore_context_free(self.context)
        print("Client > Deinit done")
    }
}

class Callback {
    var cb: (QueryStatus, Exocore_Index_EntityResults?) -> Void

    init(cb: @escaping (QueryStatus, Exocore_Index_EntityResults?) -> Void) {
        self.cb = cb
    }

    deinit {
        print("Callback > Deinit")
    }
}

enum QueryStatus {
    case running
    case done
    case error
}

class ResultStream {
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

class ResultFuture {
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
