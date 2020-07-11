//
//  ExomindDSL.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-11-08.
//  Copyright © 2016 Exomind. All rights reserved.
//

import Foundation

class ExomindDSL {
    static func on(_ entity: HCEntity) -> EntityDSL {
        return EntityDSL(entity: entity)
    }
    
    static func newEntity(traitsBuilder: [HCTraitBuilder]) -> Command {
        return DomainStore.instance.executeCommand(CommandBuilder.entityCreate(traitsBuilder: traitsBuilder))
    }
    
    static func newEntity(traits: [HCTrait]) -> Command {
        return DomainStore.instance.executeCommand(CommandBuilder.entityCreate(traits: traits))
    }
}

class EntityDSL {
    private let entity: HCEntity
    
    init(entity: HCEntity) {
        self.entity = entity
    }
   
    lazy var mutate: EntityMutator = {
        return EntityMutator(entity: self.entity)
    }()
    
    @discardableResult
    func delete() -> Command {
        let cmd = CommandBuilder.entityDelete(self.entity.id)
        return DomainStore.instance.executeCommand(cmd)
    }

    lazy var relations: EntityRelations = {
        return EntityRelations(entity: self.entity)
    }()
}

class EntityMutator {
    private let entity: HCEntity
    private var adds = [HCTraitBuilder]()
    private var puts = [HCTraitBuilder]()
    private var updates = [HCTraitBuilder]()
    private var removes = [HCTraitId]()
    
    init(entity: HCEntity) {
        self.entity = entity
    }
    
    func add<T : HCTrait>(_ trait: T) -> EntityMutator {
        self.adds.append(trait.toBuilder() as! HCTraitBuilder)
        return self
    }
    
    func add<B : HCTraitBuilder>(_ builder: B) -> EntityMutator {
        self.adds.append(builder)
        return self
    }
    
    func put<T : HCTrait>(_ trait: T) -> EntityMutator {
        self.puts.append(trait.toBuilder() as! HCTraitBuilder)
        return self
    }
    
    func put<B : HCTraitBuilder>(_ builder: B) -> EntityMutator {
        self.puts.append(builder)
        return self
    }
    
    func put<T : HCTrait>(_ traits: [T]) -> EntityMutator {
        traits.forEach { _ = self.put($0) }
        return self
    }
    
    func put<B : HCTraitBuilder>(_ builders: [B]) -> EntityMutator {
        builders.forEach { _ = self.put($0) }
        return self
    }
    
    func update<T : HCTrait>(_ trait: T) -> EntityMutator {
        self.updates.append(trait.toBuilder() as! HCTraitBuilder)
        return self
    }
    
    func update<B : HCTraitBuilder>(_ builder: B) -> EntityMutator {
        self.updates.append(builder)
        return self
    }
    
    func remove(_ id: HCTraitId) -> EntityMutator {
        self.removes.append(id)
        return self
    }
    
    func remove(_ trait: HCTrait) -> EntityMutator {
        if let traitId = trait.traitId {
            self.removes.append(traitId)
        }
        return self
    }
    
    @discardableResult
    func execute() -> Command {
        let cmd = CommandBuilder.entityTraitsCommand(self.entity.id, adds: self.adds, puts: self.puts, updates: self.updates, removes: self.removes)
        return DomainStore.instance.executeCommand(cmd)
    }
}

class EntityRelations {
    let entity: HCEntity
    
    init(entity: HCEntity) {
        self.entity = entity
    }
    
    func getParent(parentId: HCEntityId) -> Child? {
        if  let childs = self.entity.traitsByType[ChildSchema.fullType],
            let child = childs.first(where: { ($0 as? Child)?.to == parentId }) as? Child {
            return child
        } else {
            return nil
        }
    }
    
    func getParents() -> [Child] {
        if  let childs = self.entity.traitsByType[ChildSchema.fullType] {
            return childs.compactMap { $0 as? Child }
        } else {
            return []
        }
    }
    
    func hasParent(parentId: HCEntityId) -> Bool {
        return self.getParent(parentId: parentId) != nil
    }
    
    @discardableResult
    func addParent(parentId: HCEntityId, weight: Int64? = nil, date: Date? = nil) -> Command {
        let child = EntityRelations.buildChild(parentId: parentId, weight: weight, date: date)
        var cmd = ExomindDSL.on(self.entity).mutate.put(child)
        if  let oldChilds = self.entity.traitsByType[OldChildSchema.fullType],
            let oldChild = oldChilds.first(where: { ($0 as? OldChild)?.to == parentId }) {
            cmd = cmd.remove(oldChild)
        }
        
        return cmd.execute()
    }
    
    static func buildChild(parentId: HCEntityId, weight: Int64? = nil, date: Date? = nil) -> ChildFull {
        let finalDate = date ?? Date()
        let finalWeight = weight ?? Int64(finalDate.timeIntervalSince1970) * 1000
        return ChildFull(date: finalDate, to: parentId, weight: finalWeight)
    }
    
    @discardableResult
    func removeParent(parentId: HCEntityId) -> Command {
        let oldChild = OldChildFull(date: Date(), to: parentId)
        if let parent = self.getParent(parentId: parentId) {
            return ExomindDSL.on(self.entity).mutate.remove(parent).put(oldChild).execute()
        } else {
            return ExomindDSL.on(self.entity).mutate.put(oldChild).execute()
        }
    }
    
    @discardableResult
    func postpone(untilDate: Date) -> Command {
        let postponed = PostponedFull(untilDate: untilDate)
        return ExomindDSL.on(self.entity).mutate.put(postponed).execute()
    }
    
    @discardableResult
    func removePostpone() -> Command {
        if  let mulPostponed = self.entity.traitsByType[PostponedSchema.fullType],
            let postponed = mulPostponed.first {
            return ExomindDSL.on(self.entity).mutate.remove(postponed).execute()
        } else {
            return ExomindDSL.on(self.entity).mutate.execute()
        }
    }
}
