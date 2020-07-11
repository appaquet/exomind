//
//  SessionStore.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-12-14.
//  Copyright © 2015 Exomind. All rights reserved.
//

import Foundation
import JavaScriptCore

class SessionStore {
    fileprivate static var initialized: Bool = false
    fileprivate static var callbacks: [() -> ()] = []

    fileprivate static func initialize() {
        if (!SessionStore.initialized) {
            guard let sesionObj = DomainStore.instance.jsContext.evaluateScript("exomind.session.SessionStore") else { return }

            // register change callback
            let onChange: @convention(block) (String) -> Void = { url in
                SessionStore.callbacks.forEach({ (cb) -> () in
                    cb()
                })
            }
            sesionObj.setObject(unsafeBitCast(onChange, to: AnyObject.self), forKeyedSubscript: "onChangeIosCallback" as (NSCopying & NSObjectProtocol))
            let ref = sesionObj.forProperty("onChangeIosCallback")!
            let _ = sesionObj.invokeMethod("onChange", withArguments: [ref])

            SessionStore.initialized = true
        }
    }

    static func onChange(_ cb: @escaping (() -> ())) {
        SessionStore.initialize()
        SessionStore.callbacks.append(cb)
    }
    
    fileprivate static func sessionJsProperty(_ name: String) -> JSValue? {
        return DomainStore.instance.jsContext.evaluateScript("exomind.session.SessionStore.session.\(name)")
    }

    static func integrations() -> [EntityTrait] {
        guard let integrations = SessionStore.sessionJsProperty("integrations")
            else { return [] }
        
        return DomainStore.instance.jsArrayToJSValues(integrations)
            .compactMap { BridgeEntityConverter.entityFromJavascript($0) }
            .compactMap { EntityTrait.init(entity: $0) }
    }

    static func inboxEntity() -> HCEntity? {
        let jsInbox = SessionStore.sessionJsProperty("inboxEntity")
        if (!(jsInbox?.isNull)!) {
            return BridgeEntityConverter.entityFromJavascript(jsInbox!)
        } else {
            return nil
        }
    }
  
    static func mindEntity() -> HCEntity? {
        let jsInbox = SessionStore.sessionJsProperty("mindEntity")
        if (!(jsInbox?.isNull)!) {
            return BridgeEntityConverter.entityFromJavascript(jsInbox!)
        } else {
            return nil
        }
    }

    static func userLoaded() -> Bool {
        return SessionStore.sessionJsProperty("userLoaded")?.toBool() ?? false
    }

    static func userKey() -> String {
        return SessionStore.sessionJsProperty("key")?.toString() ?? ""
    }

    static func authenticated() -> Bool {
        return SessionStore.sessionJsProperty("authenticated")?.toBool() ?? false
    }

}
