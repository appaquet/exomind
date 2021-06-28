import UIKit
import Exocore

class SnoozedViewController: UIViewController {
    private let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    private var entityListViewController: EntityListViewController!

    override func viewDidLoad() {
        super.viewDidLoad()

        self.navigationItem.title = "Snoozed"
        self.setupChildrenVC()
    }

    private func setupChildrenVC() {
        self.entityListViewController = (mainStoryboard.instantiateViewController(withIdentifier: "EntityListViewController") as! EntityListViewController)
        self.addChild(self.entityListViewController)
        self.view.addSubview(self.entityListViewController.view)

        var projectSummaryFields = Exocore_Store_Projection()
        projectSummaryFields.fieldGroupIds = [1]
        projectSummaryFields.package = ["exomind.base"]
        var projectSkipRest = Exocore_Store_Projection()
        projectSkipRest.skip = true

        let query = QueryBuilder
                .withTrait(Exomind_Base_Snoozed.self)
                .project(withProjections: [projectSummaryFields, projectSkipRest])
                .count(30)
                .build()

        self.entityListViewController.loadData(fromQuery: query)

        self.entityListViewController.setItemClickHandler { [weak self] in
            self?.handleItemClick($0)
        }

        self.entityListViewController.setSwipeActions([
            ChildrenViewSwipeAction(action: .inbox, color: Stylesheet.collectionSwipeDoneBg, side: .leading, style: .destructive, handler: { [weak self] (entity, callback) -> Void in
                self?.handleCopyInbox(entity)
                callback(true)
            }),
            ChildrenViewSwipeAction(action: .clock, color: Stylesheet.collectionSwipeLaterBg, side: .trailing, style: .normal, handler: { [weak self] (entity, callback) -> Void in
                self?.handleMoveLater(entity)
                callback(false)
            }),
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
            var mutation = try MutationBuilder
                    .updateEntity(entityId: entity.id)
                    .putTrait(message: child, traitId: inboxRelationId)

            if let snoozed = entity.traitOfType(Exomind_Base_Snoozed.self) {
                mutation = mutation.deleteTrait(traitId: snoozed.id)
            }

            ExocoreClient.store.mutate(mutation: mutation.build())
        } catch {
            print("SnoozedViewController > Error copying to inbox: \(error)")
        }
    }

    private func handleMoveLater(_ entity: EntityExt) {
        (self.navigationController as? NavigationController)?.showTimeSelector(forEntity: entity)
    }
}
