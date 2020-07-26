//
//  ExomindDomainUtils.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-11-08.
//  Copyright © 2016 Exomind. All rights reserved.
//

import Foundation

typealias TraitInformationDictionary = [AnyHashable: Any]

enum TraitTypeOld {
    case inbox(inbox: Special)
    case mind(mind: Special)
    case email(email: Email)
    case emailThread(emailThread: EmailThread)
    case draftEmail(draftEmail: DraftEmail)
    case task(task: Task)
    case note(note: Note)
    case link(link: Link)
    case integration(integration: Integration)
    case collection(collection: Collection)
    case unknown
}

class TraitInformationOld {
    static var traitsInformation: TraitInformationDictionary = {
        return DomainStore.instance.jsContext.evaluateScript("exomind.exomindDomainUtils.TRAITS_INFORMATION")!.toDictionary()
    }()
    
    let trait: HCTrait
    let traitInfo: TraitInformationDictionary
    
    init(ofTrait trait: HCTrait) {
        self.trait = trait
        self.traitInfo = TraitInformationOld.information(forTrait: trait)
    }
    
    static func information(forTrait: HCTrait) -> TraitInformationDictionary {
        if let special = forTrait as? Special, special.name == "Inbox" {
            return traitsInformation["inbox"] as! TraitInformationDictionary
        } else if let special = forTrait as? Special, special.name == "Mind" {
            return traitsInformation["mind"] as! TraitInformationDictionary
        } else if let info = traitsInformation[forTrait.fullType] {
            return info as! TraitInformationDictionary
        } else {
            return traitsInformation["unknown"] as! TraitInformationDictionary
        }
    }
    
    lazy var traitType: TraitTypeOld = {
        switch (self.traitInfo["key"] as? String, self.trait) {
            case let (.some("inbox"), cTrait as Special):
                return .inbox(inbox: cTrait)
            case let (.some("mind"), cTrait as Special):
                return .mind(mind: cTrait)
            case let (.some(IntegrationSchema.fullType), cTrait as Integration):
                return .integration(integration: cTrait)
            case let (.some(EmailSchema.fullType), cTrait as Email):
                return .email(email: cTrait)
            case let (.some(EmailThreadSchema.fullType), cTrait as EmailThread):
                return .emailThread(emailThread: cTrait)
            case let (.some(DraftEmailSchema.fullType), cTrait as DraftEmail):
                return .draftEmail(draftEmail: cTrait)
            case let (.some(TaskSchema.fullType), cTrait as Task):
                return .task(task: cTrait)
            case let (.some(NoteSchema.fullType), cTrait as Note):
                return .note(note: cTrait)
            case let (.some(LinkSchema.fullType), cTrait as Link):
                return .link(link: cTrait)
            case let (.some(CollectionSchema.fullType), cTrait as Collection):
                return .collection(collection: cTrait)
            default:
                return .unknown
        }
    }()
    
    lazy var displayName: String = {
        if let name = self.traitInfo["name"] as? String {
            return name
        } else if let nameField = self.traitInfo["name_field"] as? String {
            let defaultName = self.traitInfo["name_default"] as? String
            if let fieldValue = self.trait.get(nameField) as? String? {
                return fieldValue ?? defaultName ?? "*UNKNWON"
            } else {
                return "*UNKNOWN"
            }
        } else {
            return "*UNKNOWN*"
        }
    }()
    
    lazy var icon: String = {
        return self.traitInfo["icon"] as! String
    }()
    
    lazy var color: Int = {
        return self.traitInfo["color"] as! Int
    }()
}

class EntityTraitOld {
    let entity: HCEntity
    let trait: HCTrait
    let traitInfo: TraitInformationOld
    
    init(entity: HCEntity, trait: HCTrait) {
        self.entity = entity
        self.trait = trait
        self.traitInfo = TraitInformationOld(ofTrait: trait)
    }
    
    convenience init?(entity: HCEntity) {
        guard let trait = EntityTraitOld.dominantTrait(entity: entity) else { return nil }
        self.init(entity: entity, trait: trait)
    }
    
    lazy var displayName: String = self.traitInfo.displayName
    
    lazy var icon: String = self.traitInfo.icon
    
    lazy var color: Int = self.traitInfo.color
    
    lazy var traitType: TraitTypeOld = self.traitInfo.traitType
    
    static func dominantTrait(entity: HCEntity) -> HCTrait? {
        var lowestTrait: HCTrait? = nil
        var lowestInfo: TraitInformationDictionary? = nil
        
        for currentTrait in entity.traits {
            let currentInfo = TraitInformationOld.information(forTrait: currentTrait)
            if let unpkLowestInfo = lowestInfo {
                let lowestOrder = unpkLowestInfo["order"] as! Int
                let currentOrder = currentInfo["order"] as! Int
                if currentOrder < lowestOrder {
                    lowestInfo = currentInfo
                    lowestTrait = currentTrait
                }
                
            } else {
                lowestInfo = currentInfo
                lowestTrait = currentTrait
            }
        }
        
        return lowestTrait
    }
}



