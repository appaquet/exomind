import Foundation
import Exocore
import SwiftProtobuf

typealias EntityId = String
typealias TraitId = String

extension Exocore_Store_Entity {
    func toExtension() -> EntityExt {
        EntityExt(entity: self)
    }
}

class EntityExt {
    let inner: Exocore_Store_Entity;

    private let idTraits: [TraitId: Exocore_Store_Trait]
    private let typeTraits: [TraitId: [Exocore_Store_Trait]]
    private var traitInstances: [TraitId: Message] = [:]
    private let _priorityTrait: Exocore_Store_Trait?;

    var creationDate: Date
    var modificationDate: Date?
    var anyDate: Date

    init(entity: Exocore_Store_Entity) {
        self.inner = entity
        self.anyDate = Date()

        var priorityTrait: (Exocore_Store_Trait, TraitConstants)?

        // check if it's a special entity (ex: inbox)
        if let entityConstants = TraitsConstants[entity.id] {
            for trait in entity.traits {
                if trait.id == entity.id {
                    priorityTrait = (trait, entityConstants)
                    break
                }
            }
        }

        // index traits by ids and types
        var idTraits: [String: Exocore_Store_Trait] = [:]
        var typeTraits: [String: [Exocore_Store_Trait]] = [:]
        for trait in entity.traits {
            idTraits[trait.id] = trait

            let traitType = trait.canonicalFullName()
            var traits = typeTraits[traitType] ?? []
            traits.append(trait)
            typeTraits[traitType] = traits

            let traitConstants: TraitConstants?;
            if let entityConsts = TraitsConstants[entity.id], entity.id == trait.id {
                traitConstants = entityConsts
            } else {
                traitConstants = TraitsConstants[traitType]
            }

            if let traitConstants = traitConstants, (priorityTrait == nil || traitConstants.order < priorityTrait!.1.order) {
                priorityTrait = (trait, traitConstants)
            }
        }
        self.idTraits = idTraits
        self.typeTraits = typeTraits
        self._priorityTrait = priorityTrait.map({ $0.0 })
        self.creationDate = entity.hasCreationDate ? entity.creationDate.date : Date()
        self.modificationDate = entity.hasModificationDate ? entity.modificationDate.date : nil
        self.anyDate = self.modificationDate ?? self.creationDate
    }

    var id: EntityId {
        get {
            self.inner.id
        }
    }

    func trait<T: Message>(withId id: TraitId) -> TraitInstance<T>? {
        guard let trait = self.idTraits[id] else {
            return nil
        }

        if let message = self.traitInstances[id] as? T {
            return TraitInstance(entity: self, trait: trait, message: message)
        }

        guard let message = try? T.init(unpackingAny: trait.message) else {
            return nil
        }
        self.traitInstances[id] = message

        return TraitInstance(entity: self, trait: trait, message: message)
    }

    func trait(anyWithId id: TraitId) -> AnyTraitInstance? {
        guard let trait = self.idTraits[id],
              let traitConstants = TraitsConstants[trait.canonicalFullName()] else {
            return nil
        }

        switch traitConstants.traitType {
        case .inbox:
            let trait: TraitInstance<Exomind_Base_V1_Collection>? = self.trait(withId: trait.id)
            return trait
        case .favorites:
            let trait: TraitInstance<Exomind_Base_V1_Collection>? = self.trait(withId: trait.id)
            return trait
        case .emailThread:
            let trait: TraitInstance<Exomind_Base_V1_EmailThread>? = self.trait(withId: trait.id)
            return trait
        case .draftEmail:
            let trait: TraitInstance<Exomind_Base_V1_DraftEmail>? = self.trait(withId: trait.id)
            return trait
        case .email:
            let trait: TraitInstance<Exomind_Base_V1_Email>? = self.trait(withId: trait.id)
            return trait
        case .collection:
            let trait: TraitInstance<Exomind_Base_V1_Collection>? = self.trait(withId: trait.id)
            return trait
        case .task:
            let trait: TraitInstance<Exomind_Base_V1_Task>? = self.trait(withId: trait.id)
            return trait
        case .note:
            let trait: TraitInstance<Exomind_Base_V1_Note>? = self.trait(withId: trait.id)
            return trait
        case .link:
            let trait: TraitInstance<Exomind_Base_V1_Link>? = self.trait(withId: trait.id)
            return trait
        }
    }

    func traitOfType<T: Message>(_ message: T.Type) -> TraitInstance<T>? {
        let traits = self.typeTraits[message.protoMessageName] ?? []
        return traits.compactMap({ (trait) -> TraitInstance<T>? in
                    self.trait(withId: trait.id)
                })
                .first
    }

    func traitsOfType<T: Message>(_ message: T.Type) -> [TraitInstance<T>] {
        let traits = self.typeTraits[message.protoMessageName] ?? []
        return traits.compactMap({ (trait) -> TraitInstance<T>? in
            self.trait(withId: trait.id)
        })
    }

    lazy var priorityTrait: AnyTraitInstance? = {
        guard let trait = self._priorityTrait else {
            return nil
        }

        return self.trait(anyWithId: trait.id)
    }()
}

extension EntityExt: Equatable {
    static func ==(lhs: EntityExt, rhs: EntityExt) -> Bool {
        lhs.id == rhs.id && lhs.anyDate == rhs.anyDate
    }
}

protocol AnyTraitInstance {
    var entity: EntityExt? { get }
    var id: TraitId { get }
    var trait: Exocore_Store_Trait { get }
    var constants: TraitConstants? { get }
    var type: TraitType? { get }
    var displayName: String { get }
    var strippedDisplayName: String { get }
    var creationDate: Date { get }
    var modificationDate: Date? { get }

    func typeInstance() -> TraitTypeInstance?
}

struct TraitInstance<T: Message>: AnyTraitInstance {
    weak var entity: EntityExt?;
    let trait: Exocore_Store_Trait
    let message: T
    let displayName: String
    let constants: TraitConstants?

    init(entity: EntityExt, trait: Exocore_Store_Trait, message: T) {
        self.entity = entity
        self.trait = trait
        self.message = message
        self.constants = getTraitConstants(entity: entity, trait: trait)

        if let constants = self.constants {
            self.displayName = TraitInstance.getDisplayName(constants: constants, message: message)
        } else {
            self.displayName = "*UNKNOWN*"
        }
    }

    var id: TraitId {
        get {
            self.trait.id
        }
    }

    var creationDate: Date {
        get {
            self.trait.creationDate.date
        }
    }

    var modificationDate: Date? {
        get {
            if self.trait.hasModificationDate {
                return self.trait.modificationDate.date
            }

            return nil
        }
    }

    var anyDate: Date {
        get {
            self.modificationDate ?? self.creationDate
        }
    }

    var type: TraitType? {
        get {
            self.constants?.traitType
        }
    }

    func typeInstance() -> TraitTypeInstance? {
        guard let constants = self.constants else {
            return nil
        }

        switch constants.traitType {
        case .inbox:
            return .inbox(trait: self as! TraitInstance<Exomind_Base_V1_Collection>)
        case .favorites:
            return .favorites(trait: self as! TraitInstance<Exomind_Base_V1_Collection>)
        case .emailThread:
            return .emailThread(trait: self as! TraitInstance<Exomind_Base_V1_EmailThread>)
        case .draftEmail:
            return .draftEmail(trait: self as! TraitInstance<Exomind_Base_V1_DraftEmail>)
        case .email:
            return .email(trait: self as! TraitInstance<Exomind_Base_V1_Email>)
        case .collection:
            return .collection(trait: self as! TraitInstance<Exomind_Base_V1_Collection>)
        case .task:
            return .task(trait: self as! TraitInstance<Exomind_Base_V1_Task>)
        case .note:
            return .note(trait: self as! TraitInstance<Exomind_Base_V1_Note>)
        case .link:
            return .link(trait: self as! TraitInstance<Exomind_Base_V1_Link>)
        }
    }

    func toAny() -> AnyTraitInstance? {
        self.entity?.trait(anyWithId: self.id)
    }

    var strippedDisplayName: String {
        get {
            if let constants = self.constants {
                return TraitInstance.getDisplayName(constants: constants, message: message, strip: true)
            } else {
                return "*UNKNOWN*"
            }
        }
    }

    private static func getDisplayName<M: Message>(constants: TraitConstants, message: M, strip: Bool = false) -> String {
        if let name = constants.name {
            return name
        }

        var name: String?;
        switch message {
        case let emailThread as Exomind_Base_V1_EmailThread:
            name = emailThread.subject.nonEmpty() ?? "Untitled email"
        case let draftEmail as Exomind_Base_V1_DraftEmail:
            name = draftEmail.subject.nonEmpty() ?? "Untitled email"
        case let email as Exomind_Base_V1_Email:
            name = email.subject.nonEmpty() ?? "Untitled email"
        case let collection as Exomind_Base_V1_Collection:
            if strip && collection.name.startsWithEmoji() {
                let (_, rest) = collection.name.splitFirstEmoji()
                name = rest.nonEmpty() ?? "Untitled collection"
            } else {
                name = collection.name.nonEmpty() ?? "Untitled collection"
            }
        case let task as Exomind_Base_V1_Task:
            name = task.title.nonEmpty() ?? "Untitled task"
        case let note as Exomind_Base_V1_Note:
            name = note.title.nonEmpty() ?? "Untitled note"
        case let link as Exomind_Base_V1_Link:
            name = link.title.nonEmpty() ?? "Untitled link"
        default:
            name = nil
        }

        return name ?? constants.nameDefault ?? "*UNKNOWN*"
    }
}

enum TraitType {
    case inbox
    case favorites
    case emailThread
    case draftEmail
    case email
    case collection
    case task
    case note
    case link
}

enum TraitTypeInstance {
    case inbox(trait: TraitInstance<Exomind_Base_V1_Collection>)
    case favorites(trait: TraitInstance<Exomind_Base_V1_Collection>)
    case emailThread(trait: TraitInstance<Exomind_Base_V1_EmailThread>)
    case draftEmail(trait: TraitInstance<Exomind_Base_V1_DraftEmail>)
    case email(trait: TraitInstance<Exomind_Base_V1_Email>)
    case collection(trait: TraitInstance<Exomind_Base_V1_Collection>)
    case task(trait: TraitInstance<Exomind_Base_V1_Task>)
    case note(trait: TraitInstance<Exomind_Base_V1_Note>)
    case link(trait: TraitInstance<Exomind_Base_V1_Link>)
}

struct TraitConstants {
    let key: String
    let traitType: TraitType
    let name: String?
    let nameDefault: String?
    let icon: String
    let color: Int
    let order: Int
    let collectionLike: Bool
    let canPreview: Bool
}

func getTraitConstants(entity: EntityExt, trait: Exocore_Store_Trait) -> TraitConstants? {
    if let entityConsts = TraitsConstants[entity.id], entity.id == trait.id {
        return entityConsts
    } else {
        return TraitsConstants[trait.canonicalFullName()]
    }
}

let TraitsConstants: [String: TraitConstants] = [
    "inbox": TraitConstants(
            key: "inbox",
            traitType: .inbox,
            name: "Inbox",
            nameDefault: nil,
            icon: "inbox",
            color: 4,
            order: 0,
            collectionLike: true,
            canPreview: false
    ),
    "favorites": TraitConstants(
            key: "favorites",
            traitType: .favorites,
            name: "Favorites",
            nameDefault: nil,
            icon: "star",
            color: 4,
            order: 1,
            collectionLike: true,
            canPreview: false
    ),
    "exomind.base.v1.EmailThread": TraitConstants(
            key: "exomind.base.v1.EmailThread",
            traitType: .emailThread,
            name: nil,
            nameDefault: "Untitled email",
            icon: "envelope-o",
            color: 1,
            order: 2,
            collectionLike: false,
            canPreview: true
    ),
    "exomind.base.v1.DraftEmail": TraitConstants(
            key: "exomind.base.v1.DraftEmail",
            traitType: .draftEmail,
            name: nil,
            nameDefault: "Untitled email",
            icon: "envelope-o",
            color: 6,
            order: 3,
            collectionLike: false,
            canPreview: true
    ),
    "exomind.base.v1.Email": TraitConstants(
            key: "exomind.base.v1.Email",
            traitType: .email,
            name: nil,
            nameDefault: "Untitled email",
            icon: "envelope-o",
            color: 6,
            order: 4,
            collectionLike: false,
            canPreview: true
    ),
    "exomind.base.v1.Collection": TraitConstants(
            key: "exomind.base.v1.Collection",
            traitType: .collection,
            name: nil,
            nameDefault: nil,
            icon: "folder-o",
            color: 2,
            order: 5,
            collectionLike: true,
            canPreview: true
    ),
    "exomind.base.v1.Task": TraitConstants(
            key: "exomind.base.v1.Task",
            traitType: .task,
            name: nil,
            nameDefault: nil,
            icon: "check-square-o",
            color: 7,
            order: 6,
            collectionLike: false,
            canPreview: false
    ),
    "exomind.base.v1.Note": TraitConstants(
            key: "exomind.base.v1.Note",
            traitType: .note,
            name: nil,
            nameDefault: nil,
            icon: "pencil",
            color: 3,
            order: 7,
            collectionLike: false,
            canPreview: true
    ),
    "exomind.base.v1.Link": TraitConstants(
            key: "exomind.base.v1.Link",
            traitType: .link,
            name: nil,
            nameDefault: "Untitled link",
            icon: "link",
            color: 9,
            order: 8,
            collectionLike: false,
            canPreview: true
    )
]
