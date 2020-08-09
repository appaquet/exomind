import Foundation

import Foundation
import JavaScriptCore

protocol WebSocketBridgeFactory {
    func build(url: String) -> WebSocketBridgeExport
}

@objc protocol WebSocketBridgeExport: JSExport {
    var onopen: JSValue? { get set }
    var onmessage: JSValue? { get set }
    var onerror: JSValue? { get set }
    var onclose: JSValue? { get set }
    func send(_ data: String)

    func close()
}
