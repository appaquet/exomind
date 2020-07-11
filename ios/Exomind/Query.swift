//
//  Query.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-10-07.
//  Copyright © 2015 Exomind. All rights reserved.
//

import Foundation
import JavaScriptCore
import SwiftyJSON

class Query {
    var jsObj: JSValue
    let hcSerializer = HCJsonSerialization()

    init(jsObj: JSValue) {
        self.jsObj = jsObj
    }
    
    static func unitQuery() -> Query {
        let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.queries.UnitQuery")
        let queryObj = builderFunc?.construct(withArguments: [])
        return Query(jsObj: queryObj!)
    }

    func isLoaded() -> Bool {
        return self.jsObj.invokeMethod("isLoaded", withArguments: []).toBool()
    }

    func hasError() -> Bool {
        return self.jsObj.invokeMethod("hasError", withArguments: []).toBool()
    }

    func error() -> JSValue? {
        let obj = self.jsObj.forProperty("error")
        if (!(obj?.isNull)!) {
            return obj
        } else {
            return nil
        }
    }
    
    func hash() -> String {
        return self.jsObj.invokeMethod("hash", withArguments: []).toString()
    }
    
    func release() {
        self.jsObj.invokeMethod("release", withArguments: [])
    }

    func hasResults() -> Bool {
        return !self.jsObj.invokeMethod("resultAsEntities", withArguments: []).isNull
    }

    func resultAsEntity() -> HCEntity? {
        if (!self.isLoaded()) {
            return nil
        }

        if let jsObj = self.jsObj.invokeMethod("resultAsEntity", withArguments: []) {
            if jsObj.isNull {
                return nil
            } else {
                let raw = JSON(jsObj.forProperty("_raw").toDictionary())
                let deser =  hcSerializer.deserializeEntity(raw)
                if let err = hcSerializer.error {
                    print("Query > Error deserializing entity: \(err)")
                    hcSerializer.error = nil
                }
                return deser
            }
        } else {
            return nil
        }
    }

    func resultsAsEntities() -> [HCEntity] {
        guard let data = self.jsObj.invokeMethod("resultAsEntities", withArguments: [])
            else { return [] }
        
        let entities = DomainStore.instance.jsArrayToJSValues(data).compactMap { jsval -> HCEntity? in
            let raw = JSON(jsval.forProperty("_raw").toDictionary())
            let deser = hcSerializer.deserializeEntity(raw)
            if let err = hcSerializer.error {
                print("Query > Error deserializing entities: \(err)")
                hcSerializer.error = nil
            }
            return deser
        }
        return entities
    }

    func expand(_ count: Int? = nil) -> Query? {
        let maybeQuery = self.jsObj.invokeMethod("expand", withArguments: [self.orNull(count as AnyObject?)])
        if (!(maybeQuery?.isNull)!) {
            return Query(jsObj: maybeQuery!)
        } else {
            return nil
        }
    }

    func orNull(_ opt: AnyObject?) -> JSValue {
        if let object = opt {
            return JSValue(object: object, in: self.jsObj.context)
        } else {
            return JSValue(nullIn: self.jsObj.context)
        }
    }

}
