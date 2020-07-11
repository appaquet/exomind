//
//  BridgeEntityConverter.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-11-03.
//  Copyright © 2016 Exomind. All rights reserved.
//

import Foundation
import SwiftyJSON
import JavaScriptCore

class BridgeEntityConverter {
    static let hcSerializer = HCJsonSerialization()
    
    static func entityFromJavascript(_ jsObj: JSValue) -> HCEntity? {
        let json = JSON(jsObj.forProperty("_raw").toDictionary())
        return BridgeEntityConverter.hcSerializer.deserializeEntity(json)
    }
    
    static func entityToJavascript(_ entity: HCEntity) -> JSValue? {
        let json = BridgeEntityConverter.hcSerializer.serialize(entity)
        let entity = DomainStore.instance.jsContext.evaluateScript("exomind.honeycomb.Entity")
        return entity?.construct(withArguments: [json.object])
    }
    
    static func recordToJsRecord(_ record: HCRecordFields) -> JSValue? {
        let jsonData = BridgeEntityConverter.hcSerializer.serialize(record).object
        let traitBuilderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.honeycomb.Namespaces.traitFromData")
        return traitBuilderFunc?.call(withArguments: [jsonData])
    }
    
    static func recordsToJsRecords(_ records: [HCRecordFields]) -> [JSValue] {
        return records.compactMap {
            recordToJsRecord($0)
        }
    }
}
