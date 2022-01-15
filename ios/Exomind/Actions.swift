import Foundation
import FontAwesome_swift

// TODO: Pin & unpin

class Actions {
    static func forEntity(_ entity: EntityExt, parentId: EntityId? = nil, section: ActionSection? = nil, navController: NavigationController? = nil) -> [Action] {
        var ret: [PrioAction] = []

        if let parentId = parentId {
            ret.append(PrioAction(prio: 10, action: Actions.removeFromParent(entity, parentId: parentId)))
        }

        if !isSpecialEntity(entity.id) {
            if let navController = navController, section != .snoozed {
                let removeFromParent = parentId == "inbox"
                ret.append(PrioAction(prio: 18, action: Actions.snooze(entity, navController: navController, parentId: parentId, removeFromParent: removeFromParent)))
            }

            if parentId != "inbox" && !Collections.hasParent(entity: entity, parentId: "inbox") {
                ret.append(PrioAction(prio: 13, action: Actions.addToInbox(entity)))
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
            ret.append(PrioAction(prio: 20, action: Actions.selectEntityCollection(entity, navController: navController)))
            ret.append(PrioAction(prio: 50, action: Actions.delete(entity, navController: navController)))
        }

        if let parentId = parentId {
            ret.append(PrioAction(prio: 32, action: Actions.moveTopParent(entity, parentId: parentId)))
        }

        if section == .snoozed {
            ret.append(PrioAction(prio: 10, action: Actions.removeSnooze(entity)))
        }

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
            Commands.addToParent(entity: entity, parentId: "inbox")
            cb(.success)
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
            cb(.success)
        }
    }

    static func removeSnooze(_ entity: EntityExt) -> Action {
        Action(key: .removeSnooze, label: "Remove snooze", icon: .clock) { (cb) in
            Commands.removeSnooze(entity)
            cb(.success)
        }
    }

    static func moveTopParent(_ entity: EntityExt, parentId: String) -> Action {
        Action(key: .moveTopParent, label: "Move top", icon: .arrowUp) { (cb) in
            Commands.addToParent(entity: entity, parentId: parentId)
            cb(.success)
        }
    }

    static func pinInParent(_ entity: EntityExt, parentId: String) -> Action {
        Action(key: .pinParent, label: "Pin to top", icon: .thumbtack) { (cb) in
            Commands.pinEntityInParent(entity: entity, parentId: parentId)
            cb(.success)
        }
    }

    static func unpinInParent(_ entity: EntityExt, parentId: String) -> Action {
        Action(key: .pinParent, label: "Unpin from top", icon: .thumbtack) { (cb) in
            Commands.unpinEntityInParent(entity: entity, parentId: parentId)
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
    case moveTopParent
    case pinParent
    case unpinParent
    case delete
}

enum ActionSection {
    case inbox
    case recent
    case snoozed
    case search
}