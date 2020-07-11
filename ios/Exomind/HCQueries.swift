//
//  HCQueries.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-07-31.
//  Copyright © 2016 Exomind. All rights reserved.
//

import Foundation
import SwiftyJSON

class HCQueries {
    static func Entities() -> HCEntityQuery {
        return HCEntityQuery()
    }
}

protocol HCInnerQuery {
    func toJSON() -> JSON
}

class HCEntityQuery {
    fileprivate var innerQuery: HCInnerQuery?
    fileprivate var paging: HCQueryPaging?
    fileprivate var summary: Bool = false

    func withEntityId(_ entityId: HCEntityId) -> HCEntityQuery {
        return self.appendQuery(HCIdEqual(entityId: entityId))
    }
    
    func withEntityIds(_ entityIds: [HCEntityId]) -> HCEntityQuery {
        return self.appendQuery(HCIdsEqual(entityIds: entityIds))
    }

    func withTrait(_ traitName: String) -> HCEntityQuery {
        return self.appendQuery(HCTraitQuery(traitName: traitName))
    }

    func withTrait(_ traitName: String, traitBuilder: ((HCTraitQuery) -> Void)) -> HCEntityQuery {
        let traitQuery = HCTraitQuery(traitName: traitName)
        traitBuilder(traitQuery)
        return self.appendQuery(traitQuery)
    }
    
    func matches(query: String) -> HCEntityQuery {
        return self.appendQuery(HCMatch(query: query))
    }

    func withSummary() -> HCEntityQuery {
        self.summary = true
        return self
    }
    
    func limit(_ count: Int) -> HCEntityQuery {
        if self.paging == nil {
            self.paging = HCQueryPaging()
        }
        self.paging?.count = count
        return self
    }
    
    func sortBy(_ sorting: String) -> HCEntityQuery {
        if self.paging == nil {
            self.paging = HCQueryPaging()
        }
        self.paging?.sorting = sorting
        return self
    }

    func toJSON() -> JSON {
        var ret = JSON([
                "summary": JSON(booleanLiteral: self.summary),
                "inner": self.innerQuery?.toJSON() ?? JSON.null
        ])
        
        if let paging = self.paging {
            ret["paging"] = JSON([
                "fromToken": paging.fromToken.map { JSON(stringLiteral: $0) } ?? JSON.null,
                "toToken": paging.toToken.map { JSON(stringLiteral: $0) } ?? JSON.null,
                "count": paging.count.map { JSON(integerLiteral: $0) } ?? JSON.null,
                "sorting": paging.sorting.map { JSON(stringLiteral: $0) } ?? JSON.null
            ])
        }
        return ret
    }

    func hash() -> String {
        return toJSON().rawString(.utf8, options: .sortedKeys)?.md5 ?? "nil"
    }

    fileprivate func appendQuery(_ query: HCInnerQuery) -> HCEntityQuery {
        if let currentInner = self.innerQuery {
            self.innerQuery = HCConjunctionQuery(queries: [currentInner, query])
        } else {
            self.innerQuery = query
        }

        return self
    }
}

class HCQueryPaging {
    var fromToken: String?
    var toToken: String?
    var count: Int?
    var sorting: String?
}


class HCTraitQuery: HCInnerQuery {
    fileprivate let traitName: String
    fileprivate var traitQuery: HCInnerQuery?

    init(traitName: String) {
        self.traitName = traitName
    }

    @discardableResult
    func whereField(_ fieldName: String, op: HCFieldComparisonOperator, value: AnyObject) -> HCTraitQuery {
        return self.appendQuery(HCFieldCompare(fieldName: fieldName, operation: op, value: value))
    }

    @discardableResult
    func whereFieldMatch(_ fieldName: String, value: String) -> HCTraitQuery {
        return self.appendQuery(HCFieldMatch(fieldName: fieldName, value: value as AnyObject))
    }

    @discardableResult
    func refersTo(_ entityId: HCEntityId) -> HCTraitQuery {
        return self.appendQuery(HCEntityReferenceEqual(entityId: entityId))
    }

    func toJSON() -> JSON {
        return JSON(
                [
                        "type": JSON(stringLiteral: "inner_with_trait"),
                        "traitName": JSON(stringLiteral: self.traitName),
                        "traitQuery": traitQuery?.toJSON() ?? JSON.null
                ])
    }

    fileprivate func appendQuery(_ query: HCInnerQuery) -> HCTraitQuery {
        if let currentInner = self.traitQuery {
            self.traitQuery = HCConjunctionQuery(queries: [currentInner, query])
        } else {
            self.traitQuery = query
        }

        return self
    }
}

enum HCFieldComparisonOperator: String {
    case Eq = "="
    case LowerEq = "<="
    case Lower = "<"
    case HigherEq = ">="
    case Higher = ">"
}

class HCConjunctionQuery: HCInnerQuery {
    fileprivate let queries: [HCInnerQuery]

    init(queries: [HCInnerQuery]) {
        self.queries = queries
    }

    func toJSON() -> JSON {
        return JSON(
                [
                        "type": JSON(stringLiteral: "inner_conjunction"),
                        "queries": JSON(self.queries.map {
                            $0.toJSON()
                        })
                ])
    }
}

class HCFieldCompare: HCInnerQuery {
    fileprivate let fieldName: String
    fileprivate let operation: HCFieldComparisonOperator
    fileprivate let value: AnyObject

    init(fieldName: String, operation: HCFieldComparisonOperator, value: AnyObject) {
        self.fieldName = fieldName
        self.operation = operation
        self.value = value
    }

    func toJSON() -> JSON {
        return JSON(
                [
                        "type": JSON(stringLiteral: "inner_field_compare"),
                        "fieldName": JSON(stringLiteral: self.fieldName),
                        "operation": JSON(stringLiteral: self.operation.rawValue),
                        "value": JSON(rawValue: self.value)!
                ])
    }
}

class HCFieldMatch: HCInnerQuery {
    fileprivate let fieldName: String
    fileprivate let value: AnyObject

    init(fieldName: String, value: AnyObject) {
        self.fieldName = fieldName
        self.value = value
    }

    func toJSON() -> JSON {
        return JSON(
                [
                        "type": JSON(stringLiteral: "inner_field_match"),
                        "fieldName": JSON(stringLiteral: self.fieldName),
                        "value": JSON(rawValue: self.value)!
                ])
    }
}

class HCEntityReferenceEqual: HCInnerQuery {
    fileprivate let entityId: HCEntityId

    init(entityId: HCEntityId) {
        self.entityId = entityId
    }

    func toJSON() -> JSON {
        return JSON(
                [
                        "type": JSON(stringLiteral: "inner_entity_reference_equal"),
                        "entityId": JSON(stringLiteral: self.entityId)
                ])
    }
}


class HCMatch: HCInnerQuery {
    fileprivate let query: String
    
    init(query: String) {
        self.query = query
    }
    
    func toJSON() -> JSON {
        return JSON(
            [
                "type": JSON(stringLiteral: "inner_match"),
                "query": JSON(stringLiteral: self.query)
            ])
    }
}


class HCIdEqual: HCInnerQuery {
    fileprivate let entityId: HCEntityId

    init(entityId: HCEntityId) {
        self.entityId = entityId
    }

    func toJSON() -> JSON {
        return JSON(
                [
                        "type": JSON(stringLiteral: "inner_id_equal"),
                        "entityId": JSON(stringLiteral: self.entityId)
                ])
    }
}

class HCIdsEqual: HCInnerQuery {
    fileprivate let entityIds: [HCEntityId]
    
    init(entityIds: [HCEntityId]) {
        self.entityIds = entityIds
    }
    
    func toJSON() -> JSON {
        return JSON(
            [
                "type": JSON(stringLiteral: "inner_ids_equal"),
                "entityIds": JSON(self.entityIds.map { JSON(stringLiteral: $0) })
            ])
    }
}

