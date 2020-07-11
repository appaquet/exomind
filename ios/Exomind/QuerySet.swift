//
//  QuerySet.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-10-06.
//  Copyright © 2015 Exomind. All rights reserved.
//

import Foundation
import JavaScriptCore

class QuerySet {
    fileprivate var jsContext: JSContext!
    fileprivate var jsObj: JSValue!
    fileprivate var callbacks: [() -> ()] = []

    init(jsContext: JSContext, jsObj: JSValue) {
        self.jsContext = jsContext
        self.jsObj = jsObj
        self.bind()
    }

    func bind() {
        // register change callback
        let onChange: @convention(block) (String) -> Void = { [weak self] url in
            self?.callbacks.forEach({ (cb) -> () in
                DispatchQueue.main.async {
                    cb()
                }
            })
        }
        self.jsObj.setObject(unsafeBitCast(onChange, to: AnyObject.self), forKeyedSubscript: "onChangeIosCallback" as (NSCopying & NSObjectProtocol))
        let ref = self.jsObj.forProperty("onChangeIosCallback")!
        self.jsObj.invokeMethod("onChange", withArguments: [ref])
    }

    func executeQuery(_ query: Query, reExecute: Bool = false) -> Query {
        let finalQuery = self.jsObj.invokeMethod("executeQuery", withArguments: [query.jsObj, reExecute])
        return Query(jsObj: finalQuery!)
    }
    
    func executeQuery(_ query: HCEntityQuery, reExecute: Bool = false) -> Query {
        return self.executeQuery(query.toDomainQuery(), reExecute: reExecute)
    }

    func onChange(_ callback: @escaping () -> ()) {
        self.callbacks.append(callback)
    }

    func release() {
        self.callbacks = []
        self.jsObj.invokeMethod("release", withArguments: [])
    }

    deinit {
        print("QuerySet > Deinit")
        self.release()
    }
}
