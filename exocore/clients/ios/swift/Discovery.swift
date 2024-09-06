
import Foundation

public class Discovery {
    var ptr: OpaquePointer

    private init(ptr: OpaquePointer) {
        self.ptr = ptr
    }

    public convenience init() throws {
        let res = exocore_discovery_new(nil)
        if res.status == ExocoreQueryStatus_Error.rawValue {
            throw DiscoveryError.initialization
        }

        self.init(ptr: res.discovery)
    }

    public func joinCell(localNode: LocalNode, callback: @escaping (_ stage: DiscoveryStage) -> Void) {
        let cbCtx = JoinCallbackContext(cb: callback)
        let cbCtxPtr = UnsafeRawPointer(Unmanaged.passRetained(cbCtx).toOpaque())

        return exocore_discovery_join_cell(self.ptr, localNode.ptr, { (status, pin, newLocalNode, cbCtxCtr) in
            switch UInt32(status) {
            case ExocoreDiscoveryStatus_InProgress.rawValue:
                let cbCtx = Unmanaged<JoinCallbackContext>.fromOpaque(cbCtxCtr!).takeUnretainedValue() // don't consume ptr
                cbCtx.cb(.pin(pin))

            case ExocoreDiscoveryStatus_Success.rawValue:
                let cbCtx = Unmanaged<JoinCallbackContext>.fromOpaque(cbCtxCtr!).takeRetainedValue() // consume ptr
                let node = LocalNode(ptr: newLocalNode!)
                cbCtx.cb(.success(node))

            case ExocoreDiscoveryStatus_Error.rawValue:
                let cbCtx = Unmanaged<JoinCallbackContext>.fromOpaque(cbCtxCtr!).takeRetainedValue() // consume ptr
                cbCtx.cb(.error(.join))
                
            default:
                print("Unhandled join cell status: \(status)")
            }
        }, cbCtxPtr)
    }

    deinit {
        exocore_discovery_free(self.ptr)
    }
}


public enum DiscoveryStage {
    case pin(UInt32)
    case success(LocalNode)
    case error(DiscoveryError)
}

public enum DiscoveryError: Error {
    case initialization
    case join
}

class JoinCallbackContext {
    var cb: (DiscoveryStage) -> Void

    init(cb: @escaping (DiscoveryStage) -> Void) {
        self.cb = cb
    }
}
