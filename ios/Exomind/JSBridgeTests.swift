import Foundation

import XCTest
import SwiftyJSON
import JavaScriptCore

class JSBridgeTests {
    static var initialized = false

    static func setupInstance() {
        if !initialized {
            JSBridge.instance = JSBridge()
            initialized = true
        }
    }
}
