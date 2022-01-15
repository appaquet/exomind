import Foundation
import FontAwesome_swift

class Actions {
    static func forEntity(_ entity: EntityExt, parentId: EntityId? = nil, section: ActionSection? = nil, navController: NavigationController? = nil) -> [Action] {
        var ret: [PrioAction] = []

//        let parentRel = parentId.flatMap {
//            Commands.getEntityParentRelation(entity: entity, parentId: $0)
//        }
//
        if let parentId = parentId {
            ret.append(PrioAction(prio: 10, action: Actions.removeFromParent(entity, parentId: parentId)))
        }

        if !isSpecialEntity(entity.id) {
            if let navController = navController, section != .snoozed {
                let removeFromParent = parentId == "inbox"
                ret.append(PrioAction(prio: 18, action: Actions.snooze(entity, navController: navController, parentId: parentId, removeFromParent: removeFromParent)))
            }

            if parentId != "inbox" {
                ret.append(PrioAction(prio: 13, action: Actions.addToInbox(entity)))
            }
        }

        if let navController = navController {
            ret.append(PrioAction(prio: 20, action: Actions.selectEntityCollection(entity, navController: navController)))
        }

        if section == .snoozed {
            ret.append(PrioAction(prio: 10, action: Actions.removeSnooze(entity)))
        }

        ret.append(PrioAction(prio: 50, action: Actions.delete(entity)))

        return ret.sorted {
                    $0.prio < $1.prio
                }
                .map {
                    $0.action
                }
    }

    static func removeFromParent(_ entity: EntityExt, parentId: String) -> Action {
        Action(key: .removeParent, label: "Remove from parent", icon: .check, destructive: true, swipeColor: Stylesheet.collectionSwipeDoneBg) { cb in
            Commands.removeFromParent(entity: entity, parentId: parentId)
            cb(.successRemoved)
        }
    }

    static func addToInbox(_ entity: EntityExt) -> Action {
        Action(key: .addInbox, label: "Move to inbox", icon: .inbox, swipeColor: Stylesheet.collectionSwipeMoveInboxBg) { cb in
            do {
                try Commands.addToParent(entity: entity, parentId: "inbox")
                cb(.success)
            } catch {
                cb(.failed(error))
            }
        }
    }

    static func snooze(_ entity: EntityExt, navController: NavigationController, parentId: String? = nil, removeFromParent: Bool = false) -> Action {
        Action(key: .snooze, label: "Snooze...", icon: .clock, destructive: removeFromParent, swipeColor: Stylesheet.collectionSwipeSnoozeBg) { (cb) in
            navController.showTimeSelector(forEntity: entity) { completed in
                if (completed) {
                    if removeFromParent {
                        Commands.removeFromParent(entity: entity, parentId: "inbox")
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

    static func selectEntityCollection(_ entity: EntityExt, navController: NavigationController) -> Action {
        Action(key: .selectionCollection, label: "Add to collections...", icon: .folderOpen, swipeColor: Stylesheet.collectionSwipeCollectionBg) { (cb) in
            navController.showCollectionSelector(forEntity: entity)
        }
    }

    static func removeSnooze(_ entity: EntityExt) -> Action {
        Action(key: .removeSnooze, label: "Remove snooze", icon: .clock) { (cb) in
            Commands.removeSnooze(entity)
        }
    }

    static func delete(_ entity: EntityExt) -> Action {
        Action(key: .delete, label: "Delete", icon: .trash) { (cb) in
            // TODO: Confirmation
            // TODO: Delete
        }
    }


    static func isSpecialEntity(_ id: EntityId) -> Bool {
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

    let execute: Handler
}

enum ActionResult {
    case success
    case successRemoved
    case cancelled
    case failed(Error)
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
    case delete
}

enum ActionSection {
    case inbox
    case recent
    case snoozed
    case search
}