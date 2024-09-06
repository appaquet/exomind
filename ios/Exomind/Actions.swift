import Foundation
import FontAwesome_swift

class Actions {
    static func forEntity(_ entity: EntityExt, parentId: EntityId? = nil, section: ActionSection? = nil, navController: NavigationController? = nil) -> [Action] {
        var ret: [PrioAction] = []

        if let parentId = parentId {
            ret.append(PrioAction(prio: 10, action: Actions.removeFromParent([entity], parentId: parentId)))
        }

        if !isSpecialEntity(entity.id) {
            if let navController = navController, section != .snoozed {
                let removeFromParent = parentId == "inbox"
                ret.append(PrioAction(prio: 18, action: Actions.snooze([entity], navController: navController, parentId: parentId, removeFromParent: removeFromParent)))
            }

            if parentId != "inbox" && !Collections.hasParent(entity: entity, parentId: "inbox") {
                ret.append(PrioAction(prio: 13, action: Actions.addToInbox([entity])))
            }
        }

        if let parentId = parentId, !isSpecialEntity(parentId) {
            if !Collections.isPinnedInParent(entity, parentId: parentId) {
                ret.append(PrioAction(prio: 30, action: Actions.pinInParent(entity, parentId: parentId)))
            } else {
                ret.append(PrioAction(prio: 31, action: Actions.unpinInParent(entity, parentId: parentId)))
            }
        }

        if let navController = navController {
            ret.append(PrioAction(prio: 20, action: Actions.selectEntityCollection([entity], navController: navController)))
            ret.append(PrioAction(prio: 50, action: Actions.delete(entity, navController: navController)))
        }

        if let parentId = parentId {
            ret.append(PrioAction(prio: 32, action: Actions.moveTopParent([entity], parentId: parentId)))
        }

        if section == .snoozed {
            ret.append(PrioAction(prio: 10, action: Actions.removeSnooze(entity)))
        }

        ret.append(PrioAction(prio: 40, action: Actions.copyLink(entity)))

        return ret.sorted {
                    $0.prio < $1.prio
                }
                .map {
                    $0.action
                }
    }

    static func forSelectedEntities(_ entities: [EntityExt], parentId: EntityId? = nil, section: ActionSection? = nil, navController: NavigationController? = nil) -> [Action] {
        var ret: [PrioAction] = []

        if let parentId = parentId {
            ret.append(PrioAction(prio: 10, action: Actions.removeFromParent(entities, parentId: parentId)))
            ret.append(PrioAction(prio: 32, action: Actions.moveTopParent(entities, parentId: parentId)))
        }

        if let navController = navController, section != .snoozed {
            let removeFromParent = parentId == "inbox"
            ret.append(PrioAction(prio: 18, action: Actions.snooze(entities, navController: navController, parentId: parentId, removeFromParent: removeFromParent)))
        }

        if let navController = navController {
            ret.append(PrioAction(prio: 20, action: Actions.selectEntityCollection(entities, navController: navController)))
        }

        return ret.sorted {
                    $0.prio < $1.prio
                }
                .map {
                    $0.action
                }
    }

    static func forEntityCreation(_ parentId: EntityId? = nil) -> [Action] {
        var ret: [PrioAction] = []

        ret.append(PrioAction(prio: 10, action: Actions.createNote(parentId)))
        ret.append(PrioAction(prio: 11, action: Actions.createCollection(parentId)))
        ret.append(PrioAction(prio: 13, action: Actions.createTask(parentId)))

        return ret.sorted {
                    $0.prio < $1.prio
                }
                .map {
                    $0.action
                }
    }

    static func removeFromParent(_ entities: [EntityExt], parentId: String) -> Action {
        Action(key: .removeParent, label: "Remove", icon: .check, destructive: true, swipeColor: Stylesheet.collectionSwipeDoneBg, swipeSide: .leading) { cb in
            Commands.removeFromParent(entities: entities, parentId: parentId)
            cb(.successRemoved)
        }
    }

    static func addToInbox(_ entities: [EntityExt]) -> Action {
        Action(key: .addInbox, label: "Move to inbox", icon: .inbox, swipeColor: Stylesheet.collectionSwipeMoveInboxBg, swipeSide: .trailing) { cb in
            Commands.addToParent(entities: entities, parentId: "inbox")
            cb(.success)
        }
    }

    static func snooze(_ entities: [EntityExt], navController: NavigationController, parentId: String? = nil, removeFromParent: Bool = false) -> Action {
        Action(key: .snooze, label: "Snooze...", icon: .clock, destructive: removeFromParent, swipeColor: Stylesheet.collectionSwipeSnoozeBg, swipeSide: .leading) { (cb) in
            navController.showTimeSelector(forEntities: entities) { completed in
                if (completed) {
                    if removeFromParent {
                        Commands.removeFromParent(entities: entities, parentId: "inbox")
                        cb(.successRemoved)
                    } else {
                        cb(.success)
                    }
                } else {
                    cb(.cancelled)
                }
            }
        }
    }

    static func selectEntityCollection(_ entities: [EntityExt], navController: NavigationController) -> Action {
        Action(key: .selectionCollection, label: "Add to collections...", icon: .folderOpen, swipeColor: Stylesheet.collectionSwipeCollectionBg, swipeSide: .trailing) { (cb) in
            navController.showCollectionSelector(forEntities: entities)
            cb(.success)
        }
    }

    static func removeSnooze(_ entity: EntityExt) -> Action {
        Action(key: .removeSnooze, label: "Unsnooze", icon: .clock, swipeColor: Stylesheet.collectionSwipeSnoozeBg, swipeSide: .leading) { (cb) in
            Commands.removeSnooze(entity)
            Commands.addToParent(entity: entity, parentId: "inbox")
            cb(.success)
        }
    }

    static func moveTopParent(_ entities: [EntityExt], parentId: EntityId) -> Action {
        Action(key: .moveTopParent, label: "Move to top", icon: .arrowUp) { (cb) in
            Commands.addToParent(entities: entities, parentId: parentId)
            cb(.success)
        }
    }

    static func pinInParent(_ entity: EntityExt, parentId: EntityId) -> Action {
        Action(key: .pinParent, label: "Pin to top", icon: .thumbtack) { (cb) in
            Commands.pinEntityInParent(entity: entity, parentId: parentId)
            cb(.success)
        }
    }

    static func unpinInParent(_ entity: EntityExt, parentId: EntityId) -> Action {
        Action(key: .pinParent, label: "Unpin from top", icon: .thumbtack) { (cb) in
            Commands.unpinEntityInParent(entity: entity, parentId: parentId)
            cb(.success)
        }
    }

    static func copyLink(_ entity: EntityExt) -> Action {
        Action(key: .copyLink, label: "Copy link", icon: .link) { (cb) in
            UIPasteboard.general.string = "entity://\(entity.id)"
            cb(.success)
        }
    }

    static func delete(_ entity: EntityExt, navController: NavigationController) -> Action {
        Action(key: .delete, label: "Delete", icon: .trash) { (cb) in
            let ctrl = UIAlertController(title: "Deletion", message: "Do you want to permanently delete the entity?", preferredStyle: .alert)
            ctrl.addAction(UIAlertAction(title: "Yes", style: .default) { _ in
                Commands.delete(entity)
                cb(.success)
            })
            ctrl.addAction(UIAlertAction(title: "No", style: .cancel) { _ in
                cb(.cancelled)
            })
            navController.present(ctrl, animated: true)
        }
    }

    static func createNote(_ parentId: EntityId?) -> Action {
        Action(key: .createNote, label: "Create note", icon: .pen) { cb in
            Commands.createNote(parentId) { res in
                cb(ActionResult.fromCreateResult(res))
            }
        }
    }

    static func createCollection(_ parentId: EntityId?) -> Action {
        Action(key: .createCollection, label: "Create collection", icon: .folderOpen) { cb in
            Commands.createCollection(parentId) { res in
                cb(ActionResult.fromCreateResult(res))
            }
        }
    }

    static func createTask(_ parentId: EntityId?) -> Action {
        Action(key: .createTask, label: "Create task", icon: .check) { cb in
            Commands.createTask(parentId) { res in
                cb(ActionResult.fromCreateResult(res))
            }
        }
    }

    private static func isSpecialEntity(_ id: EntityId) -> Bool {
        id == "inbox" || id == "favorites"
    }
}

struct Action {
    typealias Handler = (@escaping (ActionResult) -> Void) -> Void

    let key: ActionKey
    let label: String

    let icon: FontAwesome?
    var destructive: Bool = false
    var swipeColor: UIColor?
    var swipeSide: SwipeSide = .none

    let execute: Handler
}

enum ActionResult {
    case success
    case successCreated(EntityExt?)
    case successRemoved
    case cancelled
    case failed(Error)
}

extension ActionResult {
    static func fromCreateResult(_ res: EntityCreateResult) -> ActionResult {
        switch res {
        case .success(let entity):
            return .successCreated(entity)
        case .failed(let err):
            return .failed(err)
        }
    }
}

fileprivate struct PrioAction {
    let prio: Int
    let action: Action
}

enum ActionKey {
    case removeParent
    case addInbox
    case snooze
    case selectionCollection
    case removeSnooze
    case moveTopParent
    case pinParent
    case unpinParent
    case copyLink
    case delete
    case createNote
    case createTask
    case createCollection
}

enum ActionSection {
    case inbox
    case recent
    case snoozed
    case search
}

enum SwipeSide {
    case none
    case leading
    case trailing
}