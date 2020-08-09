import Foundation
import JavaScriptCore

class JSBridge {
    static var instance: JSBridge!

    fileprivate let serverHost: String
    fileprivate var timers: [JSTimer] = []
    fileprivate(set) var jsContext: JSContext!
    fileprivate let webSocketBridgeFactory: WebSocketBridgeFactory
    fileprivate let ajaxBridgeFactory: XMLHttpRequestBridgeFactory

    init(serverHost: String, webSocketBridgeFactory: WebSocketBridgeFactory, ajaxBridgeFactory: XMLHttpRequestBridgeFactory) {
        self.serverHost = serverHost
        self.webSocketBridgeFactory = webSocketBridgeFactory
        self.ajaxBridgeFactory = ajaxBridgeFactory

        self.initializeContext()
    }

    func pauseConnections() {
        self.jsContext.evaluateScript("exomind.backendSocket.pauseConnections()")
    }

    func resumeConnections() {
        self.jsContext.evaluateScript("exomind.backendSocket.resumeConnections()")
    }

    func resetConnections() {
        self.jsContext.evaluateScript("exomind.backendSocket.resetConnections()")
    }

    func connected() -> Bool {
        return self.jsContext.evaluateScript("exomind.backendSocket.connected").toBool()
    }

    func unauthorized() -> Bool {
        return self.jsContext.evaluateScript("exomind.backendSocket.unauthorized").toBool()
    }

    func orNull(_ opt: AnyObject?) -> JSValue {
        if let object = opt {
            return JSValue(object: object, in: self.jsContext)
        } else {
            return JSValue(nullIn: self.jsContext)
        }
    }

    func jsArrayToJSValues(_ array: JSValue) -> [JSValue] {
        let length = array.forProperty("length").toUInt32()
        var ret: [JSValue] = []
        if (length > 0) {
            for i in 0...(length - 1) {
                ret.append(array.atIndex(Int(i)))
            }
        }
        return ret
    }

    func destroy() {
        self.timers.forEach {
            (elem) -> () in
            elem.stop()
        }
        self.jsContext = nil
    }

    fileprivate func initializeContext() {
        if jsContext == nil {
            jsContext = JSContext()

            jsContext.exceptionHandler = {
                context, exception in
                let stack = exception?.forProperty("stack")
                print("JS > Got an exception error='\(String(describing: exception))' function='\(String(describing: stack))'")
            }

            jsContext.evaluateScript("window = {}; window.location = {}; window.location.host = '\(serverHost)'; exomind = {};");

            // Websocket polyfill
            let websocketBuilder: @convention(block) (String) -> WebSocketBridgeExport = {
                [weak self] url in
                return self!.webSocketBridgeFactory.build(url: url)
            }
            jsContext.setObject(unsafeBitCast(websocketBuilder, to: AnyObject.self), forKeyedSubscript: "WebSocket" as (NSCopying & NSObjectProtocol))

            // XMLHttpRequest polyfill
            let xmlhttprequestBuilder: @convention(block) (String) -> XMLHttpRequestBridgeExport = {
                [weak self] url in
                return self!.ajaxBridgeFactory.build()
            }
            jsContext.setObject(unsafeBitCast(xmlhttprequestBuilder, to: AnyObject.self), forKeyedSubscript: "XMLHttpRequest" as (NSCopying & NSObjectProtocol))

            // support for setInterval
            let setInterval: @convention(block) (JSValue, Int) -> Void = {
                callback, delay in
                let interval = JSTimer(callback: callback, delay: Double(delay) / 1000.0, repeats: true)
                self.timers.append(interval)
            }
            jsContext.setObject(unsafeBitCast(setInterval, to: AnyObject.self), forKeyedSubscript: "setInterval" as (NSCopying & NSObjectProtocol))

            // support for setTimeout
            let setTimeout: @convention(block) (JSValue, Int) -> Void = {
                callback, delay in
                let _ = JSTimer(callback: callback, delay: Double(delay) / 1000.0, repeats: false)
            }
            jsContext.setObject(unsafeBitCast(setTimeout, to: AnyObject.self), forKeyedSubscript: "setTimeout" as (NSCopying & NSObjectProtocol))

            // support for primitive console.log
            let consoleLog: @convention(block) (JSValue) -> Void = {
                log in
                print("JS > \(log.description)")
            }
            jsContext.setObject(unsafeBitCast(consoleLog, to: AnyObject.self), forKeyedSubscript: "consoleLog" as (NSCopying & NSObjectProtocol))
            jsContext.evaluateScript("console = {log: consoleLog};")

            // load the store.js file
            let resource = Bundle.main.path(forResource: "store", ofType: "js", inDirectory: "js")
            let content = FileManager.default.contents(atPath: resource!)
            let str = NSString(data: content!, encoding: String.Encoding.utf8.rawValue) as String?
            jsContext.evaluateScript(str!)

            // ensure session loaded
            jsContext.evaluateScript("exomind.session.SessionStore.ensureLoaded()")
        }
    }
}
