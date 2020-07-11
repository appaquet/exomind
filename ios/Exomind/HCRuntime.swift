//
//  HCRuntime.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-07-10.
//  Copyright © 2016 Exomind. All rights reserved.
//

import Foundation

class HCNamespaces {
    static var namespaces = [HCNamespace]()

    static func registerNamespace(_ namespace: HCNamespace) {
        HCNamespaces.namespaces.append(namespace)
    }

    static func builderForType(_ fullType: String) -> HCRecordBuilder? {
        for namespace in HCNamespaces.namespaces {
            if let builder = namespace.builderForType(fullType) {
                return builder
            }
        }
        return nil
    }
}

protocol HCNamespace {
    var name: String { get }
    func builderForType(_ typeName: String) -> HCRecordBuilder?
}

protocol HCFieldType {
    var typeName: String { get }

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool

    func equals(_ this: Any, that: Any) -> Bool
}

protocol HCFieldWithSubtype {
    var subtype: HCFieldType { get }
}

class HCStringFieldType: HCFieldType {
    let typeName: String = "string"

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool {
        let thisValue = this.get(field.name) as? String
        let thatValue = that.get(field.name) as? String
        return thisValue == thatValue
    }

    func equals(_ this: Any, that: Any) -> Bool {
        return this as? String == that as? String
    }
}

class HCLongFieldType: HCFieldType {
    let typeName: String = "long"

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool {
        let thisValue = this.get(field.name) as? Int64
        let thatValue = that.get(field.name) as? Int64
        return thisValue == thatValue
    }

    func equals(_ this: Any, that: Any) -> Bool {
        return this as? Int64 == that as? Int64
    }
}

class HCBooleanFieldType: HCFieldType {
    let typeName: String = "boolean"

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool {
        let thisValue = this.get(field.name) as? Bool
        let thatValue = that.get(field.name) as? Bool
        return thisValue == thatValue
    }

    func equals(_ this: Any, that: Any) -> Bool {
        return this as? Bool == that as? Bool
    }
}

class HCDateFieldType: HCFieldType {
    let typeName: String = "date"

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool {
        let thisValue = this.get(field.name) as? Date
        let thatValue = that.get(field.name) as? Date
        return self.equals(thisValue as Any, that: thatValue as Any)
    }

    func equals(_ this: Any, that: Any) -> Bool {
        let thisValue = this as? Date
        let thatValue = that as? Date
        if let thisValue = thisValue, let thatValue = thatValue {
            return (thisValue == thatValue)
        } else {
            // compare reference
            return thisValue == thatValue
        }
    }
}

class HCEntityReferenceFieldType: HCFieldType {
    let typeName: String = "entity_reference"

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool {
        let thisValue = this.get(field.name) as? HCEntityId
        let thatValue = that.get(field.name) as? HCEntityId
        return thisValue == thatValue
    }

    func equals(_ this: Any, that: Any) -> Bool {
        return this as? HCEntityId == that as? HCEntityId
    }
}

class HCTraitReferenceFieldType: HCFieldType {
    let typeName: String = "trait_reference"

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool {
        let thisValue = this.get(field.name) as? HCTraitId
        let thatValue = that.get(field.name) as? HCTraitId
        return thisValue == thatValue
    }

    func equals(_ this: Any, that: Any) -> Bool {
        return this as? HCTraitId == that as? HCTraitId
    }
}

class HCStructureFieldType: HCFieldType {
    let name: String
    let typeName: String = "structure"

    init(name: String) {
        self.name = name
    }

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool {
        let thisValue = this.get(field.name) as? HCRecord
        let thatValue = that.get(field.name) as? HCRecord
        return self.equals(thisValue as Any, that: thatValue as Any)
    }

    func equals(_ this: Any, that: Any) -> Bool {
        let thisValue = this as? HCRecord
        let thatValue = that as? HCRecord
        if let thisValue = thisValue, let thatValue = thatValue {
            return thisValue.equals(thatValue)
        } else {
            return thisValue == nil && thatValue == nil
        }
    }
}

class HCMapFieldType: HCFieldType, HCFieldWithSubtype {
    let subtype: HCFieldType
    let typeName: String = "map"

    init(subtype: HCFieldType) {
        self.subtype = subtype
    }

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool {
        let thisValue = this.getMap(field.name)
        let thatValue = that.getMap(field.name)
        return self.equals(thisValue as Any, that: thatValue as Any)
    }

    func equals(_ this: Any, that: Any) -> Bool {
        let thisValue = this as? [String:Any]
        let thatValue = that as? [String:Any]

        if let thisValue = thisValue, let thatValue = thatValue {
            if thisValue.count == thatValue.count {
                var anyDifferent = false
                for key in thisValue.keys {
                    anyDifferent = anyDifferent || !self.subtype.equals(thisValue[key] as Any, that: thatValue[key] as Any)
                }
                return !anyDifferent
            } else {
                return false
            }
        } else {
            // otherwise, only if both nil
            return thisValue == nil && thatValue == nil
        }
    }
}

class HCArrayFieldType: HCFieldType, HCFieldWithSubtype {
    let subtype: HCFieldType
    let typeName: String = "array"

    init(subtype: HCFieldType) {
        self.subtype = subtype
    }

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool {
        let thisValue = this.getArray(field.name)
        let thatValue = that.getArray(field.name)
        return self.equals(thisValue as Any, that: thatValue as Any)
    }

    func equals(_ this: Any, that: Any) -> Bool {
        let thisValue = this as? [Any]
        let thatValue = that as? [Any]

        if let thisValue = thisValue, let thatValue = thatValue {
            if thisValue.count == thatValue.count {
                var anyDifferent = false
                for i in 0 ..< thisValue.count {
                    anyDifferent = anyDifferent || !self.subtype.equals(thisValue[i], that: thatValue[i])
                }
                return !anyDifferent
            } else {
                return false
            }
        } else {
            // otherwise, only if both nil
            return thisValue == nil && thatValue == nil
        }
    }
}

class HCOptionFieldType: HCFieldType, HCFieldWithSubtype {
    let subtype: HCFieldType
    let typeName: String = "option"

    init(subtype: HCFieldType) {
        self.subtype = subtype
    }

    func equals(_ field: HCField, this: HCRecordFields, that: HCRecordFields) -> Bool {
        // make sure that both records have the fields
        switch (this.get(field.name), that.get(field.name)) {
        case (.none, .none):
            return true
        case (.some(_), .none):
            return false
        case (.none, .some(_)):
            return false
        default:
            // if any of the 2 are are nil, they should be both nil in order to be equal
            if  let thisValue = this.get(field.name),
                let thatValue = that.get(field.name),
                (thisValue == nil || thatValue == nil) {
                return thisValue == nil && thatValue == nil
            }
            
            return self.subtype.equals(field, this: this, that: that)
        }
    }

    func equals(_ this: Any, that: Any) -> Bool {
        return false
    }
}

class HCField {
    let name: String
    let type: HCFieldType
    init(name: String, type: HCFieldType) {
        self.name = name
        self.type = type
    }
}

protocol HCRecordSchema {
    var fields: [HCField] { get }
}

typealias HCEntityId = String
class HCEntity: Equatable {
    let id: HCEntityId
    let traits: [HCTrait]

    init(id: String, traits: [HCTrait]) {
        self.id = id
        self.traits = traits
    }

    lazy var traitsById: [HCTraitId : HCTrait] = {
        var ret = [HCTraitId : HCTrait]()
        self.traits.forEach { (trait: HCTrait) in
            if let id = trait.traitId {
                ret[id] = trait
            }
        }
        return ret
    }()

    lazy var traitsByType: [String : [HCTrait]] = {
        var ret = [String : [HCTrait]]()
        self.traits.forEach { (trait: HCTrait) in
            if var traits = ret[trait.fullType] {
                traits.append(trait)
                ret[trait.fullType] = traits
            } else {
                ret[trait.fullType] = [trait]
            }
        }
        return ret
    }()
    
    func equals(_ that: HCEntity) -> Bool {
        if (that.id != self.id) {
            return false
        }
        
        if (that.traits.count != self.traits.count) {
            return false
        }

        for i in 0..<self.traits.count {
            if !self.traits[i].equals(that.traits[i]) {
                return false
            }
        }
        
        return true
    }
    
    public static func ==(lhs: HCEntity, rhs: HCEntity) -> Bool {
        return lhs.equals(rhs)
    }
}

enum HCBuildingError: Error {
    case missingField(name:String)
    case invalidFieldType(name:String, expectedType:HCFieldType, value:Any)
}

protocol HCRecordFields: class {
    var type: String { get }
    var fullType: String { get }
    var schema: HCRecordSchema { get }

    func get(_ name: String) -> Any??

    func getArray(_ name: String) -> [Any]?

    func getMap(_ name: String) -> [String:Any]?
}

protocol HCRecordBuilder: HCRecordFields {
    var error: HCBuildingError? { get }

    func set(_ name: String, value: Any)

    func build() -> HCFullRecord?

    func buildSummary() -> HCSummaryRecord?
}

protocol HCTraitBuilder: HCRecordBuilder {
    var traitId: String? { get set }
}

protocol HCStructureBuilder: HCRecordBuilder {
}

protocol HCRecord: HCRecordFields {
    func clone() -> HCRecord
}

extension HCRecord {
    func toBuilder() -> HCRecordBuilder {
        let builder = HCNamespaces.builderForType(self.fullType)!
        for field in self.schema.fields {
            if let fieldValue = self.get(field.name) {
                if field.type is HCArrayFieldType {
                    builder.set(field.name, value: self.getArray(field.name) as Any)
                } else if field.type is HCMapFieldType {
                    builder.set(field.name, value: self.getMap(field.name) as Any)
                } else if let value = fieldValue {
                    builder.set(field.name, value: value)
                }
            }
        }
        return builder
    }

    func diff(_ that: HCRecord) -> [HCFieldDifference] {
        var diffs: [HCFieldDifference] = []
        for field in self.schema.fields {
            let equals = field.type.equals(field, this: self, that: that)
            if !equals {
                var thisAny: Any?
                var thatAny: Any?
                if  let thisValue = self.get(field.name) {
                    thisAny = thisValue
                }
                if let thatValue = that.get(field.name) {
                    thatAny = thatValue
                }
                
                diffs.append(HCFieldDifference(field: field, thisValue: thisAny, thatValue: thatAny))
            }
        }
        return diffs
    }

    func equals(_ that: HCRecord) -> Bool {
        return self.diff(that).isEmpty
    }
}

class HCFieldDifference {
    let field: HCField
    let thisValue: Any?
    let thatValue: Any?

    init(field: HCField, thisValue: Any?, thatValue: Any?) {
        self.field = field
        self.thisValue = thisValue
        self.thatValue = thatValue
    }
}


typealias HCTraitId = String
protocol HCTrait: HCRecord {
    var traitId: HCTraitId? { get set }
    var modificationDate: Date? { get }
    var creationDate: Date? { get }
}

protocol HCStructure: HCRecord {
}

protocol HCSummaryRecord: HCRecord {
}

extension HCSummaryRecord {
    func clone() -> HCRecord {
        let newRecord = self.toBuilder().buildSummary()!
        if let newTrait = newRecord as? HCTrait, let thisTrait = self as? HCTrait {
            newTrait.traitId = thisTrait.traitId
        }
        return newRecord
    }
}

protocol HCFullRecord: HCRecord {
}

extension HCFullRecord {
    func clone() -> HCRecord {
        let newRecord = self.toBuilder().build()!
        if let newTrait = newRecord as? HCTrait, let thisTrait = self as? HCTrait {
            newTrait.traitId = thisTrait.traitId
        }
        return newRecord
    }
}
