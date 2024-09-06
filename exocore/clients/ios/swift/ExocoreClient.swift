import UIKit

public enum LogLevel: UInt {
    case none = 0
    case error
    case warn
    case info
    case debug
    case trace
}

public func initialize(logLevel: LogLevel = .info, logFile: String? = nil) {
    // see https://github.com/tokio-rs/mio/issues/949
    signal(SIGPIPE, SIG_IGN)

    exocore_init(logLevel.rawValue, logFile)
}

public class ExocoreClient {
    public static var defaultInstance: ClientInstance?

    @discardableResult
    public static func initialize(node: LocalNode, defaultInstance: Bool = true) throws -> ClientInstance {
        let instance = try ClientInstance(node: node)
        if (defaultInstance) {
            ExocoreClient.defaultInstance = instance
        }
        return instance
    }

    @discardableResult
    public static func initialize(config: Exocore_Core_LocalNodeConfig, defaultInstance: Bool = true) throws -> ClientInstance {
        let node = try LocalNode.from(config: config)
        return try initialize(node: node)
    }

    public static var cell: Cell {
        get {
            ExocoreClient.defaultInstance!.cell
        }
    }

    public static var store: Store {
        get {
            ExocoreClient.defaultInstance!.store
        }
    }
}

public class ClientInstance {
    var client: OpaquePointer?

    public init(node: LocalNode) throws {
        let res = exocore_client_new(node.ptr)

        if res.status == UInt8(ExocoreQueryStatus_Error.rawValue) {
            throw ExocoreError.initialization
        }

        self.client = res.client
    }

    public lazy var cell: Cell = {
        Cell(client: self)
    }()

    public lazy var store: Store = {
        Store(client: self)
    }()

    public func resetTransport() {
        exocore_reset_transport(self.client)
    }

    deinit {
        if self.client != nil {
            // free client, which will trigger all query to fail and get freed
            exocore_client_free(self.client)
        }
    }
}

public enum ConfigFormat {
    case protobuf
    case yaml
}

public enum ExocoreError: Error {
    case initialization
}

public func generateId(prefix: String? = nil) -> String {
    let idPtr = exocore_generate_id(prefix)
    let idStr = String(cString: idPtr!)
    exocore_free_string(idPtr)
    return idStr
}

public func buildInfo() -> Exocore_Core_BuildInfo {
    let infoBytes = exocore_build_info()
    let infoData = Data(bytes: infoBytes.bytes, count: Int(infoBytes.size))
    let info = try! Exocore_Core_BuildInfo(serializedData: infoData)
    exocore_bytes_free(infoBytes)
    return info
}
