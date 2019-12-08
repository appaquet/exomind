
import UIKit


class Client {
    var context: OpaquePointer?

    init() {
        let res = exocore_context_new();
        if res.status == UInt8(ExocoreQueryStatus_Success.rawValue) {
            self.context = res.context
        }
    }

    func query(onChange: @escaping (QueryStatus, QueryResults?) -> Void) -> ResultFuture {
        // See https://www.mikeash.com/pyblog/friday-qa-2017-08-11-swiftunmanaged.html
        let cb = Callback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let handle = exocore_query(self.context, "hello world", { (status, jsonPtr, observer) in
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
            if let results = QueryResults.parse(jsonPtr: jsonPtr) {
                cb.cb(.running, results)
            } else {
                cb.cb(.error, nil)
            }
        }, observer)

        return ResultFuture(queryHandle: handle, client: self)
    }

    func watched_query(onChange: @escaping (QueryStatus, QueryResults?) -> Void) -> ResultStream {
        // See https://www.mikeash.com/pyblog/friday-qa-2017-08-11-swiftunmanaged.html
        let cb = Callback(cb: onChange)
        let observer = UnsafeRawPointer(Unmanaged.passRetained(cb).toOpaque())

        let handle = exocore_watched_query(self.context, "hello world", { (status, jsonPtr, observer) in
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
            if let results = QueryResults.parse(jsonPtr: jsonPtr) {
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
    var cb: (QueryStatus, QueryResults?) -> Void

    init(cb: @escaping (QueryStatus, QueryResults?) -> Void) {
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

public struct QueryResults: Codable, Equatable {
    public var results: [QueryResult]
    public var source: String?

    enum CodingKeys: String, CodingKey {
        case results
        case source
    }

    static func parse(jsonPtr: UnsafePointer<Int8>?) -> QueryResults? {
        guard let jsonString = jsonPtr, let nativeJsonString = String(utf8String: jsonString) else {
            print("QueryResults > Error converting results cstring")
            return nil
        }

        guard let resultsJsonData = nativeJsonString.data(using: .utf8) else {
            print("QueryResults > Error converting results string to data")
            return nil
        }

        do {
            let decoder = JSONDecoder()
            let results = try decoder.decode(QueryResults.self, from: resultsJsonData)
            return results

        } catch {
            print("QueryResults > Error parsing results JSON \(error)")
            return nil
        }
    }
}

public struct QueryResult: Codable, Equatable {
    public var entity: Entity

    enum CodingKeys: String, CodingKey {
        case entity
    }
}

public struct Entity: Codable, Equatable {
    public var id: String
    public var traits: [Trait]

    enum CodingKeys: String, CodingKey {
        case id
        case traits
    }
}

public struct Trait: Codable, Equatable {
    public var id: String
    public var type: String
    public var title: String?

    enum CodingKeys: String, CodingKey {
        case id = "_id"
        case type = "_type"
        case title
    }
}
