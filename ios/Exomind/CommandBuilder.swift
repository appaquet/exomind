//
//  CommandBuilder.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-11-22.
//  Copyright © 2015 Exomind. All rights reserved.
//

import Foundation
import JavaScriptCore

class CommandBuilder {
    static func entityCreate(traits: [HCTrait]) -> Command {
        let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.commands.EntityTraitsCommand")
        let jsTraits = BridgeEntityConverter.recordsToJsRecords(traits)
        let queryObj = builderFunc?.construct(withArguments: [jsTraits])
        return Command(jsObj: queryObj!)
    }
    
    static func entityCreate(traitsBuilder: [HCTraitBuilder]) -> Command {
        let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.commands.EntityTraitsCommand")
        let jsTraits = BridgeEntityConverter.recordsToJsRecords(traitsBuilder)
        let queryObj = builderFunc?.construct(withArguments: [jsTraits])
        return Command(jsObj: queryObj!)
    }
    
    static func entityDelete(_ entityId: HCEntityId) -> Command {
        let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.commands.EntityDeleteCommand")
        let queryObj = builderFunc?.construct(withArguments: [entityId])
        return Command(jsObj: queryObj!)
    }

    static func entityTraitsCommand(_ entityId: HCEntityId, adds: [HCTraitBuilder] = [], puts: [HCTraitBuilder] = [], updates: [HCTraitBuilder] = [], removes: [HCTraitId] = []) -> Command {
        let jsAdds = BridgeEntityConverter.recordsToJsRecords(adds)
        let jsPuts = BridgeEntityConverter.recordsToJsRecords(puts)
        let jsUpdates = BridgeEntityConverter.recordsToJsRecords(updates)
        let entityTraitsCommand = DomainStore.instance.jsContext.evaluateScript("exomind.commands.EntityTraitsCommand")
        let queryObj = entityTraitsCommand?.construct(withArguments: [jsAdds, jsPuts, jsUpdates, removes, entityId])
        return Command(jsObj: queryObj!)
    }
}
