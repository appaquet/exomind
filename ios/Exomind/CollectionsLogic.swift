//
//  CollectionsLogic.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-12-07.
//  Copyright © 2015 Exomind. All rights reserved.
//

import Foundation
import JavaScriptCore

class CollectionsLogic {

    static func objectAddableCollections(_ entity: HCEntity?, collections: [HCEntity]) -> [HCEntity] {
        guard let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.collectionsLogic.objectAddableCollections")
            else { return [] }
 
        let jsEntity = entity.flatMap(BridgeEntityConverter.entityToJavascript) as AnyObject?
        let jsCollections = collections.map(BridgeEntityConverter.entityToJavascript)
        
        guard let jsArray = builderFunc.call(withArguments: [DomainStore.instance.orNull(jsEntity), jsCollections])
            else { return [] }
        return DomainStore.instance.jsArrayToJSValues(jsArray).compactMap(BridgeEntityConverter.entityFromJavascript)
    }
}
