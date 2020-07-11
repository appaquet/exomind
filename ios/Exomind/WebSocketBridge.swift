//
//  WebSocketBridge.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-10-06.
//  Copyright © 2015 Exomind. All rights reserved.
//

import Foundation
import JavaScriptCore
import SwiftWebSocket

class RealWebSocketBridgeFactory: WebSocketBridgeFactory {
    func build(url: String) -> WebSocketBridgeExport {
        return WebSocketBridge(url: url)
    }
}

@objc class WebSocketBridge: NSObject, WebSocketBridgeExport {
    var ws: WebSocket?

    dynamic var onopen: JSValue?
    dynamic var onmessage: JSValue?
    dynamic var onerror: JSValue?
    dynamic var onclose: JSValue?

    init(url: String) {
        super.init()

        print("WebSocketBridge > Trying to connect to \(url)")

        // create a custom http request so that we can transfert cookies with it
        let nsurl = URL(string: url)!
        var request = URLRequest(url: nsurl)
        let cookies = HTTPCookieStorage.shared.cookies(for: nsurl)!
        let headers = HTTPCookie.requestHeaderFields(with: cookies)
        request.allHTTPHeaderFields = headers

        self.ws = WebSocket(request: request)
        self.ws?.event.open = self.handleOpen
        self.ws?.event.message = self.handleMessage
        self.ws?.event.error = self.handleError
        self.ws?.event.close = self.handleClose
    }

    func send(_ data: String) {
        self.ws?.send(data)
    }

    func close() {
        print("WebSocketBride > Closing WebSocket")
        self.ws?.close()
    }

    func handleMessage(_ data: Any) {
        if let msg = data as? String {
            let obj = [
                    "data": msg
            ]
            let _ = self.onmessage?.call(withArguments: [obj])
        }
    }

    func handleOpen() {
        print("WebSocketBridge > WebSocket open")
        let _ = self.onopen?.call(withArguments: [])
    }

    func handleError(_ error: Error) {
        print("WebSocketBridge > WebSocket got an error \(error)")
        let _ = self.onerror?.call(withArguments: ["\(error)"])
    }

    func handleClose(_ code: Int, reason: String, wasClean: Bool) {
        print("WebSocketBridge > WebSocket got closed \(code) \(reason) \(wasClean)")
        let _ = self.onclose?.call(withArguments: [code, reason])
    }

}
