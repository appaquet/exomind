import Foundation

import XCTest
import SwiftyJSON
import JavaScriptCore

class JSBridgeTests {
    static var initialized = false

    static func setupInstance() {
        if !initialized {
            let wsFactory = MockWebSocketBridgeFactory()
            let ajaxFactory = MockAjaxBridgeFactory()
            JSBridge.instance = JSBridge(serverHost: "exomind.io", webSocketBridgeFactory: wsFactory, ajaxBridgeFactory: ajaxFactory)
            initialized = true
        }
    }
}

class MockWebSocketBridgeFactory: WebSocketBridgeFactory {
    func build(url: String) -> WebSocketBridgeExport {
        return MockWebSocketBridgeExport()
    }
}

class MockWebSocketBridgeExport: WebSocketBridgeExport {
    var onopen: JSValue? = nil
    var onmessage: JSValue? = nil
    var onerror: JSValue? = nil
    var onclose: JSValue? = nil

    func send(_ data: String) {
    }

    func close() {
    }
}

class MockAjaxBridgeFactory: XMLHttpRequestBridgeFactory {
    func build() -> XMLHttpRequestBridgeExport {
        return MockAjaxBridgeExport()
    }
}

class MockAjaxBridgeExport: XMLHttpRequestBridgeExport {
    var onreadystatechange: JSValue? = nil
    var readyState: Int = 0
    var status: Int = 0
    var responseText: String = ""

    // Multiple parameters functions are not supported. Need to use this method instead. Is it safe? Unowned self?
    // https://gist.github.com/zeitiger/1387f7d996f64b493608
    var open: ((_ method: String, _ url: String) -> Void)? = nil
    var send: ((_ data: String) -> Void)? = nil
    var setRequestHeader: ((_ header: String, _ value: String) -> Void)? = nil
}

