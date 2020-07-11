//
//  HCJsonSerialization.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-07-10.
//  Copyright © 2016 Exomind. All rights reserved.
//

import SwiftyJSON
import Foundation

enum HCSerializationError: Error {
    case error(message:String)
    case fieldError(field:String, message:String)
    case fieldBuildingError(error:HCBuildingError)
}

class HCJsonSerialization {
    var error: HCSerializationError?
    
    func serialize(_ entity: HCEntity) -> JSON {
        let jsonTraits = entity.traits.map { serialize($0).object }
        return JSON([
            "id": entity.id,
            "traits": jsonTraits
        ])
    }

    func serialize(_ record: HCRecordFields) -> JSON {
        var out = [String: JSON]()
        out["_type"] = JSON(rawValue: record.fullType)
        out["_summary"] = JSON(rawValue: record is HCSummaryRecord)
        
        if let trait = record as? HCTrait,
            let id = trait.traitId {
            out["_id"] = JSON(rawValue: id)
        }
        
        for field in record.schema.fields {
            if let fieldValue = record.get(field.name) {
                if let value = fieldValue {
                    if let jsonValue = serializeData(record, field: field, fieldType: field.type, value: value) {
                        out[field.name] = jsonValue
                    } else {
                        error = .fieldError(field: field.name, message: "Serialization returned nil")
                    }
                }
            }
        }
        return JSON(out)
    }

    func serializeData(_ record: HCRecordFields, field: HCField, fieldType: HCFieldType, value: Any) -> JSON? {
        let fieldSubtype = (fieldType as? HCFieldWithSubtype)?.subtype
        switch (fieldType, fieldSubtype, value) {
        case (_ as HCStringFieldType, _, let v as String):
            return JSON(rawValue: v)
        case (_ as HCOptionFieldType, _ as HCStringFieldType, let v as String):
            return JSON(rawValue: v)

        case (_ as HCLongFieldType, _, let v as Int64):
            return JSON(rawValue: NSNumber(value: v as Int64))
        case (_ as HCOptionFieldType, _ as HCLongFieldType, let v as Int64):
            return JSON(rawValue: NSNumber(value: v as Int64))

        case (_ as HCBooleanFieldType, _, let v as Bool):
            return JSON(rawValue: v)
        case (_ as HCOptionFieldType, _ as HCBooleanFieldType, let v as Bool):
            return JSON(rawValue: v)

        case (_ as HCEntityReferenceFieldType, _, let v as HCEntityId):
            return JSON(rawValue: v)
        case (_ as HCOptionFieldType, _ as HCEntityReferenceFieldType, let v as HCEntityId):
            return JSON(rawValue: v)

        case (_ as HCTraitReferenceFieldType, _, let v as HCTraitId):
            return JSON(rawValue: v)
        case (_ as HCOptionFieldType, _ as HCTraitReferenceFieldType, let v as HCTraitId):
            return JSON(rawValue: v)

        case (_ as HCDateFieldType, _, let v as Date):
            return JSON(rawValue: Date.ISOStringFromDate(v))
        case (_ as HCOptionFieldType, _ as HCDateFieldType, let v as Date):
            return JSON(rawValue: Date.ISOStringFromDate(v))

        case (_ as HCStructureFieldType, _, let v as HCRecord):
            return serialize(v)
        case (_ as HCOptionFieldType, _ as HCStructureFieldType, let v as HCRecord):
            return serialize(v)

        case (_ as HCArrayFieldType, .some(let subtyp), _):
            if let records = record.getArray(field.name) {
                let json = records.compactMap {
                    serializeData(record, field: field, fieldType: subtyp, value: $0)
                }
                return JSON(json)
            } else {
                return JSON(NSNull.self)
            }

        case (_ as HCMapFieldType, .some(let subtyp), _):
            if let records = record.getMap(field.name) {
                let json = records
                .map {
                    (tup) in (tup.0, serializeData(record, field: field, fieldType: subtyp, value: tup.1))
                }
                .filter {
                    $0.1 != nil
                }
                .map {
                    (tup) in (tup.0, tup.1!)
                }

                return JSON(Dictionary(json))
            } else {
                return JSON(NSNull.self)
            }

        default:
            return nil
        }
    }

    func deserializeBuilder(_ json: JSON) -> HCRecordBuilder? {
        guard
        let type = json["_type"].string
        else {
            error = .error(message: "Missing _type field in JSON payload")
            return nil
        }

        if let builder = HCNamespaces.builderForType(type) {
            deserializeInto(json, builder: builder)
            return builder
        } else {
            self.error = .error(message: "Couldn't find record of type \(type)")
            return nil
        }

    }

    func deserializeEntity(_ json: JSON) -> HCEntity? {
        guard let id = json["id"].string else {
            self.error = .error(message: "Couldn't deserialize entity, id was missing")
            return nil
        }

        let traits = json["traits"].array
        .flatMap {
            (jsonTraits) in
            jsonTraits.compactMap {
                deserializeRecord($0)
            }.compactMap {
                $0 as? HCTrait
            }
        }

        if let traits = traits {
            return HCEntity(id: id, traits: traits)
        } else {
            self.error = .error(message: "Couldn't deserialize entity, traits couldn't be extracted")
            return nil
        }
    }

    func deserializeRecord(_ json: JSON) -> HCRecord? {
        if let builder = self.deserializeBuilder(json) {
            if let err = builder.error {
                self.error = .fieldBuildingError(error: err)
            }

            if let traitBuilder = builder as? HCTraitBuilder,
               let id = json["_id"].string {
                traitBuilder.traitId = id
            }

            let summary = json["_summary"].bool ?? false
            if summary {
                let record = builder.buildSummary()
                if let err = builder.error {
                    self.error = .fieldBuildingError(error: err)
                }
                return record
            } else {
                let record = builder.build()
                if let err = builder.error {
                    self.error = .fieldBuildingError(error: err)
                }
                return record
            }
        } else {
            return nil
        }
    }

    fileprivate func deserializeInto(_ json: JSON, builder: HCRecordBuilder) {
        for field in builder.schema.fields {
            let fieldType = field.type
            let fieldSubtype = (fieldType as? HCFieldWithSubtype)?.subtype
            let jsonValue = json[field.name]
            let jsonType = jsonValue.type

            switch (jsonType, fieldType, fieldSubtype) {

            case (.string, _ as HCStringFieldType, _):
                builder.set(field.name, value: jsonValue.stringValue)
            case (.string, _ as HCOptionFieldType, _ as HCStringFieldType):
                builder.set(field.name, value: jsonValue.stringValue)

            case (.number, _ as HCLongFieldType, _):
                builder.set(field.name, value: jsonValue.int64Value)
            case (.number, _ as HCOptionFieldType, _ as HCLongFieldType):
                builder.set(field.name, value: jsonValue.int64Value)

            case (.bool, _ as HCBooleanFieldType, _):
                builder.set(field.name, value: jsonValue.boolValue)
            case (.bool, _ as HCOptionFieldType, _ as HCBooleanFieldType):
                builder.set(field.name, value: jsonValue.boolValue)

            case (.string, _ as HCDateFieldType, _):
                if let date = Date.dateFromISOString(jsonValue.stringValue) {
                    builder.set(field.name, value: date)
                } else {
                    print("Invalid iso date \(jsonValue.stringValue)")
                }
            case (.string, _ as HCOptionFieldType, _ as HCDateFieldType):
                if let date = Date.dateFromISOString(jsonValue.stringValue) {
                    builder.set(field.name, value: date)
                } else {
                    print("Invalid iso date \(jsonValue.stringValue)")
                }

            case (.string, _ as HCEntityReferenceFieldType, _):
                builder.set(field.name, value: jsonValue.stringValue)
            case (.string, _ as HCOptionFieldType, _ as HCEntityReferenceFieldType):
                builder.set(field.name, value: jsonValue.stringValue)

            case (.string, _ as HCTraitReferenceFieldType, _):
                builder.set(field.name, value: jsonValue.stringValue)
            case (.string, _ as HCOptionFieldType, _ as HCTraitReferenceFieldType):
                builder.set(field.name, value: jsonValue.stringValue)

            case (.array, _ as HCArrayFieldType, _ as HCStructureFieldType):
                let casteds: [Any] = jsonValue.arrayValue.compactMap {
                    self.deserializeRecord($0) as? HCFullRecord
                }
                builder.set(field.name, value: casteds)

            case (.dictionary, _ as HCMapFieldType, _ as HCStructureFieldType):
                var casteds = [String: Any]()
                jsonValue.dictionaryValue.forEach({
                    (tup) in
                    if let value = self.deserializeRecord(tup.1) as? HCFullRecord {
                        casteds[tup.0] = value
                    }
                })
                builder.set(field.name, value: casteds)

            case (.dictionary, _ as HCMapFieldType, _ as HCStringFieldType):
                let casteds: [String:Any] = jsonValue.dictionaryValue.mapPairs {
                    tup in (tup.0, tup.1.stringValue)
                }
                builder.set(field.name, value: casteds)

            case (.dictionary, _ as HCStructureFieldType, _):
                if let casted = self.deserializeRecord(jsonValue) as? HCFullRecord {
                    builder.set(field.name, value: casted)
                }
            case (.dictionary, _ as HCOptionFieldType, _ as HCStructureFieldType):
                if let casted = self.deserializeRecord(jsonValue) as? HCFullRecord {
                    builder.set(field.name, value: casted)
                }

            case (.null, _, _):
                break

            default:
                self.error = .fieldError(field: field.name, message: "Unhandled types: json type = \(jsonType) schema type = \(fieldType) and subtype \(String(describing: fieldSubtype))")
                print("Unhandled field type name \(field.name) in json deserialization: Json type = \(jsonType) schema type = \(fieldType) and subtype \(String(describing: fieldSubtype))")
            }

        }
    }
}
