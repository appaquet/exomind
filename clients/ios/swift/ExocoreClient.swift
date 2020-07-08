
import UIKit

public enum EXOConfigFormat {
    case protobuf
    case yaml
}

public class ExocoreClient {
    public static var defaultInstance: ExocoreClient?

    var context: OpaquePointer?

    public init(config: Exocore_Core_LocalNodeConfig, defaultInstance: Bool = true) throws {
        let configData = try config.serializedData()

        self.context = try ExocoreClient.contextFromConfig(configData: configData, format: UInt8(ExocoreConfigFormat_Protobuf.rawValue))

        if defaultInstance {
            ExocoreClient.defaultInstance = self
        }
    }

    public init(yamlConfig: String, defaultInstance: Bool = true) throws {
        guard let configData = yamlConfig.data(using: .utf8) else {
            print("EXOClient > Couldn't get data from YAML string")
            throw EXOError.initialization
        }

        self.context = try ExocoreClient.contextFromConfig(configData: configData, format: UInt8(ExocoreConfigFormat_Yaml.rawValue))

        if defaultInstance {
            ExocoreClient.defaultInstance = self
        }
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

    public func store() -> EXOStore {
        EXOStore(client: self)
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

public func EXOGenerateId(prefix: String? = nil) -> String {
    let idPtr = exocore_generate_id(prefix)
    let idStr = String(cString: idPtr!)
    exocore_free_string(idPtr)
    return idStr
}
