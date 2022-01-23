import UIKit
import Exocore

class InboxViewController: UIViewController {
    private let objectsStoryboard: UIStoryboard = UIStoryboard(name: "Objects", bundle: nil)

    private var entityListViewController: EntityListViewController!

    override func viewDidLoad() {
        super.viewDidLoad()

        self.navigationItem.title = "Inbox"
        self.setupChildrenVC()
    }

    private func setupChildrenVC() {
        self.entityListViewController = (objectsStoryboard.instantiateViewController(withIdentifier: "EntityListViewController") as! EntityListViewController)
        self.addChild(self.entityListViewController)
        self.view.addSubview(self.entityListViewController.view)

        self.entityListViewController.itemClickHandler = { [weak self] (entity) in
            self?.handleItemClick(entity)
        }

        self.entityListViewController.collectionClickHandler = { [weak self] (entity, collection) in
            self?.handleItemClick(collection)
        }

        self.entityListViewController.actionsForEntity = { [weak self] entity in
            guard let this = self else {
                return []
            }
            let navController = this.navigationController as? NavigationController
            return Actions.forEntity(entity, parentId: "inbox", section: .inbox, navController: navController)
        }

        self.entityListViewController.actionsForSelectedEntities = { [weak self] entities in
            guard let this = self else {
                return []
            }
            let navController = this.navigationController as? NavigationController
            return Actions.forSelectedEntities(entities, parentId: "inbox", section: .inbox, navController: navController)
        }

        self.entityListViewController.loadData(fromChildrenOf: "inbox")
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        self.setupNavigationActions()
    }

    private func setupNavigationActions() {
        if let nav = self.navigationController as? NavigationController {
            nav.resetState()
            nav.setBarActions([
                NavigationControllerBarAction(icon: .search, handler: { [weak self] () -> Void in
                    (self?.navigationController as? NavigationController)?.showSearch("inbox")
                }),
                NavigationControllerBarAction(icon: .checkCircle, handler: { [weak self] () -> Void in
                    self?.entityListViewController.editMode = !(self?.entityListViewController.editMode ?? true)
                })
            ])

            // quick button only visible in current
            nav.setQuickButtonActions([// TODO: replace by actions
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
    }

    private func handleCreateObject() -> ()? {
        (self.navigationController as? NavigationController)?.showCreateObject("inbox") { [weak self] (res) -> Void in
            guard case let .successCreated(entity) = res, let entity = entity else {
                return
            }
            (self?.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
        }
    }

    private func handleItemClick(_ entity: EntityExt) {
        (self.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
    }
}
