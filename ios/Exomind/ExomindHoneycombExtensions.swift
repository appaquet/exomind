//
//  ExomindHoneycombExtensions.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-11-12.
//  Copyright © 2016 Exomind. All rights reserved.
//

import Foundation

extension HCEntityQuery {
    func withParent(entityId: HCEntityId) -> HCEntityQuery {
        return HCQueries.Entities().withTrait(ChildSchema.fullType, traitBuilder: { (qb)  in
            qb.refersTo(entityId)
        })
    }
    
    func toDomainQuery() -> Query {
        let hash = self.hash()
        let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.queries.DomainEntityQueryRaw")
        let queryObj = builderFunc?.construct(withArguments: [self.toJSON().object, hash])
        return Query(jsObj: queryObj!)
    }
}
