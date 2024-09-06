import UIKit
import Exocore

class HomeViewController: UITableViewController {
    private var query: ManagedQuery?

    private lazy var exmElements: [Element] = {
        [
            Element(title: "Inbox", action: pushInbox, iconName: "inbox"),
            Element(title: "Snoozed", action: pushSnoozed, iconName: "clock-o"),
            Element(title: "Recent", action: pushRecent, iconName: "history")
        ]
    }()
    private var favElements: [Element] = []

    override func viewDidLoad() {
        self.navigationItem.title = "Home"

        let traitQuery = TraitQueryBuilder.refersTo(field: "collection", entityId: "favorites").build()
        let query = QueryBuilder
                .withTrait(Exomind_Base_V1_CollectionChild.self, query: traitQuery)
                .orderByField("weight", ascending: false)
                .count(100)
                .build()
        self.query = ManagedQuery(query: query) { [weak self] in
            self?.loadFavoritesEntities()
        }

        (self.navigationController as? NavigationController)?.pushInbox(false)
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)

        (self.navigationController as? NavigationController)?.resetState()
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        // force reload data when dark/light style has changed
        if self.traitCollection.userInterfaceStyle != previousTraitCollection?.userInterfaceStyle {
            self.tableView.reloadData()
        }
    }

    override func numberOfSections(in tableView: UITableView) -> Int {
        2
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        switch section {
        case 0:
            return self.exmElements.count
        case 1:
            return self.favElements.count
        default:
            return 0
        }
    }

    override func tableView(_ tableView: UITableView, titleForHeaderInSection section: Int) -> String? {
        switch section {
        case 0:
            return "Exomind"
        case 1:
            return "Favorites"
        default:
            return nil
        }
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: "cell") ?? UITableViewCell(style: .default, reuseIdentifier: "cell")

        if let element = self.elementForPath(indexPath) {
            cell.textLabel?.text = element.title
            cell.imageView?.image = element.icon()
        } else {
            cell.textLabel?.text = "Unknown"
            cell.imageView?.image = nil
        }

        return cell
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        if let element = self.elementForPath(indexPath) {
            element.action()
            self.tableView.deselectRow(at: indexPath, animated: false)
        }
    }

    private func loadFavoritesEntities() {
        self.favElements = self.query?.results.map({ (entityRes) -> Element in
            let extEntity = entityRes.entity.toExtension()

            let action: () -> () = { [weak self] in
                self?.handleFavorites(entity: extEntity)
            }

            return Element.fromEntity(extEntity, action: action)
        }) ?? []

        DispatchQueue.main.async { [weak self] in
            self?.tableView.reloadData()
        }
    }

    private func elementForPath(_ indexPath: IndexPath) -> Element? {
        let elements: [Element]
        if indexPath.section == 0 {
            elements = self.exmElements
        } else if indexPath.section == 1 {
            elements = self.favElements
        } else {
            return nil
        }

        if indexPath.item >= elements.count {
            return nil
        }

        return elements[indexPath.item]
    }

    private func pushInbox() {
        (self.navigationController as? NavigationController)?.pushInbox()
    }

    private func pushSnoozed() {
        (self.navigationController as? NavigationController)?.pushSnoozed()
    }

    private func pushRecent() {
        (self.navigationController as? NavigationController)?.pushRecent()
    }

    private func handleFavorites(entity: EntityExt?) {
        if let entity = entity {
            (self.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
        }
    }
}

fileprivate class Element {
    let title: String
    let action: () -> ()
    var entity: EntityExt? = nil

    var iconName: String?

    init(title: String, action: @escaping () -> (), entity: EntityExt? = nil, iconName: String? = nil) {
        self.title = title
        self.action = action
        self.entity = entity
        self.iconName = iconName
    }

    static func fromEntity(_ entity: EntityExt, action: @escaping () -> ()) -> Element {
        guard let priorityTrait = entity.priorityTrait else {
            return Element(title: "Unknown \(entity.id)", action: action, entity: entity)
        }

        return Element(title: priorityTrait.strippedDisplayName, action: action, entity: entity)
    }

    func icon() -> UIImage {
        if let iconName = self.iconName {
            return ObjectsIcon.icon(forName: iconName, color: UIColor.label, dimension: 24)
        } else if let entity = entity, let priorityTrait = entity.priorityTrait {
            return ObjectsIcon.icon(forAnyTrait: priorityTrait, color: UIColor.label, dimension: 24)
        } else {
            return ObjectsIcon.icon(forFontAwesome: .question, color: UIColor.label, dimension: 24)
        }
    }
}
