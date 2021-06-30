import UIKit
import Exocore

class RecentViewController: UIViewController {
    private let objectsStoryboard: UIStoryboard = UIStoryboard(name: "Objects", bundle: nil)

    private var entityListViewController: EntityListViewController!

    override func viewDidLoad() {
        super.viewDidLoad()

        self.navigationItem.title = "Recent"
        self.setupChildrenVC()
    }

    private func setupChildrenVC() {
        self.entityListViewController = (objectsStoryboard.instantiateViewController(withIdentifier: "EntityListViewController") as! EntityListViewController)
        self.addChild(self.entityListViewController)
        self.view.addSubview(self.entityListViewController.view)

        var projectSummaryFields = Exocore_Store_Projection()
        projectSummaryFields.fieldGroupIds = [1]
        projectSummaryFields.package = ["exomind.base"]
        var projectSkipRest = Exocore_Store_Projection()
        projectSkipRest.skip = true

        let query = QueryBuilder
                .all()
                .orderByOperationIds(ascending: false)
                .includeDeleted()
                .project(withProjections: [projectSummaryFields, projectSkipRest])
                .count(30)
                .build()

        self.entityListViewController.loadData(fromQuery: query)

        self.entityListViewController.setItemClickHandler { [weak self] in
            self?.handleItemClick($0)
        }

        self.entityListViewController.setSwipeActions([
            EntityListSwipeAction(action: .inbox, color: Stylesheet.collectionSwipeDoneBg, side: .leading, style: .normal) { [weak self] (entity, callback) -> Void in
                self?.handleCopyInbox(entity)
                callback(true)
            }
        ])
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        (self.navigationController as? NavigationController)?.resetState()
    }

    private func handleItemClick(_ entity: EntityExt) {
        (self.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
    }

    private func handleCopyInbox(_ entity: EntityExt) {
        let inboxRelation = entity
                .traitsOfType(Exomind_Base_CollectionChild.self)
                .first(where: { $0.message.collection.entityID == "inbox" })
        let inboxRelationId = inboxRelation?.id ?? "child_inbox"

        var child = Exomind_Base_CollectionChild()
        child.collection.entityID = "inbox"
        child.weight = UInt64(Date().millisecondsSince1970)

        do {
            let mutation = try MutationBuilder
                    .updateEntity(entityId: entity.id)
                    .putTrait(message: child, traitId: inboxRelationId)
                    .build()

            ExocoreClient.store.mutate(mutation: mutation)
        } catch {
            print("RecentViewController > Error copying to inbox: \(error)")
        }
    }
}
