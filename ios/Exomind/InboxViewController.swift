import UIKit
import Exocore

class InboxViewController: UIViewController {
    private let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    private var entityListViewController: EntityListViewController!

    override func viewDidLoad() {
        super.viewDidLoad()

        self.navigationItem.title = "Inbox"
        self.setupChildrenVC()
    }

    private func setupChildrenVC() {
        self.entityListViewController = (mainStoryboard.instantiateViewController(withIdentifier: "EntityListViewController") as! EntityListViewController)
        self.addChild(self.entityListViewController)
        self.view.addSubview(self.entityListViewController.view)

        self.entityListViewController.loadData(fromChildrenOf: "inbox")

        self.entityListViewController.setItemClickHandler { [weak self] in
            self?.handleItemClick($0)
        }

        self.entityListViewController.setSwipeActions([
            ChildrenViewSwipeAction(action: .check, color: Stylesheet.collectionSwipeDoneBg, state: .state1, mode: .exit, handler: { [weak self] (entity) -> Void in
                self?.handleDone(entity)
            }),
            ChildrenViewSwipeAction(action: .clock, color: Stylesheet.collectionSwipeLaterBg, state: .state3, mode: .switch, handler: { [weak self] (entity) -> Void in
                self?.handleMoveLater(entity)
            }),
            ChildrenViewSwipeAction(action: .folderOpen, color: Stylesheet.collectionSwipeAddCollectionBg, state: .state4, mode: .switch, handler: { [weak self] (entity) -> Void in
                self?.handleAddToCollection(entity)
            })
        ])
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
        ExomindMutations.removeParent(entity: entity, parentId: "inbox")
    }

    private func handleMoveLater(_ entity: EntityExt) {
        (self.navigationController as? NavigationController)?.showTimeSelector(forEntity: entity) { completed in
            if (completed) {
                ExomindMutations.removeParent(entity: entity, parentId: "inbox")
            }
        }
    }

    private func handleAddToCollection(_ entity: EntityExt) {
        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: entity)
    }
}
