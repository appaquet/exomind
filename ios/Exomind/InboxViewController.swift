
import UIKit
import Exocore

class InboxViewController: UIViewController {
    private let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)
    private var childrenVC: ChildrenViewController!

    override func viewDidLoad() {
        super.viewDidLoad()

        self.title = "Inbox"
        self.setupChildrenVC()
    }

    private func setupChildrenVC() {
        self.childrenVC = (mainStoryboard.instantiateViewController(withIdentifier: "ChildrenViewController") as! ChildrenViewController)
        self.addChild(self.childrenVC)
        self.view.addSubview(self.childrenVC.view)

        self.childrenVC.loadData(fromChildrenOf: "inbox")

        self.childrenVC.setItemClickHandler { [weak self] in
            self?.handleItemClick($0)
        }

        self.childrenVC.setSwipeActions(
                [
                    ChildrenViewSwipeAction(action: .check, color: Stylesheet.collectionSwipeDoneBg, state: .state1, mode: .exit, handler: { [weak self] (entity) -> Void in
                        self?.handleDone(entity)
                    }),
                    ChildrenViewSwipeAction(action: .clock, color: Stylesheet.collectionSwipeLaterBg, state: .state3, mode: .switch, handler: { [weak self] (entity) -> Void in
                        self?.handleMoveLater(entity)
                    }),
                    ChildrenViewSwipeAction(action: .folderOpen, color: Stylesheet.collectionSwipeAddCollectionBg, state: .state4, mode: .switch, handler: { [weak self] (entity) -> Void in
                        self?.handleAddToCollection(entity)
                    })
                ]
        )
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        self.setupNavigationActions()
    }

    private func setupNavigationActions() {
        let nav = (self.navigationController as! NavigationController)
        nav.resetState()

        nav.setBarActions([
            NavigationControllerBarAction(icon: .search, handler: { [weak self] () -> Void in
                (self?.navigationController as? NavigationController)?.showSearch("inbox")
            })
        ])

        // quick button only visible in current
        nav.setQuickButtonActions([
            QuickButtonAction(icon: .clock, handler: { () -> Void in
                // TODO: Goto snoozed
            }),
            QuickButtonAction(icon: .plus, handler: { [weak self] () -> Void in
                self?.handleCreateObject()
            }),
            QuickButtonAction(icon: .check, handler: { () -> Void in
                // TODO: Goto History
            })
        ])
    }

    private func handleCreateObject() -> ()? {
        (self.navigationController as? NavigationController)?.showCreateObject("inbox") { [weak self] (entity) -> Void in
            guard let entity = entity else {
                return
            }
            (self?.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
        }
    }

    private func handleItemClick(_ entity: EntityExt) {
        (self.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
    }

    private func handleDone(_ entity: EntityExt) {
        guard let childTrait = entity
                .traitsOfType(Exomind_Base_CollectionChild.self)
                .first(where: { $0.message.collection.entityID == "inbox" }) else {
            return
        }

        let mutation = MutationBuilder
                .updateEntity(entityId: entity.id)
                .deleteTrait(traitId: childTrait.id)
                .build()
        ExocoreClient.store.mutate(mutation: mutation)
    }

    private func handleMoveLater(_ entity: EntityExt) {
        // TODO:
//        (self.navigationController as? NavigationController)?.showTimeSelector(forEntity: entity) { completed in
//            if (completed) {
//                ExomindDSL.on(entity).relations.removeParent(parentId: self.entityId)
//            }
//        }
    }

    private func handleAddToCollection(_ entity: EntityExt) {
        // TODO:
//        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: entity)
    }

    deinit {
        print("InboxViewController > Deinit")
    }

}
